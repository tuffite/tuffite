"""Public argparse entry; no terminal UI or third-party Python dependencies."""
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

from . import framework, release, roll, style, tasks
from .common import Context, TaskError, require

ROOT = Path(__file__).resolve().parents[1]


def positive(value):
    parsed = int(value)
    if parsed < 1:
        raise argparse.ArgumentTypeError("must be at least 1")
    return parsed


def global_options(parser, *, root=False):
    parser.add_argument("--source", type=Path, default=ROOT if root else argparse.SUPPRESS, help="Framework checkout (default: this checkout)")
    for flag, description in (("non-interactive", "never prompt; require explicit confirmation flags"), ("dry-run", "print the plan without changing files or running tasks"), ("yes", "confirm the requested operation")):
        parser.add_argument(f"--{flag}", action="store_true", default=False if root else argparse.SUPPRESS, help=description)


def distribution_options(parser):
    parser.add_argument("--profile", choices=("release", "component", "debug", "ci"), help="native profile (default: release)")
    parser.add_argument("--target-cpu", choices=("x64", "arm64"), help="architecture (default: existing output in interactive mode, otherwise host)")
    parser.set_defaults(chromium_out=None)


def native_options(parser):
    distribution_options(parser)
    parser.add_argument("-j", "--jobs", type=positive, help="Ninja jobs (default: two thirds of logical CPU cores)")
    parser.add_argument("--chromium-out", help="Chromium output under src/out (default: TuffiteRelease, TuffiteComponent, TuffiteExe (debug) or TuffiteCI for the profile)")
    parser.set_defaults(targets=[], destination=None, skip_build=False, include_symbols=False, strip=False, symbols_dir=None)


def distribution_argument(parser):
    parser.add_argument("--distribution", "--destination", dest="destination", type=Path, metavar="DIR",
                        help="Framework Distribution directory, relative to --source (default: dist/tuffite-framework[-PROFILE]-PLATFORM-CPU; --destination is an alias)")


def upload_options(parser):
    parser.add_argument("--repository", "--repo", metavar="OWNER/NAME", help=f"GitHub release repository (default: TUFFITE_GITHUB_REPOSITORY or {release.DEFAULT_REPOSITORY})")
    parser.add_argument("--tag", help="release tag (default: TUFFITE_FRAMEWORK_DIST_RELEASE_TAG or v<Framework version>)")
    parser.add_argument("--replace", action="store_true", help="replace a same-named release asset (default: reject duplicates)")
    parser.add_argument("--create", action="store_true", help="create the release if missing (default: require an existing release)")
    parser.add_argument("--prerelease", action="store_true", help="mark a newly created release as prerelease; requires --create (default: stable)")
    parser.add_argument("--no-progress", action="store_true", help="hide upload progress (default: enabled unless TUFFITE_FRAMEWORK_DIST_PROGRESS=0)")


def parser():
    result = argparse.ArgumentParser(prog="x.py", description="Tuffite Framework development", epilog="Examples: python3 x.py build --run; python3 x.py roll; cargo spo")
    global_options(result, root=True)
    commands = result.add_subparsers(dest="command", metavar="COMMAND")
    def command(name, description, native_args=False):
        child = commands.add_parser(name, help=description, description=description)
        global_options(child)
        if native_args:
            native_options(child)
        return child

    build = command("build", "Build local Framework and Playground", True)
    choice = build.add_mutually_exclusive_group()
    choice.add_argument("--framework", action="store_true", help="build native targets only")
    choice.add_argument("--run", action="store_true", help="launch Playground with Vite HMR")
    choice.add_argument("--explain", action="store_true", help="explain pending native work without compiling or packaging; Ninja may regenerate an outdated GN graph")
    build.add_argument("--target", dest="targets", nargs="+", default=[], help="specific Ninja targets")
    build.add_argument("--app", type=Path, default=Path("playground"), help="application directory relative to --source (default: playground)")

    package = command("package", "Package the local Framework Distribution", True)
    distribution_argument(package)
    package.add_argument("--skip-build", action="store_true", help="package existing native output (default: build incrementally first)")
    package.add_argument("--upload", action="store_true", help="upload the verified package after packaging (default: package only)")
    upload_options(package)
    package.add_argument("--include-symbols", action="store_true", help="legacy: include external symbols in the runtime")
    package.add_argument("--strip", action=argparse.BooleanOptionalAction, default=True, help="strip staged binaries (default: enabled)")
    package.set_defaults(strip=True)
    package.add_argument("--symbols-dir", type=Path, help="export matching private symbols to this directory (default: no separate export)")

    test = command("test", "Run Framework, native, or integration checks", True)
    test.add_argument("--suite", choices=("framework", "native", "smoke", "explorer"), default="framework", help="checks to run (default: framework)")

    formatting = command("format", "Format native C++ and GN sources with Chromium tools")
    formatting.add_argument("--check", action="store_true", help="check formatting without changing files")

    compdb = command("compdb", "Update clangd compilation database", True)
    compdb.add_argument("--all", action="store_true", help="include all Chromium sources")
    compdb.set_defaults(output="compile_commands.json")

    verify = command("verify", "Verify an existing Framework Distribution")
    verify.add_argument("distribution", type=Path, help="Framework Distribution directory relative to --source")

    upload = command("upload", "Upload a verified Distribution to GitHub Releases")
    distribution_options(upload)
    distribution_argument(upload)
    upload_options(upload)

    rolling = command("roll", "Sync dependencies and apply patches, preserving the Framework branch")
    rolling.add_argument("--chromium-revision", help="full Chromium commit SHA to write into DEPS")
    rolling.add_argument("--patches", choices=("checked-in", "local"), default="checked-in", help="patch source (default: checked-in; local preserves unexported commits)")
    rolling.add_argument("--dirty", choices=("reject", "stash"), default="reject", help="working changes policy (default: reject; stash saves and restores WIP)")
    rolling.add_argument("--skip-hooks", action="store_true", help="reuse an already prepared toolchain without running gclient hooks")
    recovery = rolling.add_mutually_exclusive_group()
    recovery.add_argument("--continue", dest="resume", action="store_true", help="continue the journaled roll")
    recovery.add_argument("--abort", action="store_true", help="restore stacks and declarations saved before the roll")
    rolling.add_argument("--wip-restored", action="store_true", help="confirm WIP conflicts were resolved; requires --continue")
    rolling.add_argument("--wip-saved", metavar="STASH_OID", help="recover an interrupted WIP save using its inspected stash object; requires --continue")

    hook = commands.add_parser("bootstrap")
    global_options(hook)
    hook.description = "Internal gclient first-checkout patch hook"

    return result


def main(argv=None):
    cli = parser()
    args = cli.parse_args(argv)
    if args.command is None:
        cli.print_help()
        return 0
    if args.command == "build" and args.run and args.targets:
        cli.error("--run cannot be combined with --target")
    if args.command == "roll" and (args.wip_restored or args.wip_saved) and not args.resume:
        cli.error("--wip-restored and --wip-saved require --continue")
    if args.command == "roll" and args.skip_hooks and args.abort:
        cli.error("--skip-hooks cannot be combined with --abort")
    if args.command in ("package", "upload") and args.prerelease and not args.create:
        cli.error("--prerelease requires --create")
    if args.command == "package" and not args.upload and any((args.repository, args.tag, args.replace, args.create, args.prerelease, args.no_progress)):
        cli.error("release options require package --upload")
    try:
        context = Context(args)
        require((context.source / "scripts/framework.py").is_file() and (context.source / "patches/config.json").is_file(), "--source must name the Framework checkout")
        tasks.configure_context(context)
        if args.command in ("build", "package", "test", "compdb"):
            getattr(tasks, args.command)(context, args)
        elif args.command == "format":
            style.run(context, check=args.check)
        elif args.command == "verify":
            if context.dry_run:
                print(f"Verify {framework.absolute(args.distribution)}")
            else:
                framework.verify_distribution(framework.absolute(args.distribution))
        elif args.command == "upload":
            tasks.resolve_profile_and_cpu(context, args)
            release.upload(context, args)
        elif args.command == "roll":
            roll.run(context, args)
        elif args.command == "bootstrap":
            roll.bootstrap(context)
        return 0
    except (KeyboardInterrupt, EOFError):
        print("error: operation cancelled; use roll --continue if a roll is active", file=sys.stderr)
        return 130
    except (OSError, ValueError, TaskError, subprocess.CalledProcessError, RuntimeError) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1
