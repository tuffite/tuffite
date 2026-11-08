#!/usr/bin/env python3
"""Refresh the SDK's official Mojo JS runtime from the pinned Chromium build."""
import argparse
from pathlib import Path


def main():
    source = Path(__file__).resolve().parents[1]
    chromium = source.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--chromium-out', default='TuffiteRelease')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    generated = chromium / 'out' / args.chromium_out / 'gen/mojo/public/js/bindings.js'
    runtime = source / 'crates/tuffite/src/build/mojom'
    files = {
        runtime / 'chromium_mojo.mjs': (
            '// Chromium Mojo JavaScript bindings. Generated from the pinned Chromium checkout.\n'
            '// Copyright The Chromium Authors. BSD-3-Clause; see chromium_mojo.LICENSE.\n'
            + generated.read_text(encoding='utf-8')
        ),
        runtime / 'chromium_mojo.LICENSE': (chromium / 'LICENSE').read_text(encoding='utf-8'),
        source / 'packages/tuffite/CHROMIUM_LICENSE': (chromium / 'LICENSE').read_text(encoding='utf-8'),
    }
    for path, text in files.items():
        if args.check:
            if path.read_text(encoding='utf-8') != text:
                raise SystemExit(f'Mojo runtime drift: {path}; run {Path(__file__).name}')
        else:
            path.write_text(text, encoding='utf-8')


if __name__ == '__main__':
    main()
