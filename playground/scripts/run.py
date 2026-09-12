#!/usr/bin/env python3
"""Launch the packaged Playground for the current host."""

import os
import platform
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SYSTEM = {"win32": "windows", "darwin": "macos"}.get(sys.platform, "linux")
MACHINE = platform.machine().lower()
ARCHITECTURE = {"amd64": "x64", "x86_64": "x64", "aarch64": "arm64"}.get(
    MACHINE, MACHINE
)
EXECUTABLE = "playground.exe" if os.name == "nt" else "playground"
APPLICATION = ROOT / "dist" / f"tuffite-playground-{SYSTEM}-{ARCHITECTURE}" / EXECUTABLE

if not APPLICATION.is_file():
    raise SystemExit(f"packaged Playground is missing: {APPLICATION}")
raise SystemExit(subprocess.call([str(APPLICATION)], cwd=APPLICATION.parent))
