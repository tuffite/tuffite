"""Verified Distribution upload using Python's standard HTTP and ZIP libraries."""
from __future__ import annotations

import http.client
import json
import os
import re
import shutil
import sys
import stat
import subprocess
import tempfile
import time
import urllib.parse
import zipfile
from pathlib import Path

from . import framework
from .common import TaskError, require



DEFAULT_REPOSITORY = "tuffite/tuffite"


class UploadProgress:
    """Single-line TTY progress, or throttled plain-text CI logs."""

    def __init__(self, total, enabled=True, stream=None):
        self.total = total
        self.enabled = enabled
        self.stream = stream if stream is not None else sys.stderr
        self.interactive = self.stream.isatty()
        self.started = time.monotonic()
        self.last = None
        self.sent = 0
        self.width = 0

    @staticmethod
    def size(value):
        for unit in ("B", "KiB", "MiB", "GiB"):
            if value < 1024 or unit == "GiB":
                return f"{value:.1f} {unit}"
            value /= 1024

    def __enter__(self):
        self.update(0)
        return self

    def update(self, sent):
        self.sent = sent
        if not self.enabled:
            return
        now = time.monotonic()
        interval = 0.1 if self.interactive else 2.0
        if self.last is not None and now - self.last < interval and sent != self.total:
            return
        self.last = now
        fraction = min(sent / self.total, 1.0) if self.total else 1.0
        elapsed = max(now - self.started, 0.001)
        speed = sent / elapsed
        remaining = max(self.total - sent, 0) / speed if speed else None
        eta = f"{int(remaining) // 60:02d}:{int(remaining) % 60:02d}" if remaining is not None else "--:--"
        details = f"{fraction:6.1%} {self.size(sent)}/{self.size(self.total)} {self.size(speed)}/s ETA {eta}"
        if self.interactive:
            columns = max(1, shutil.get_terminal_size((80, 24)).columns - 1)
            bar_width = min(20, max(0, columns - len(details) - len("Upload [] ")))
            filled = int(fraction * bar_width)
            bar = f"[{'#' * filled}{'-' * (bar_width - filled)}] " if bar_width else ""
            line = f"Upload {bar}{details}"[:columns]
            self.stream.write("\r" + line + " " * max(0, self.width - len(line)))
            self.width = len(line)
        else:
            self.stream.write(f"Upload: {details}\n")
        self.stream.flush()

    def __exit__(self, *exception):
        if self.enabled and self.interactive:
            self.stream.write("\n")
            self.stream.flush()


def request(url, token, method="GET", data=None, archive=None, progress=True):
    parsed = urllib.parse.urlsplit(url)
    require(parsed.scheme == "https" and parsed.hostname in ("api.github.com", "uploads.github.com") and not parsed.username and not parsed.password and parsed.port in (None, 443), "unexpected GitHub API URL")
    connection = http.client.HTTPSConnection(parsed.hostname, timeout=120)
    headers = {"Authorization": f"Bearer {token}", "User-Agent": "tuffite-framework", "X-GitHub-Api-Version": "2022-11-28", "Accept": "application/vnd.github+json"}
    try:
        route = urllib.parse.urlunsplit(("", "", parsed.path, parsed.query, ""))
        if archive is None:
            body = json.dumps(data).encode() if data is not None else None
            if body is not None:
                headers["Content-Type"] = "application/json"
            connection.request(method, route, body, headers)
        else:
            total = archive.stat().st_size
            headers.update({"Content-Type": "application/zip", "Content-Length": str(total)})
            connection.putrequest(method, route)
            for name, value in headers.items():
                connection.putheader(name, value)
            connection.endheaders()
            sent = 0
            with UploadProgress(total, progress) as transfer, archive.open("rb") as stream:
                while block := stream.read(1024 * 1024):
                    connection.send(block)
                    sent += len(block)
                    transfer.update(sent)
        response = connection.getresponse()
        body = response.read()
        if response.status == 404 and method == "GET":
            return None
        require(200 <= response.status < 300, f"GitHub {method} failed ({response.status}): {body[:1000].decode(errors='replace')}")
        return json.loads(body) if body else None
    except http.client.HTTPException as error:
        raise TaskError(f"GitHub {method} transfer failed: {error}") from error
    finally:
        connection.close()


def token():
    value = os.environ.get("GITHUB_TOKEN") or os.environ.get("GH_TOKEN")
    if not value:
        try:
            result = subprocess.run(["gh", "auth", "token"], capture_output=True, text=True, check=False)
            value = result.stdout.strip() if result.returncode == 0 else None
        except OSError:
            value = None
    require(value, "upload requires GITHUB_TOKEN, GH_TOKEN, or authenticated gh CLI")
    return value


def archive_distribution(root, destination):
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED, allowZip64=True) as archive:
        for path in sorted(root.rglob("*")):
            require(not path.is_symlink(), f"Distribution archive cannot contain symlinks: {path}")
            if path.is_dir():
                continue
            require(stat.S_ISREG(path.stat().st_mode), f"unsupported archive entry: {path}")
            archive.write(path, path.relative_to(root).as_posix())


def upload(context, args):
    root = framework.distribution_path(args)
    repository = args.repository or os.environ.get("TUFFITE_GITHUB_REPOSITORY") or DEFAULT_REPOSITORY
    require(re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) and all(part not in (".", "..") for part in repository.split("/")), "repository must be owner/name")
    tag = args.tag or os.environ.get("TUFFITE_FRAMEWORK_DIST_RELEASE_TAG") or f"v{framework.framework_version()}"
    print(f"Upload {root} to {repository} ({tag})")
    if context.dry_run:
        return
    framework.verify_distribution(root)
    context.confirm("Upload this verified Distribution?")
    credential = token()
    target = json.loads((root / framework.FRAMEWORK_MANIFEST).read_text(encoding="utf-8"))["target"]
    asset = f"tuffite-framework-{target}.zip"
    endpoint = f"https://api.github.com/repos/{repository}/releases"
    release = request(f"{endpoint}/tags/{urllib.parse.quote(tag, safe='')}", credential)
    if release is None:
        require(args.create, f"Release {tag} does not exist; pass --create")
        release = request(endpoint, credential, "POST", {"tag_name": tag, "name": tag, "prerelease": args.prerelease})
    existing = next((item for item in release["assets"] if item["name"] == asset), None)
    require(not existing or args.replace, f"Release already contains {asset}; pass --replace")
    # Finish archive creation before replacing any remote asset.
    with tempfile.TemporaryDirectory(prefix="tuffite-release-") as directory:
        archive = Path(directory) / asset
        archive_distribution(root, archive)
        if existing:
            request(f"{endpoint}/assets/{existing['id']}", credential, "DELETE")
        url = release["upload_url"].split("{", 1)[0] + "?" + urllib.parse.urlencode({"name": asset})
        request(url, credential, "POST", archive=archive, progress=not args.no_progress and os.environ.get("TUFFITE_FRAMEWORK_DIST_PROGRESS", "1").lower() not in ("0", "false", "no"))
    print(f"Uploaded {asset} to {repository} ({tag})")

