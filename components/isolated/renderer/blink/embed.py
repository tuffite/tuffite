import pathlib
import sys
out = pathlib.Path(sys.argv[3])
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text("\n".join(
    'constexpr char kIsolated' + name + 'Script[] = R"ISOLATED(' +
    pathlib.Path(path).read_text() + ')ISOLATED";'
    for name, path in zip(("Host", "World"), sys.argv[1:3])
))
