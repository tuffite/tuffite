"""Process execution and recoverable state shared by Framework tasks."""
from __future__ import annotations

import contextlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path


class TaskError(RuntimeError):
    """An actionable task failure."""


def require(condition: object, message: str) -> None:
    if not condition:
        raise TaskError(message)


def git(repo: str | Path, *args: str) -> str:
    result = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    require(result.returncode == 0, result.stderr.strip() or f"git {' '.join(args)} failed")
    return result.stdout.strip()


def try_git(repo: str | Path, *args: str) -> str | None:
    try:
        return git(repo, *args)
    except TaskError:
        return None


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            json.dump(value, stream, indent=2)
            stream.write("\n")
            stream.flush()
            os.fsync(stream.fileno())
        except BaseException:
            temporary.unlink(missing_ok=True)
            raise
    try:
        os.replace(temporary, path)
    finally:
        temporary.unlink(missing_ok=True)


@contextlib.contextmanager
def state_lock(path: Path):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.with_suffix(".lock").open("a+b") as stream:
        if os.name == "nt":
            import msvcrt
            if stream.tell() == 0:
                stream.write(b"\0")
                stream.flush()
            stream.seek(0)
            try:
                msvcrt.locking(stream.fileno(), msvcrt.LK_NBLCK, 1)
            except OSError as error:
                raise TaskError(f"another roll is running: {error}") from error
            try:
                yield
            finally:
                stream.seek(0)
                msvcrt.locking(stream.fileno(), msvcrt.LK_UNLCK, 1)
        else:
            import fcntl
            try:
                fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except OSError as error:
                raise TaskError(f"another roll is running: {error}") from error
            try:
                yield
            finally:
                fcntl.flock(stream, fcntl.LOCK_UN)


class Context:
    def __init__(self, args):
        self.args = args
        self.source = args.source.resolve()
        self.workspace = self.source.parent.parent
        self.dry_run = args.dry_run
        self.spo_command = None

    def command(self, command, *, cwd=None, framework=None, extra_env=None):
        cwd = cwd or self.source
        command = [str(item) for item in command]
        print(f"+ {subprocess.list2cmdline(command)} (cwd: {cwd})", flush=True)
        if self.dry_run:
            return
        environment = os.environ.copy()
        environment["TUFFITE_FRAMEWORK_DIR"] = str(self.source)
        if framework is not None:
            environment.update(TUFFITE_FRAMEWORK_DIST=str(framework), TUFFITE_FRAMEWORK_DIST_AUTO_DOWNLOAD="0")
        environment.update(extra_env or {})
        subprocess.run(command, cwd=cwd, env=environment, check=True)

    def application(self, args, framework):
        self.command(["cargo", "run", "--package", "tuffite", "--bin", "tuff", "--", *args], framework=framework)

    def bridge(self, operation, target=None, **fields):
        # Build once per invocation; Cargo still performs freshness checks across invocations.
        if self.spo_command is None:
            override = os.environ.get("TUFFITE_SPO")
            if override:
                self.spo_command = [override]
            else:
                command = ["cargo", "build", "--manifest-path", str(self.source / "Cargo.toml"), "--package", "spo", "--bin", "spo", "--message-format=json"]
                result = subprocess.run(command, cwd=self.source, capture_output=True, text=True)
                if result.stderr:
                    print(result.stderr, end="", flush=True)
                require(result.returncode == 0, "cannot build spo; prepare tools with `mise run setup`")
                binaries = [item["executable"] for line in result.stdout.splitlines() if line.startswith("{") and (item := json.loads(line)).get("reason") == "compiler-artifact" and item.get("executable") and item["target"]["name"] == "spo"]
                require(binaries, "Cargo did not report the spo executable")
                self.spo_command = [binaries[-1]]
        with tempfile.TemporaryDirectory(prefix="tuffite-spo-") as directory:
            request = Path(directory) / "request.json"
            response = Path(directory) / "response.json"
            request.write_text(json.dumps({"operation": operation, "target": target, **fields}), encoding="utf-8")
            subprocess.run([*self.spo_command, "bridge", str(request), str(response)], cwd=self.source, check=True)
            return json.loads(response.read_text(encoding="utf-8"))

    def confirm(self, question):
        if self.args.yes:
            return
        require(not self.args.non_interactive and sys.stdin.isatty(), "confirmation required; pass --yes")
        require(input(f"{question} [y/N] ").strip().lower() in ("y", "yes"), "operation cancelled")
