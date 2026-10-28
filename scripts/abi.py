#!/usr/bin/env python3
"""Generate native ABI version definitions from the ABI crate's Cargo version."""

import argparse
from pathlib import Path
import re
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def abi_version():
    with (ROOT / "crates/tuffite-abi/Cargo.toml").open("rb") as source:
        version = tomllib.load(source)["package"]["version"]
    if isinstance(version, dict) and version.get("workspace") is True:
        with (ROOT / "Cargo.toml").open("rb") as source:
            version = tomllib.load(source)["workspace"]["package"]["version"]
    match = re.fullmatch(r"(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", version)
    if not match or any(int(value) > 65535 for value in match.groups()):
        raise ValueError(f"Invalid ABI crate version: {version}")
    return tuple(int(value) for value in match.groups())


def header(version=None):
    major, minor, patch = version or abi_version()
    return (
        "// Generated from crates/tuffite-abi/Cargo.toml; do not edit.\n"
        "#ifndef TUFFITE_ABI_VERSION_H_\n#define TUFFITE_ABI_VERSION_H_\n"
        f"#define TUFFITE_FFI_ABI_MAJOR {major}\n"
        f"#define TUFFITE_FFI_ABI_MINOR {minor}\n"
        f"#define TUFFITE_FFI_ABI_PATCH {patch}\n"
        "#endif\n"
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    text = header()
    if args.check:
        if not args.output.is_file() or args.output.read_text() != text:
            parser.exit(1, f"stale ABI version header: {args.output}\n")
        parser.exit()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    if not args.output.exists() or args.output.read_text() != text:
        args.output.write_text(text, encoding="utf-8", newline="\n")
