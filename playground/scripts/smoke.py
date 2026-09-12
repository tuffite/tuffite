#!/usr/bin/env python3
"""Run the packaged Playground integration smoke test."""

from __future__ import annotations

import argparse
import functools
import http.server
import json
import os
import platform
import shutil
import signal
import socket
import subprocess
import sys
import tempfile
import threading
import time
import urllib.error
import urllib.request
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PLAYGROUND = Path(__file__).resolve().parents[1]
MACHINE = platform.machine().lower()
ARCHITECTURE = {"amd64": "x64", "x86_64": "x64", "aarch64": "arm64"}.get(
    MACHINE, MACHINE
)
SYSTEM = (
    "windows" if os.name == "nt" else ("macos" if sys.platform == "darwin" else "linux")
)
FIELDS = (
    "shell_api",
    "network_service",
    "explorer",
    "utility",
    "window",
    "path",
    "fs",
    "storage",
    "log",
    "chromium_thread_pool",
    "application_event",
    "large_asset",
    "utility_async",
    "utility_concurrent",
    "utility_session",
    "javascript_extension",
    "explorer_navigation",
    "explorer_execute_javascript",
    "explorer_dialog",
    "explorer_partition",
    "explorer_nested",
    "explorer_surface_messaging",
)


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, _format: str, *args: object) -> None:
        pass


def free_port() -> int:
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


def terminate(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is not None:
        return
    if os.name == "nt":
        subprocess.run(
            ["taskkill", "/PID", str(process.pid), "/T", "/F"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        )
    else:
        os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()


def grant_sandbox_access(path: Path) -> None:
    if os.name != "nt":
        return
    subprocess.run(
        [
            "icacls",
            str(path),
            "/grant",
            "*S-1-5-12:(OI)(CI)(RX)",
            "*S-1-15-3-1024-3424233489-972189580-2057154623-747635277-1604371224-316187997-3786583170-1043257646:(OI)(CI)(RX)",
            "*S-1-15-3-1024-2302894289-466761758-1166120688-1039016420-2430351297-4240214049-4028510897-3317428798:(OI)(CI)(RX)",
            "*S-1-15-2-1:(OI)(CI)(M)",
            "*S-1-15-2-2:(OI)(CI)(M)",
            "/C",
            "/Q",
        ],
        check=True,
        stdout=subprocess.DEVNULL,
    )


def wait_for_cdp(port: int, process: subprocess.Popen[bytes], timeout: int) -> None:
    deadline = time.monotonic() + timeout
    endpoint = f"http://127.0.0.1:{port}/json/version"
    while time.monotonic() < deadline:
        if process.poll() is not None:
            raise RuntimeError(
                f"Playground exited before the CDP endpoint was ready ({process.returncode})"
            )
        try:
            with urllib.request.urlopen(endpoint, timeout=1) as response:
                if response.status == 200:
                    return
        except (OSError, urllib.error.URLError):
            pass
        time.sleep(0.25)
    raise RuntimeError(f"Playground CDP endpoint did not become ready: {endpoint}")


def run_explorer_demo(
    package: Path,
    application: Path,
    staging: Path,
    timeout: int,
    keep_files: bool,
    requested_port: int,
    script: str = "explorer-demo-smoke.mjs",
) -> int:
    test_root = staging / f"tuffite-explorer-smoke-{uuid.uuid4().hex}"
    profile = test_root / "profile"
    stdout_log = test_root / "stdout.log"
    stderr_log = test_root / "stderr.log"
    profile.mkdir(parents=True)
    grant_sandbox_access(test_root)
    port = requested_port or free_port()
    command = [
        str(application),
        "--lang=en-US",
        f"--remote-debugging-port={port}",
        f"--user-data-dir={profile}",
        f"--log-file={test_root / 'tuffite.log'}",
        "--no-first-run",
        "--disable-features=CalculateNativeWinOcclusion",
        "--disable-backgrounding-occluded-windows",
    ]
    process = None
    try:
        with stdout_log.open("wb") as stdout, stderr_log.open("wb") as stderr:
            process = subprocess.Popen(
                command,
                cwd=package,
                stdout=stdout,
                stderr=stderr,
                start_new_session=os.name != "nt",
            )
            wait_for_cdp(port, process, timeout)
            environment = os.environ.copy()
            environment["TUFFITE_PLAYGROUND_CDP_PORT"] = str(port)
            environment["TUFFITE_PLAYGROUND_COMMAND"] = json.dumps(command)
            environment["TUFFITE_PLAYGROUND_CWD"] = str(package)
            subprocess.run(
                ["bun", "run", f"../scripts/{script}"],
                cwd=PLAYGROUND / "ui",
                env=environment,
                check=True,
                timeout=max(timeout, 180),
            )
            if script == "browser-contract-smoke.mjs":
                import sqlite3

                names = set()
                for database in profile.rglob("Cookies"):
                    with sqlite3.connect(database) as connection:
                        for name, value, encrypted in connection.execute(
                            "SELECT name, value, encrypted_value FROM cookies"
                        ):
                            if value or not encrypted:
                                raise RuntimeError("Cookie persistence used an unencrypted value")
                            names.add(name)
                if not {"root", "persistent"}.issubset(names):
                    raise RuntimeError("Persistent Cookie databases were not written")
                print("Cookie databases verified: encrypted values only")
        return 0
    except Exception as error:
        diagnostic = stderr_log.read_text(encoding="utf-8", errors="replace")[-8000:]
        raise RuntimeError(
            f"Explorer demo smoke failed: {error}\n{diagnostic}"
        ) from error
    finally:
        if process is not None:
            terminate(process)
        if not keep_files:
            shutil.rmtree(test_root, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--package",
        default=f"dist/tuffite-playground-{SYSTEM}-{ARCHITECTURE}",
    )
    parser.add_argument("--timeout", type=int, default=90)
    parser.add_argument("--keep-files", action="store_true")
    parser.add_argument(
        "--explorer-demo",
        action="store_true",
        help="Run the detailed cross-platform Explorer CDP smoke.",
    )
    parser.add_argument("--explorer-capabilities", action="store_true",
                        help="Run origin grants and command allowlist smoke.")
    parser.add_argument("--explorer-resources", action="store_true",
                        help="Run native download, PDF, snapshot and PiP smoke.")
    parser.add_argument("--browser-contract", action="store_true",
                        help="Run browser language, dialogs and restart persistence smoke.")
    parser.add_argument("--explorer-webauthn", action="store_true",
                        help="Run WebAuthn resident credential and cancellation smoke.")
    parser.add_argument("--explorer-transfer", action="store_true",
                        help="Run native detach/attach lifecycle and window transfer smoke.")
    parser.add_argument(
        "--cdp-port",
        type=int,
        default=0,
        help="Explorer smoke CDP port; zero selects an available port.",
    )
    parser.add_argument(
        "--staging-dir",
        help="Directory used for the isolated smoke package and profile.",
    )
    parser.add_argument(
        "--single-process",
        action="store_true",
        help="Run Chromium in one process for restricted CI hosts.",
    )
    args = parser.parse_args()
    package = (ROOT / args.package).resolve()
    bundle = package if package.suffix == ".app" else None
    if bundle:
        package = bundle / "Contents/Resources"
    code = bundle / "Contents/MacOS" if bundle else package
    application = code / ("playground.exe" if os.name == "nt" else "playground")
    framework = code / (
        "tuffite.dll"
        if os.name == "nt"
        else (
            "libtuffite.dylib"
            if sys.platform == "darwin"
            else "libtuffite.so"
        )
    )
    business_runtime = code / (
        "playground.dll"
        if os.name == "nt"
        else ("libplayground.dylib" if sys.platform == "darwin" else "libplayground.so")
    )
    for required in (application, framework, business_runtime):
        if not required.is_file():
            raise RuntimeError(f"Packaged Playground is missing: {required}")

    public = os.environ.get("PUBLIC") if os.name == "nt" else None
    staging = (
        Path(args.staging_dir).resolve()
        if args.staging_dir
        else Path(public)
        if public
        else Path(tempfile.gettempdir())
    )
    if args.browser_contract or args.explorer_demo or args.explorer_transfer or args.explorer_capabilities or args.explorer_resources or args.explorer_webauthn:
        return run_explorer_demo(
            package,
            application,
            staging,
            args.timeout,
            args.keep_files,
            args.cdp_port,
            "browser-contract-smoke.mjs" if args.browser_contract else
            "explorer-webauthn-smoke.mjs" if args.explorer_webauthn else
            "explorer-resources-smoke.mjs" if args.explorer_resources else
            "explorer-capabilities-smoke.mjs" if args.explorer_capabilities else
            "explorer-transfer-smoke.mjs" if args.explorer_transfer else "explorer-demo-smoke.mjs",
        )
    test_root = staging / f"tuffite-smoke-{uuid.uuid4().hex}"
    server_root = test_root / "http"
    profile = test_root / "profile"
    deployed = test_root / "package"
    marker = test_root / "result.json"
    stdout_log = test_root / "stdout.log"
    stderr_log = test_root / "stderr.log"
    server_root.mkdir(parents=True)
    profile.mkdir()
    if bundle:
        deployed_bundle = deployed.with_suffix(".app")
        shutil.copytree(bundle, deployed_bundle)
        deployed = deployed_bundle / "Contents/Resources"
    else:
        shutil.copytree(package, deployed)
    application = (deployed_bundle / "Contents/MacOS" if bundle else deployed) / application.name
    grant_sandbox_access(test_root)
    (server_root / "index.html").write_text(
        "tuffite network service smoke", encoding="utf-8"
    )
    # The native visual probe requires a distinct, stable child background.
    # Override only the deployed test copy; interactive themes remain unchanged.
    stylesheets = list((deployed / "assets" / "ui" / "dist" / "assets").glob("styles-*.css"))
    if not stylesheets:
        raise RuntimeError("Packaged Playground visual probe stylesheet is missing")
    for stylesheet in stylesheets:
        with stylesheet.open("a", encoding="utf-8") as fixture:
            fixture.write("\n.explorer-page { background: #182033 !important; }\n")

    port = free_port()
    config_path = deployed / "tuffite.json"
    config = json.loads(config_path.read_text())
    config["app"]["security"]["capabilities"][0]["shellApi"].append({
        "method": "tuffite.net.request",
        "allow": [{"url": f"http://127.0.0.1:{port}/*"}],
    })
    config_path.write_text(json.dumps(config))
    if bundle:
        # Fixture CSS and manifest edits invalidate the copied bundle's seal.
        # Keep native bundle validation active during the integration test.
        subprocess.run(["codesign", "--force", "--sign", "-", str(deployed_bundle)], check=True)
    handler = functools.partial(QuietHandler, directory=str(server_root))
    server = http.server.ThreadingHTTPServer(("127.0.0.1", port), handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    process = None
    try:
        with stdout_log.open("wb") as stdout, stderr_log.open("wb") as stderr:
            command = [
                str(application),
                f"--tuffite-smoke-test={marker}",
                f"--tuffite-network-test-url=http://127.0.0.1:{port}/",
                f"--user-data-dir={profile}",
                "--window-size=1100,1200",
                "--force-device-scale-factor=1",
                "--disable-features=CalculateNativeWinOcclusion",
                "--disable-backgrounding-occluded-windows",
                # Keep startup timers responsive while utility concurrency is
                # proved by an explicit service handshake rather than elapsed time.
                "--disable-background-timer-throttling",
                "--disable-renderer-backgrounding",
                "--no-first-run",
            ]
            if args.cdp_port:
                command.append(f"--remote-debugging-port={args.cdp_port}")
            if args.single_process:
                command.extend(
                    [
                        "--no-sandbox",
                        "--single-process",
                        "--in-process-gpu",
                        "--enable-features=NetworkServiceInProcess2",
                        "--disable-features=AudioServiceOutOfProcess,CalculateNativeWinOcclusion,DirectCompositorThreadIpc",
                    ]
                )
            process = subprocess.Popen(
                command,
                cwd=deployed,
                stdout=stdout,
                stderr=stderr,
                start_new_session=os.name != "nt",
            )
            deadline = time.monotonic() + args.timeout
            while not marker.is_file() and time.monotonic() < deadline:
                if process.poll() is not None:
                    stderr.flush()
                    diagnostic = stderr_log.read_text(
                        encoding="utf-8", errors="replace"
                    )[-8000:]
                    raise RuntimeError(
                        f"Playground exited before smoke completion ({process.returncode})\n{diagnostic}"
                    )
                time.sleep(0.2)
            if not marker.is_file():
                raise RuntimeError(f"Timed out waiting for smoke marker: {marker}")
            result = json.loads(marker.read_text(encoding="utf-8-sig"))
            if result.get("status") != "ok" or any(
                not result.get(field) for field in FIELDS
            ):
                raise RuntimeError(
                    f"Invalid Playground smoke result: {json.dumps(result)}"
                )
            # The native observer closes windows after publishing the marker.
            # Wait for teardown so a successful marker cannot hide a shutdown crash.
            try:
                returncode = process.wait(timeout=15)
            except subprocess.TimeoutExpired as error:
                raise RuntimeError(
                    "Playground did not exit after smoke completion"
                ) from error
            if returncode != 0:
                diagnostic = stderr_log.read_text(encoding="utf-8", errors="replace")[
                    -8000:
                ]
                raise RuntimeError(
                    f"Playground shutdown failed ({returncode})\n{diagnostic}"
                )
            print(json.dumps(result, ensure_ascii=False, separators=(",", ":")))
    finally:
        if process is not None:
            terminate(process)
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
        if not args.keep_files:
            shutil.rmtree(test_root, ignore_errors=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
