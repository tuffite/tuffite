"""Verify the DevTools master switch and document-origin denials in an isolated package."""

import argparse
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import time
import uuid
import urllib.request

from smoke import ROOT, free_port, grant_sandbox_access, terminate, wait_for_cdp, with_runtime_fixture


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--staging-dir", type=Path, required=True)
    args = parser.parse_args()
    source = args.package.resolve()
    run_root = args.staging_dir.resolve() / f"tuffite-devtools-{uuid.uuid4().hex}"
    run_root.mkdir(parents=True)
    grant_sandbox_access(run_root)
    print(f"run_root={run_root}", flush=True)
    for mode in ("disabled", "enabled", "root-denied", "compiled"):
        case = run_root / mode
        bundle = source.suffix == ".app"
        deployed = case / (source.name if bundle else "application")
        shutil.copytree(source, deployed, symlinks=True)
        resources = deployed / "Contents/Resources" if bundle else deployed
        code = deployed / "Contents/MacOS" if bundle else deployed
        manifest_path = resources / "tuffite.json"
        manifest = with_runtime_fixture(json.loads(manifest_path.read_text(encoding="utf-8")))
        window = manifest["app"]["windows"][0]
        manifest["app"]["windows"] = [window]
        manifest["browser"]["devtools"] = mode not in ("disabled", "compiled")
        # An explicit origin allow must not override the disabled master switch.
        manifest["browser"]["rules"].append(
            {"origins": ["playground://main"], "devtools": mode != "root-denied"}
        )
        manifest["browser"]["rules"].append(
            {"origins": ["playground://business"], "devtools": False}
        )
        manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
        if bundle:
            subprocess.run(
                ["codesign", "--force", "--sign", "-", str(deployed)], check=True
            )
        port = free_port()
        command = [
            str(code / ("playground.exe" if os.name == "nt" else "playground")),
            f"--user-data-dir={case / 'profile'}",
            f"--remote-debugging-port={port}",
            "--no-first-run",
            "--tuffite-runtime-snapshot",
            "--disable-background-timer-throttling",
        ]
        if mode == "compiled":
            command.remove("--tuffite-runtime-snapshot")
        with (
            (case / "stdout.log").open("wb") as stdout,
            (case / "stderr.log").open("wb") as stderr,
        ):
            process = subprocess.Popen(
                command,
                cwd=resources,
                stdout=stdout,
                stderr=stderr,
                start_new_session=os.name != "nt",
            )
            try:
                if mode == "disabled":
                    deadline = time.monotonic() + 8
                    while time.monotonic() < deadline:
                        if process.poll() is not None:
                            raise RuntimeError("Disabled candidate exited during startup")
                        with socket.socket() as probe:
                            probe.settimeout(0.25)
                            if probe.connect_ex(("127.0.0.1", port)) == 0:
                                raise RuntimeError(
                                    "browser.devtools=false opened a debugging port"
                                )
                        time.sleep(0.1)
                    print("devtools_master=ok explicit_port_blocked=true", flush=True)
                else:
                    wait_for_cdp(port, process, 30)
                    if mode == "compiled":
                        with urllib.request.urlopen(
                            f"http://127.0.0.1:{port}/json/version", timeout=5
                        ) as response:
                            assert json.load(response)["webSocketDebuggerUrl"]
                        print("devtools_compiled_configuration=ok Rust_setup_is_authoritative=true", flush=True)
                        continue
                    env = os.environ.copy()
                    env.update(
                        {
                            "TUFFITE_PLAYGROUND_CDP_PORT": str(port),
                            "TUFFITE_DEVTOOLS_ROOT_DENIED": (
                                "1" if mode == "root-denied" else "0"
                            ),
                            "TUFFITE_DEVTOOLS_ROOT_URL": window["url"],
                            "TUFFITE_DEVTOOLS_ALLOWED_URL": (
                                "playground://public/ui/dist/fixtures/behavior.html"
                            ),
                            "TUFFITE_DEVTOOLS_DENIED_URL": (
                                "playground://business/ui/dist/fixtures/behavior.html"
                            ),
                        }
                    )
                    subprocess.run(
                        ["bun", "run", "../scripts/devtools-origin-smoke.mjs"],
                        cwd=ROOT / "playground/ui",
                        env=env,
                        check=True,
                        timeout=90,
                    )
            finally:
                terminate(process)
    print(
        "devtools_policy=passed master switch, origin restrictions, navigation revocation",
        flush=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
