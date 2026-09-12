#!/usr/bin/env python3
"""Native GN/Ninja build and Distribution packaging implementation."""

from __future__ import annotations

import argparse
import ctypes
import hashlib
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path, PurePosixPath

import tomllib

from .abi import abi_version
from . import symbols

if os.name == "nt":
    from ctypes import wintypes

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT.parent
WORKSPACE_ROOT = SOURCE.parent
FRAMEWORK_TARGET = "//tuffite/framework:distribution"
NINJA_TARGET = "tuffite/framework:distribution"
FRAMEWORK_MANIFEST = "tuffite-framework.json"
LICENSE_FILES = (
    "licenses/LICENSE.tuffite.txt",
    "licenses/LICENSE.chromium.txt",
    "licenses/THIRD_PARTY_NOTICES.chromium.txt",
)
FFI_ABI_MAJOR, FFI_ABI_MINOR, FFI_ABI_PATCH = abi_version()
FEATURES = ("shell_api", "explorer", "remoting")
WINDOWS_COMPONENT_LIBRARIES = (
    "tuffite.dll.lib",
    "base.dll.lib",
    "content.dll.lib",
    "third_party_abseil-cpp_absl.dll.lib",
    "third_party_perfetto_libperfetto.dll.lib",
    "base_allocator_partition_allocator_src_partition_alloc_raw_ptr.dll.lib",
    "base_allocator_partition_allocator_src_partition_alloc_allocator_base.dll.lib",
    "base_allocator_partition_allocator_src_partition_alloc_allocator_core.dll.lib",
    "base_allocator_partition_allocator_src_partition_alloc_allocator_shim.dll.lib",
    "libc++.dll.lib",
)


def absolute(path: str | Path) -> Path:
    value = Path(path)
    return (value if value.is_absolute() else ROOT / value).resolve()


def default_output(profile: str) -> Path:
    name = {
        "debug": "TuffiteExe",
        "release": "TuffiteRelease",
        "component": "TuffiteComponent",
        "ci": "TuffiteCI",
    }[profile]
    return SOURCE / "out" / name


def platform_id() -> str:
    return {"win32": "windows", "darwin": "macos"}.get(sys.platform, "linux")


def rust_target(target_cpu: str) -> str:
    architecture = {"x64": "x86_64", "arm64": "aarch64"}.get(
        target_cpu, target_cpu
    )
    suffix = {
        "windows": "pc-windows-msvc",
        "macos": "apple-darwin",
        "linux": "unknown-linux-gnu",
    }[platform_id()]
    return f"{architecture}-{suffix}"


def executable_name(name: str) -> str:
    return f"{name}.exe" if os.name == "nt" else name


def display_command(command: list[str]) -> str:
    return subprocess.list2cmdline(command) if os.name == "nt" else " ".join(command)


def positive_int(value: str) -> int:
    parsed = int(value)
    if parsed < 1:
        raise argparse.ArgumentTypeError("must be at least 1")
    return parsed


class WindowsJob:
    """Kill the complete child tree if the orchestrator exits."""

    def __init__(self) -> None:
        self.handle: int | None = None
        if os.name != "nt":
            return

        class BasicLimitInformation(ctypes.Structure):
            _fields_ = [
                ("PerProcessUserTimeLimit", ctypes.c_longlong),
                ("PerJobUserTimeLimit", ctypes.c_longlong),
                ("LimitFlags", wintypes.DWORD),
                ("MinimumWorkingSetSize", ctypes.c_size_t),
                ("MaximumWorkingSetSize", ctypes.c_size_t),
                ("ActiveProcessLimit", wintypes.DWORD),
                ("Affinity", ctypes.c_size_t),
                ("PriorityClass", wintypes.DWORD),
                ("SchedulingClass", wintypes.DWORD),
            ]

        class IoCounters(ctypes.Structure):
            _fields_ = [
                (name, ctypes.c_ulonglong)
                for name in (
                    "ReadOperationCount",
                    "WriteOperationCount",
                    "OtherOperationCount",
                    "ReadTransferCount",
                    "WriteTransferCount",
                    "OtherTransferCount",
                )
            ]

        class ExtendedLimitInformation(ctypes.Structure):
            _fields_ = [
                ("BasicLimitInformation", BasicLimitInformation),
                ("IoInfo", IoCounters),
                ("ProcessMemoryLimit", ctypes.c_size_t),
                ("JobMemoryLimit", ctypes.c_size_t),
                ("PeakProcessMemoryUsed", ctypes.c_size_t),
                ("PeakJobMemoryUsed", ctypes.c_size_t),
            ]

        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel32.CreateJobObjectW.argtypes = (ctypes.c_void_p, wintypes.LPCWSTR)
        kernel32.CreateJobObjectW.restype = wintypes.HANDLE
        kernel32.SetInformationJobObject.argtypes = (
            wintypes.HANDLE,
            ctypes.c_int,
            ctypes.c_void_p,
            wintypes.DWORD,
        )
        kernel32.SetInformationJobObject.restype = wintypes.BOOL
        kernel32.CloseHandle.argtypes = (wintypes.HANDLE,)
        kernel32.CloseHandle.restype = wintypes.BOOL
        handle = kernel32.CreateJobObjectW(None, None)
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        information = ExtendedLimitInformation()
        information.BasicLimitInformation.LimitFlags = 0x00002000
        if not kernel32.SetInformationJobObject(
            handle, 9, ctypes.byref(information), ctypes.sizeof(information)
        ):
            kernel32.CloseHandle(handle)
            raise ctypes.WinError(ctypes.get_last_error())
        self.handle = handle

    def assign(self, process: subprocess.Popen[bytes]) -> None:
        if self.handle is None:
            return
        kernel32 = ctypes.WinDLL("kernel32", use_last_error=True)
        kernel32.AssignProcessToJobObject.argtypes = (wintypes.HANDLE, wintypes.HANDLE)
        kernel32.AssignProcessToJobObject.restype = wintypes.BOOL
        if not kernel32.AssignProcessToJobObject(self.handle, int(process._handle)):
            raise ctypes.WinError(ctypes.get_last_error())

    def close(self) -> None:
        if self.handle is not None:
            ctypes.WinDLL("kernel32", use_last_error=True).CloseHandle(self.handle)
            self.handle = None


def terminate_process(process: subprocess.Popen[bytes]) -> None:
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


def execute(
    command: list[str], *, cwd: Path = ROOT, capture: bool = False,
    extra_env: dict[str, str] | None = None,
) -> subprocess.CompletedProcess[str] | None:
    print(f"+ {display_command(command)}", flush=True)
    creationflags = subprocess.CREATE_NEW_PROCESS_GROUP if os.name == "nt" else 0
    job = WindowsJob()
    process = subprocess.Popen(
        command,
        cwd=cwd,
        env={**os.environ, **extra_env} if extra_env else None,
        creationflags=creationflags,
        start_new_session=os.name != "nt",
        stdout=subprocess.PIPE if capture else None,
        stderr=subprocess.PIPE if capture else None,
        text=capture,
    )
    try:
        try:
            job.assign(process)
        except OSError:
            terminate_process(process)
            raise
        stdout, stderr = process.communicate()
    except KeyboardInterrupt:
        terminate_process(process)
        raise
    finally:
        job.close()
    if process.returncode:
        if capture and stderr:
            print(stderr.rstrip(), file=sys.stderr)
        raise subprocess.CalledProcessError(process.returncode, command)
    if capture:
        return subprocess.CompletedProcess(command, process.returncode, stdout, stderr)
    return None


def tool(name: str) -> str:
    suffix = ".bat" if os.name == "nt" else ""
    local = WORKSPACE_ROOT / "depot_tools" / f"{name}{suffix}"
    if local.is_file():
        return str(local)
    resolved = shutil.which(name)
    if resolved is None:
        raise RuntimeError(
            f"{name} was not found; run `mise install` and `mise run doctor`"
        )
    return resolved


def selected_profile(args: argparse.Namespace) -> str:
    if getattr(args, "component", False):
        if args.profile != "release":
            raise RuntimeError(
                "--component cannot be combined with an explicit non-release profile"
            )
        return "component"
    return args.profile


def chromium_output(args: argparse.Namespace) -> Path:
    output = (
        absolute(args.chromium_out)
        if args.chromium_out
        else default_output(selected_profile(args))
    )
    try:
        output.relative_to(SOURCE / "out")
    except ValueError as error:
        raise RuntimeError(
            f"Chromium output must be under {SOURCE / 'out'}: {output}"
        ) from error
    return output


def default_distribution(profile: str, target_cpu: str) -> Path:
    suffix = "" if profile == "release" else f"-{profile}"
    return (
        ROOT / "dist" / f"tuffite-framework{suffix}-{platform_id()}-{target_cpu}"
    )


def distribution_path(args: argparse.Namespace) -> Path:
    return (
        absolute(args.destination)
        if args.destination
        else default_distribution(selected_profile(args), args.target_cpu)
    )


def configure(args: argparse.Namespace) -> None:
    profile = selected_profile(args)
    output = chromium_output(args)
    preset = ROOT / "build"/ "args" / f"{profile}.gn"
    output.mkdir(parents=True, exist_ok=True)
    args_file = output / "args.gn"
    args_contents = "\n".join(
        line
        for line in preset.read_text(encoding="utf-8").splitlines()
        if not line.strip().startswith("target_cpu =")
    )
    # Keep GN, Cargo, output naming, and Distribution metadata on one explicit
    # target identity instead of deriving artifacts from the host machine.
    args_contents += f'\ntarget_cpu = "{args.target_cpu}"\n'
    changed = (
        not args_file.is_file()
        or args_file.read_text(encoding="utf-8") != args_contents
    )
    if changed:
        args_file.write_text(args_contents, encoding="utf-8")
    if changed or not (output / "build.ninja").is_file():
        print(f"Configuring Chromium {profile} graph at {output}")
        execute([tool("gn"), "gen", str(output), "--check"], cwd=SOURCE)
    else:
        print(f"Reusing Chromium {profile} graph at {output}")


def compiler_cache_environment() -> dict[str, str]:
    if os.name != "nt":
        return {}
    # Cached /showIncludes output can contain absolute compiler-header paths.
    # A moved checkout must not replay the old paths into Ninja's dependency DB.
    identity = str(SOURCE.resolve()).casefold()
    custom = os.environ.get("SCCACHE_C_CUSTOM_CACHE_BUSTER", "")
    digest = hashlib.sha256(f"{identity}\0{custom}".encode("utf-8")).hexdigest()
    return {"SCCACHE_C_CUSTOM_CACHE_BUSTER": f"tuffite-checkout-v1-{digest}"}


def build(args: argparse.Namespace) -> None:
    configure(args)
    targets = args.targets or [NINJA_TARGET]
    ninja = SOURCE / "third_party" / "ninja" / executable_name("ninja")
    command = [str(ninja), "-C", str(chromium_output(args)), "-j", str(args.jobs)]
    command.extend(targets)
    cache_environment = compiler_cache_environment()
    if cache_environment:
        print(f"Compiler cache checkout: {SOURCE.resolve()}")
        print(f"Compiler cache identity: {cache_environment['SCCACHE_C_CUSTOM_CACHE_BUSTER']}")
    execute(command, cwd=SOURCE, extra_env=cache_environment)


def compdb(args: argparse.Namespace) -> None:
    configure(args)
    output = chromium_output(args)
    destination = absolute(args.output)
    destination.parent.mkdir(parents=True, exist_ok=True)
    temporary = tempfile.NamedTemporaryFile(
        prefix=f".{destination.name}.",
        suffix=".tmp",
        dir=destination.parent,
        delete=False,
    )
    generated = Path(temporary.name)
    temporary.close()
    try:
        execute(
            [
                sys.executable,
                "tools/clang/scripts/generate_compdb.py",
                "-p",
                str(output.relative_to(SOURCE)),
                "-o",
                str(generated),
                *(args.targets or [NINJA_TARGET]),
            ],
            cwd=SOURCE,
        )
        commands = json.loads(generated.read_text(encoding="utf-8"))
        if not args.all:
            tuffite_root = (SOURCE / "tuffite").resolve()
            commands = [
                entry
                for entry in commands
                if Path(entry["directory"])
                .joinpath(entry["file"])
                .resolve()
                .is_relative_to(tuffite_root)
            ]
            if not commands:
                raise RuntimeError(
                    "generated compilation database contains no Tuffite sources"
                )
        destination.write_text(json.dumps(commands, indent=2) + "\n", encoding="utf-8")
    finally:
        generated.unlink(missing_ok=True)
    print(f"compile_commands={destination}")
    print(f"compile_commands_entries={len(commands)}")


def read_component_build(output: Path) -> bool:
    args_file = output / "args.gn"
    return any(
        line.strip() == "is_component_build = true"
        for line in args_file.read_text(encoding="utf-8").splitlines()
    )


def read_development_enabled(output: Path) -> bool:
    args_file = output / "args.gn"
    return any(
        line.strip() == "tuffite_dev_mode = true"
        for line in args_file.read_text(encoding="utf-8").splitlines()
    )


def gn_runtime_dependencies(output: Path) -> list[Path]:
    relative_output = output.relative_to(SOURCE)
    result = execute(
        [tool("gn"), "desc", str(relative_output), FRAMEWORK_TARGET, "runtime_deps"],
        cwd=SOURCE,
        capture=True,
    )
    assert result is not None
    return [Path(line.strip()) for line in result.stdout.splitlines() if line.strip()]


def portable_path(path: Path) -> str:
    return "/".join(path.parts)


def is_packageable(path: Path, include_symbols: bool) -> bool:
    portable = portable_path(path)
    if not include_symbols and (path.suffix.lower() in {".pdb", ".lib", ".exp", ".ilk", ".debug", ".dwp"} or any(part.endswith(".dSYM") for part in path.parts)):
        return False
    if (
        portable.startswith("gen/third_party/devtools-frontend/")
        or portable.startswith("test_fonts/")
        or portable.startswith("pyproto/")
        or portable.startswith("angledata/validation/")
        or portable.endswith(".pak.info")
    ):
        return False
    return path.name not in {
        "test_trace_processor.dll",
        "VkICD_mock_icd.dll",
        "VkICD_mock_icd.json",
        "VkLayer_khronos_validation.dll",
        "VkLayer_khronos_validation.json",
        "libVkICD_mock_icd.dylib",
        "libVkICD_mock_icd.so",
        "libVkLayer_khronos_validation.dylib",
        "libVkLayer_khronos_validation.so",
    }


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def copy_file(source: Path, destination: Path) -> None:
    if not source.is_file():
        raise RuntimeError(f"Required Framework source is missing: {source}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def link_files(component_build: bool, target: str | None = None) -> tuple[str, ...]:
    windows = "windows" in target if target else os.name == "nt"
    macos = "apple-darwin" in target if target else sys.platform == "darwin"
    if windows:
        return (
            WINDOWS_COMPONENT_LIBRARIES
            if component_build
            else ("tuffite.dll.lib",)
        )
    runtime = (
        "libtuffite.dylib"
        if macos
        else "libtuffite.so"
    )
    return (runtime,)


def framework_version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as source:
        return str(tomllib.load(source)["workspace"]["package"]["version"])


def chromium_revision() -> str:
    for relative in ("build/util/LASTCHANGE", "build/util/LASTCHANGE.committime"):
        path = SOURCE / relative
        if not path.is_file():
            continue
        for line in path.read_text(encoding="utf-8").splitlines():
            value = line.removeprefix("LASTCHANGE=").strip()
            if value:
                return value
    raise RuntimeError("Unable to determine the Chromium revision")


def remove_tree(path: Path) -> None:
    for attempt in range(20):
        try:
            shutil.rmtree(path)
            return
        except OSError:
            if attempt == 19:
                raise
            time.sleep(0.1)


def install_directories(pairs):
    """Publish verified runtime/symbol copies together, retaining old outputs."""
    backups = []
    installed = []
    try:
        for staging, destination in pairs:
            backup = destination.with_name(f".{destination.name}.backup-{os.getpid()}")
            if backup.exists() or backup.is_symlink():
                raise RuntimeError(f"Existing package backup requires review: {backup}")
            if destination.is_symlink() or (destination.exists() and not destination.is_dir()):
                raise RuntimeError(f"Package destination is not a regular directory: {destination}")
            if destination.exists():
                destination.replace(backup)
                backups.append((backup, destination))
        for staging, destination in pairs:
            for attempt in range(30):
                try:
                    staging.replace(destination)
                    installed.append(destination)
                    break
                except OSError:
                    if attempt == 29:
                        raise
                    time.sleep(0.2)
    except BaseException:
        for destination in reversed(installed):
            remove_tree(destination)
        for backup, destination in reversed(backups):
            backup.replace(destination)
        raise
    for backup, _ in backups:
        remove_tree(backup)


def install_directory(staging: Path, destination: Path) -> None:
    install_directories([(staging, destination)])


def package_notices(output: Path, runtime_root: Path) -> None:
    """Generate notices from the selected Chromium graph, not a hand-written list."""
    licenses = runtime_root / "licenses"
    licenses.mkdir(parents=True, exist_ok=True)
    copy_file(ROOT / "LICENSE", licenses / "LICENSE.tuffite.txt")
    copy_file(SOURCE / "LICENSE", licenses / "LICENSE.chromium.txt")
    execute(
        [sys.executable, str(SOURCE / "tools/licenses/licenses.py"), "license_file",
         "--format", "txt", "--scan-root", str(SOURCE),
         "--gn-out-dir", str(output), "--gn-target", FRAMEWORK_TARGET,
         str(licenses / "THIRD_PARTY_NOTICES.chromium.txt")],
        cwd=SOURCE,
    )
    verify_notices(runtime_root)


def verify_notices(runtime_root: Path) -> None:
    for relative in LICENSE_FILES:
        path = runtime_root / relative
        if not path.is_file() or path.is_symlink() or not path.read_bytes().strip():
            raise RuntimeError(
                f"Missing or empty Framework notice: {path}; repackage the Framework"
            )


def package(args: argparse.Namespace) -> None:
    if not args.skip_build:
        build(args)
    output = chromium_output(args)
    destination = distribution_path(args)
    distribution_root = (ROOT / "dist").resolve()
    try:
        destination.relative_to(distribution_root)
    except ValueError as error:
        raise RuntimeError(
            f"Framework distributions must be under {distribution_root}: {destination}"
        ) from error
    if destination == distribution_root:
        raise RuntimeError(f"Refusing unsafe distribution destination: {destination}")
    destination.parent.mkdir(parents=True, exist_ok=True)
    staging = Path(
        tempfile.mkdtemp(prefix=f".{destination.name}-", dir=destination.parent)
    )
    component_build = read_component_build(output)
    development_enabled = read_development_enabled(output)
    try:
        link_root = staging / "lib"
        runtime_root = staging / "runtime"
        imports = link_files(component_build)
        for name in imports:
            copy_file(output / name, link_root / name)

        copied: set[str] = set()
        bootstrap_name = executable_name("tuffite_launcher")
        for dependency in gn_runtime_dependencies(output):
            if not dependency.is_absolute() and ".." in dependency.parts:
                continue
            source = dependency if dependency.is_absolute() else output / dependency
            if not source.is_file():
                continue
            source = source.resolve()
            try:
                relative = source.relative_to(output)
            except ValueError as error:
                raise RuntimeError(
                    f"Runtime dependency escaped Chromium output: {source}"
                ) from error
            portable = portable_path(relative)
            if relative == Path(bootstrap_name):
                continue
            if portable in copied or not is_packageable(relative, args.include_symbols):
                continue
            copy_file(source, runtime_root / relative)
            copied.add(portable)
        if not copied:
            raise RuntimeError("GN returned no packageable runtime dependencies")
        web_runtime_sources = {
            "tuffite-runtime.js": ROOT / "packages" / "tuffite" / "runtime.js",
            "tuffite-shell-api.js": ROOT
            / "packages"
            / "tuffite"
            / "generated"
            / "index.js",
        }
        for name, source in web_runtime_sources.items():
            copy_file(source, runtime_root / name)
            copied.add(name)

        package_notices(output, runtime_root)
        bootstrap_source = output / bootstrap_name
        if not bootstrap_source.is_file():
            raise RuntimeError(f"Missing Tuffite launcher: {bootstrap_source}")
        packaged_bootstrap_name = executable_name("tuffite_launcher")
        bootstrap_target = staging / packaged_bootstrap_name
        copy_file(bootstrap_source, bootstrap_target)
        symbols_destination = absolute(args.symbols_dir) if getattr(args, "symbols_dir", None) else None
        if symbols_destination and (not symbols_destination.is_relative_to(distribution_root) or symbols_destination == distribution_root):
            raise RuntimeError(f"Symbols must be under {distribution_root}")
        if symbols_destination and (symbols_destination == destination or symbols_destination.is_relative_to(destination) or destination.is_relative_to(symbols_destination)):
            raise RuntimeError("Symbols and runtime destinations must not overlap")
        symbols_staging = staging.parent / (staging.name + ".symbols") if symbols_destination else None
        symbols.process(staging, strip=getattr(args, "strip", True), symbols=symbols_staging, source_root=output, metadata={"framework_version": framework_version(), "chromium_revision": chromium_revision(), "target": rust_target(args.target_cpu), "profile": selected_profile(args)})
        runtime_files = []
        for path in sorted(item for item in runtime_root.rglob("*") if item.is_file()):
            relative = path.relative_to(runtime_root)
            runtime_files.append(
                {
                    "path": portable_path(relative),
                    "size": path.stat().st_size,
                    "sha256": sha256(path),
                }
            )
        manifest = {
            "framework_version": framework_version(),
            "chromium_revision": chromium_revision(),
            "target": rust_target(args.target_cpu),
            "profile": selected_profile(args),
            "variant": "desktop",
            "component_build": component_build,
            "development_enabled": development_enabled,
            "features": list(FEATURES),
            "ffi_abi_major": FFI_ABI_MAJOR,
            "ffi_abi_minor": FFI_ABI_MINOR,
            "ffi_abi_patch": FFI_ABI_PATCH,
            "link_directory": "lib",
            "runtime_directory": "runtime",
            "bootstrap": {
                "path": packaged_bootstrap_name,
                "size": bootstrap_target.stat().st_size,
                "sha256": sha256(bootstrap_target),
            },
            "runtime_files": runtime_files,
        }
        (staging / FRAMEWORK_MANIFEST).write_text(
            json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
        )
        verify_distribution(staging)
        pairs = [(staging, destination)]
        if symbols_staging:
            symbols_destination.parent.mkdir(parents=True, exist_ok=True)
            pairs.append((symbols_staging, symbols_destination))
        install_directories(pairs)
        if symbols_staging:
            print(f"symbols={symbols_destination}")
    except BaseException:
        if staging.exists():
            remove_tree(staging)
        if "symbols_staging" in locals() and symbols_staging and symbols_staging.exists():
            remove_tree(symbols_staging)
        raise
    print(f"framework={destination}")


def safe_child(root: Path, value: object, field: str) -> Path:
    if not isinstance(value, str) or not value or "\\" in value:
        raise RuntimeError(f"Invalid Framework {field}")
    relative = PurePosixPath(value)
    if relative.is_absolute() or any(
        part in {"", ".", ".."} for part in relative.parts
    ):
        raise RuntimeError(f"Unsafe Framework {field}: {value}")
    result = root.joinpath(*relative.parts).resolve()
    try:
        result.relative_to(root)
    except ValueError as error:
        raise RuntimeError(f"Unsafe Framework {field}: {value}") from error
    return result


def binary_architecture(path: Path) -> str:
    with path.open("rb") as stream:
        header = stream.read(64)
        if header[:2] == b"MZ":
            stream.seek(int.from_bytes(header[60:64], "little") + 4)
            machine = int.from_bytes(stream.read(2), "little")
            return {0x8664: "x64", 0xAA64: "arm64"}.get(machine, f"pe-{machine:#x}")
        if header[:4] == b"\x7fELF":
            endian = "little" if header[5] == 1 else "big"
            machine = int.from_bytes(header[18:20], endian)
            return {62: "x64", 183: "arm64"}.get(machine, f"elf-{machine}")
        magic = int.from_bytes(header[:4], "big")
        if magic in {0xFEEDFACF, 0xCFFAEDFE}:
            endian = "big" if magic == 0xFEEDFACF else "little"
            cpu = int.from_bytes(header[4:8], endian)
            return {0x01000007: "x64", 0x0100000C: "arm64"}.get(
                cpu, f"macho-{cpu:#x}"
            )
    raise RuntimeError(f"Unsupported native binary format: {path}")


def verify_distribution(root: Path) -> None:
    root = root.resolve()
    manifest_path = root / FRAMEWORK_MANIFEST
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    expected = {
        "framework_version": framework_version(),
        "ffi_abi_major": FFI_ABI_MAJOR,
    }
    for field, value in expected.items():
        if manifest.get(field) != value:
            raise RuntimeError(
                f"Framework {field} is {manifest.get(field)!r}; expected {value!r}"
            )
    target = manifest.get("target")
    supported_targets = {
        f"{cpu}-{platform}"
        for cpu in ("x86_64", "aarch64")
        for platform in ("apple-darwin", "unknown-linux-gnu", "pc-windows-msvc")
    }
    if target not in supported_targets:
        raise RuntimeError(
            f"Framework target is {target!r}; expected one of {sorted(supported_targets)!r}"
        )
    if manifest.get("variant") not in {"full", "desktop"}:
        raise RuntimeError("Unsupported Framework variant")
    if (
        type(manifest.get("ffi_abi_minor")) is not int
        or not 0 <= manifest["ffi_abi_minor"] <= 65535
    ):
        raise RuntimeError("Incompatible Framework FFI ABI minor version")
    patch = manifest.get("ffi_abi_patch")
    if type(patch) is not int or not 0 <= patch <= 65535:
        raise RuntimeError("Invalid Framework FFI ABI patch version")
    if (manifest["ffi_abi_minor"], patch) < (FFI_ABI_MINOR, FFI_ABI_PATCH):
        raise RuntimeError("Framework FFI ABI is older than the required semver version")
    link_root = safe_child(root, manifest.get("link_directory"), "link_directory")
    runtime_root = safe_child(
        root, manifest.get("runtime_directory"), "runtime_directory"
    )
    verify_notices(runtime_root)
    bootstrap_data = manifest.get("bootstrap")
    if not isinstance(bootstrap_data, dict):
        raise RuntimeError("Invalid Framework bootstrap entry")
    bootstrap = safe_child(root, bootstrap_data.get("path"), "bootstrap")
    bootstrap_digest = bootstrap_data.get("sha256")
    if (
        not bootstrap.is_file()
        or bootstrap.stat().st_size != bootstrap_data.get("size")
        or not isinstance(bootstrap_digest, str)
        or len(bootstrap_digest) != 64
        or sha256(bootstrap) != bootstrap_digest.lower()
    ):
        raise RuntimeError(f"Framework bootstrap mismatch: {bootstrap}")
    expected_architecture = "arm64" if target.startswith("aarch64-") else "x64"
    actual_architecture = binary_architecture(bootstrap)
    if actual_architecture != expected_architecture:
        raise RuntimeError(
            f"Framework bootstrap architecture is {actual_architecture}; "
            f"manifest target {target} requires {expected_architecture}"
        )
    component_build = manifest.get("component_build")
    if not isinstance(component_build, bool):
        raise RuntimeError("Invalid Framework component_build")
    if not isinstance(manifest.get("development_enabled"), bool):
        raise RuntimeError("Invalid Framework development_enabled")
    imports = link_files(component_build, target)
    for name in imports:
        if not (link_root / name).is_file():
            raise RuntimeError(f"Missing Framework link input: {link_root / name}")

    entries = manifest.get("runtime_files")
    if not isinstance(entries, list) or not entries:
        raise RuntimeError("Framework runtime file list is empty")
    declared: set[str] = set()
    total = 0
    for entry_data in entries:
        if not isinstance(entry_data, dict):
            raise RuntimeError("Invalid Framework runtime file entry")
        path = safe_child(runtime_root, entry_data.get("path"), "runtime file")
        portable = portable_path(path.relative_to(runtime_root))
        if portable in declared:
            raise RuntimeError(f"Duplicate Framework runtime file: {portable}")
        declared.add(portable)
        if not path.is_file() or path.stat().st_size != entry_data.get("size"):
            raise RuntimeError(f"Framework runtime file size mismatch: {path}")
        digest = entry_data.get("sha256")
        if (
            not isinstance(digest, str)
            or len(digest) != 64
            or sha256(path) != digest.lower()
        ):
            raise RuntimeError(f"Framework runtime file hash mismatch: {path}")
        total += path.stat().st_size
    actual = {
        portable_path(path.relative_to(runtime_root))
        for path in runtime_root.rglob("*")
        if path.is_file()
    }
    if actual != declared:
        difference = sorted(actual ^ declared)
        raise RuntimeError(f"Framework runtime manifest mismatch: {difference[:5]}")
    print(f"verified={root}")
    print(f"runtime_files={len(entries)}")
    print(f"runtime_bytes={total}")

