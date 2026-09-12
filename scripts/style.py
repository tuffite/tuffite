"""Format native sources with the tools pinned by the Chromium checkout."""
import platform
import subprocess
from pathlib import Path

from . import framework
from .common import require

NATIVE_SUFFIXES = {".cc", ".h", ".mm"}
GN_SUFFIXES = {".gn", ".gni"}


def clang_format(source):
    chromium = source.parent
    system = platform.system()
    host = {"Darwin": "mac", "Windows": "win", "Linux": "linux64"}.get(system)
    require(host, f"unsupported formatter platform: {system}")
    executable = "clang-format.exe" if system == "Windows" else "clang-format"
    hosts = [host]
    if system == "Darwin" and platform.machine().lower() in ("arm64", "aarch64"):
        hosts.insert(0, "mac_arm64")
    candidates = [chromium / "third_party/clang-format" / executable]
    for name in hosts:
        candidates.extend([
            chromium / "buildtools" / f"{name}-format" / executable,
            chromium / "buildtools" / name / "format" / executable,
            chromium / "buildtools" / name / executable,
        ])
    for path in candidates:
        if path.is_file():
            return path
    raise RuntimeError("Chromium clang-format is missing; run `mise run setup`")


def run(context, *, check=False):
    if context.dry_run:
        print(f"{'Check' if check else 'Format'} native C++ and GN sources with Chromium tools")
        return
    result = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=context.source, capture_output=True, check=True,
    )
    paths = sorted({
        Path(name.decode("utf-8"))
        for name in result.stdout.split(b"\0") if name
        and Path(name.decode("utf-8")).suffix in NATIVE_SUFFIXES | GN_SUFFIXES
        and (context.source / Path(name.decode("utf-8"))).is_file()
    })
    cpp = [str(path) for path in paths if path.suffix in NATIVE_SUFFIXES]
    gn = [str(path) for path in paths if path.suffix in GN_SUFFIXES]
    formatter = clang_format(context.source)
    # Bound Windows command lengths without running a process per file.
    for start in range(0, len(cpp), 24):
        flags = ["--dry-run", "--Werror"] if check else ["-i"]
        context.command([formatter, *flags, *cpp[start:start + 24]])
    for start in range(0, len(gn), 24):
        flags = ["--dry-run"] if check else []
        context.command([framework.tool("gn"), "format", *flags, *gn[start:start + 24]])
    print(f"{'Checked' if check else 'Formatted'} {len(paths)} native source files")
