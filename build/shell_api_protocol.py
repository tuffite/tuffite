#!/usr/bin/env python3
"""Generate typed C++ ShellAPI protocol sources for a GN action."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--framework-root", type=Path, required=True)
    parser.add_argument("--target-dir", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--depfile", type=Path, required=True)
    parser.add_argument("--depfile-target", type=Path, required=True)
    parser.add_argument("--definition", type=Path, action="append", required=True)
    args = parser.parse_args()

    framework_root = args.framework_root.resolve()
    target_dir = args.target_dir.resolve()
    manifest = args.manifest.resolve()
    output = args.output.resolve()
    configured = json.loads(manifest.read_text(encoding="utf-8"))
    manifest_definitions = [
        (manifest.parent / value).resolve()
        for value in configured["build"]["shellApi"]["definitions"]
    ]
    gn_definitions = [definition.resolve() for definition in args.definition]
    if manifest_definitions != gn_definitions:
        raise SystemExit(
            "ShellAPI definition inputs differ between the GN target and manifest:\n"
            f"  GN:       {gn_definitions}\n"
            f"  manifest: {manifest_definitions}"
        )
    output.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "cargo",
            "run",
            "--locked",
            "--manifest-path",
            str(framework_root / "Cargo.toml"),
            "--target-dir",
            str(target_dir),
            "--package",
            "tuffite",
            "--bin",
            "tuff",
            "--features",
            "cli",
            "--",
            "framework",
            "generate-cpp",
            str(manifest),
            "--output",
            str(output),
        ],
        check=True,
        cwd=framework_root,
    )

    cargo_dependencies = [framework_root / "Cargo.toml", framework_root / "Cargo.lock", framework_root / "framework/shell_api/scope_registry.json"]
    for crate in (framework_root / "crates").iterdir():
        manifest_path = crate / "Cargo.toml"
        if manifest_path.is_file():
            cargo_dependencies.append(manifest_path)
            cargo_dependencies.extend((crate / "src").rglob("*.rs"))

    def depfile_path(path: Path) -> str:
        return os.path.relpath(path.resolve(), Path.cwd()).replace("\\", "/").replace(" ", "\\ ")

    args.depfile.parent.mkdir(parents=True, exist_ok=True)
    dependencies = " ".join(depfile_path(path) for path in cargo_dependencies)
    args.depfile.write_text(
        f"{depfile_path(args.depfile_target)}: {dependencies}\n", encoding="utf-8"
    )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
