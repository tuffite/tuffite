"""Build, package, and validation workflows."""
import os
import platform
import re
from pathlib import Path

from . import framework, release, style
from .common import require


def configure_context(context):
    framework.ROOT = context.source
    framework.SOURCE = context.source.parent
    framework.WORKSPACE_ROOT = context.workspace


def resolve_profile_and_cpu(context, args):
    args.profile = args.profile or "release"
    if args.target_cpu is None and not context.args.non_interactive:
        output = framework.chromium_output(args) / "args.gn"
        if output.is_file():
            match = re.search(r'^\s*target_cpu\s*=\s*"(x64|arm64)"', output.read_text(encoding="utf-8"), re.MULTILINE)
            if match:
                args.target_cpu = match[1]
    args.target_cpu = args.target_cpu or ("arm64" if platform.machine().lower() in ("arm64", "aarch64") else "x64")


def defaults(context, args):
    resolve_profile_and_cpu(context, args)
    args.jobs = args.jobs or max(1, (os.cpu_count() or 1) * 2 // 3)
    print(f"Framework: {context.source}\nDistribution: local sources\nProfile: {args.profile}\nArchitecture: {args.target_cpu}\nJobs: {args.jobs}")


def native_action(context, args, action):
    if context.dry_run:
        output = framework.chromium_output(args)
        print(f"Configure GN {args.profile} / {args.target_cpu}: {output}")
        if action in ("build", "package") and not getattr(args, "skip_build", False):
            targets = args.targets or [framework.NINJA_TARGET]
            print(f"Ninja -C {output} -j {args.jobs} {' '.join(targets)}")
        if action == "package":
            print(f"Package Framework: {framework.distribution_path(args)}")
        elif action == "compdb":
            print(f"Compilation database: {framework.absolute(args.output)} (all: {args.all})")
        return
    getattr(framework, action)(args)


def web_assets(context, app):
    ui = app / "ui"
    if (ui / "package.json").is_file():
        context.command(["bun", "install", "--frozen-lockfile"], cwd=ui)
        context.command(["bun", "run", "build"], cwd=ui)


def package_framework(context, args):
    args.destination = args.destination or framework.default_distribution(args.profile, args.target_cpu)
    native_action(context, args, "package")
    return framework.absolute(args.destination)


def build(context, args):
    defaults(context, args)
    if args.explain:
        output = framework.chromium_output(args)
        require((output / "build.ninja").is_file(), f"missing Ninja graph: {output}; build this profile first")
        ninja = framework.SOURCE / "third_party" / "ninja" / framework.executable_name("ninja")
        context.command([ninja, "-C", output, "-d", "explain", "-n",
                         *(args.targets or [framework.NINJA_TARGET])], cwd=framework.SOURCE)
        return
    if args.framework or args.targets:
        native_action(context, args, "build")
        return
    distribution = package_framework(context, args)
    app = framework.absolute(args.app)
    context.application(["build", str(app / "tuffite.json"), "--release"], distribution)
    web_assets(context, app)
    if args.run:
        context.application(["dev", "--manifest", str(app / "tuffite.json"), "--release"], distribution)


def package(context, args):
    defaults(context, args)
    package_framework(context, args)
    if args.upload:
        release.upload(context, args)


def compdb(context, args):
    defaults(context, args)
    native_action(context, args, "compdb")


def test(context, args):
    if args.suite == "framework":
        style.run(context, check=True)
        distribution = context.source / "dist/tuffite-framework"
        context.application(["framework", "generate-js", "packages/tuffite/tuffite.json", "--output",
                             "packages/tuffite/generated", "--check"], distribution)
        context.application(["framework", "generate-rust", "packages/tuffite/tuffite.json",
                             "--output", "crates/tuffite/src/shell_api_generated.rs", "--check"], distribution)
        context.command(["node", "scripts/test-browser-runtime.mjs"])
        context.command(["node", "scripts/test-mojom-codec.mjs"])
        context.command(["node", "scripts/test-framework-mojom-codec.mjs"])
        context.command(["python3", "-m", "unittest", "discover", "-s", "scripts/tests", "-v"])
        context.command(["cargo", "fmt", "--all", "--", "--check"])
        context.command(["cargo", "test", "--workspace", "--exclude", "playground", "--all-features"])
        context.command(["cargo", "clippy", "--workspace", "--exclude", "playground", "--all-targets", "--all-features", "--", "-D", "warnings"])
    elif args.suite == "native":
        defaults(context, args)
        args.targets = ["tuffite/framework:unittests"]
        native_action(context, args, "build")
        output = framework.chromium_output(args)
        for name in ("dispatch_unittests", "manifest_service_unittests", "capability_pattern_unittests", "abi_host_bridge_unittests", "explorer_browser_support_unittests", "utility_service_unittests", "remoting_service_unittests"):
            binary = output / framework.executable_name(name)
            require(context.dry_run or binary.is_file(), f"missing native test: {binary}")
            context.command([binary], cwd=output)
    else:
        defaults(context, args)
        app = context.source / "playground"
        distribution = package_framework(context, args)
        context.application(["build", str(app / "tuffite.json"), "--release"], distribution)
        web_assets(context, app)
        context.application(["package", "--manifest", str(app / "tuffite.json"),
                             "--destination", str(context.source / "dist"),
                             "--name", "playground", "--force", "--no-strip"], distribution)
        command = ["python3", "playground/scripts/smoke.py", "--package", "dist/playground"]
        if args.suite == "explorer":
            command.append("--explorer-demo")
        if args.suite == "smoke":
            context.command([*command, "--mojom"])
            context.command([*command, "--explorer-capabilities"])
        context.command(command)
