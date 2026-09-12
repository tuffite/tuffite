#!/usr/bin/env python3
import pathlib
import sys


def main() -> None:
    source, output = map(pathlib.Path, sys.argv[1:])
    contents = (
        'constexpr char kShellApiScopeRegistry[] = R"scope('
        + source.read_text(encoding="utf-8")
        + ')scope";\n'
    ).encode("utf-8")
    # GN actions use restat: unchanged output mtimes avoid downstream work.
    if output.is_file() and output.read_bytes() == contents:
        return
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(contents)


if __name__ == "__main__":
    main()
