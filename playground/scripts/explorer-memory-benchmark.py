#!/usr/bin/env python3
"""Compare Explorer allocation and lifecycle using macOS process footprints.

Requires a packaged Playground, Bun, and the same build's optional per-instance
control dylib. Only the disposable package is modified. The driver never attaches
DevTools to Explorer documents or changes production process/partition policy.
"""

from __future__ import annotations

import argparse
import ctypes
import datetime
import hashlib
import http.server
import json
import os
from pathlib import Path
import platform
import re
import shutil
import statistics
import subprocess
import sys
import tempfile
import threading
import time

from smoke import free_port, terminate, wait_for_cdp, with_runtime_fixture


class Rusage(ctypes.Structure):
    _fields_ = [("uuid", ctypes.c_ubyte * 16)] + [
        (name, ctypes.c_uint64) for name in (
            "user_ns", "system_ns", "idle_wakeups", "interrupt_wakeups",
            "pageins", "wired", "resident", "footprint", "start", "exit",
        )
    ]


def processes(root: int) -> list[dict]:
    """Use XNU's billed footprint, not a sum of shared-library RSS mappings."""
    libproc = ctypes.CDLL("/usr/lib/libproc.dylib", use_errno=True)
    libproc.proc_pid_rusage.argtypes = [ctypes.c_int, ctypes.c_int, ctypes.c_void_p]
    libproc.proc_pid_rusage.restype = ctypes.c_int
    rows = {}
    output = subprocess.check_output(["ps", "-axww", "-o", "pid=,ppid=,command="], text=True)
    for line in output.splitlines():
        parts = line.strip().split(None, 2)
        if len(parts) == 3:
            rows[int(parts[0])] = (int(parts[1]), parts[2])
    if root not in rows:
        raise RuntimeError("Application exited while sampling its process tree")
    selected = {root}
    while True:
        children = {pid for pid, (parent, _) in rows.items() if parent in selected}
        if children <= selected:
            break
        selected |= children
    result = []
    for pid in sorted(selected):
        if pid not in rows:
            continue
        usage = Rusage()
        if libproc.proc_pid_rusage(pid, 0, ctypes.byref(usage)):
            # Renderer teardown can race a tree snapshot.
            if pid == root:
                raise OSError(ctypes.get_errno(), "Cannot sample application footprint")
            continue
        if pid == root and (usage.exit or not usage.footprint):
            raise RuntimeError("Application exited while sampling its footprint")
        command = rows[pid][1]
        kind = "browser" if pid == root else "other"
        for name in ("renderer", "gpu-process", "utility"):
            if f"--type={name}" in command:
                kind = name
                break
        client_id = re.search(r"--renderer-client-id=(\d+)", command)
        result.append({"pid": pid, "kind": kind, "footprintBytes": usage.footprint,
                       "rendererClientId": int(client_id[1]) if client_id else None,
                       "cpuNs": usage.user_ns + usage.system_ns})
    return result


def sample(root: int, renderer_ids: list[int]) -> dict:
    readings = []
    for index in range(5):
        if index:
            time.sleep(0.5)
        entries = processes(root)
        readings.append({"time": time.monotonic(), "processes": entries,
                         "treeBytes": sum(p["footprintBytes"] for p in entries),
                         "explorerBytes": sum(p["footprintBytes"] for p in entries
                                              if p["rendererClientId"] in renderer_ids),
                         "rendererBytes": sum(p["footprintBytes"] for p in entries
                                              if p["kind"] == "renderer")})
    first, last = readings[0], readings[-1]
    initial_cpu = {p["pid"]: p["cpuNs"] for p in first["processes"]}
    cpu_delta = sum(max(0, p["cpuNs"] - initial_cpu[p["pid"]])
                    for p in last["processes"] if p["pid"] in initial_cpu)
    return {
        "treeFootprintBytes": statistics.median(r["treeBytes"] for r in readings),
        "rendererFootprintBytes": statistics.median(r["rendererBytes"] for r in readings),
        "explorerFootprintBytes": statistics.median(r["explorerBytes"] for r in readings),
        "explorerRendererPids": [p["pid"] for p in last["processes"]
                                 if p["rendererClientId"] in renderer_ids],
        "treeSamplesBytes": [r["treeBytes"] for r in readings],
        "cpuPercentOneCore": cpu_delta / 1e9 / (last["time"] - first["time"]) * 100,
        "processes": last["processes"],
    }


FIXTURE = b"""<!doctype html><meta charset=utf-8><title>Allocating</title>
<style>body{margin:0}div{height:20px;overflow:hidden}</style><script>
window.payload = new Uint8Array(16 * 1024 * 1024); payload.fill(173);
window.ticks = 0;
addEventListener('DOMContentLoaded', () => {
  const fragment = document.createDocumentFragment();
  for(let i=0;i<1500;i++) {
    const node = document.createElement('div');
    node.textContent = String(i) + ':' + 'fixture'.repeat(140); fragment.append(node);
  }
  document.body.append(fragment); document.title = 'Memory ready';
  setInterval(() => {
    const start = performance.now();
    while(performance.now() - start < 1) payload[(++ticks * 4096) % payload.length] ^= 1;
  }, 50);
});
</script>"""


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.send_header("Cache-Control", "no-store")
        self.send_header("Content-Length", str(len(FIXTURE)))
        self.end_headers()
        self.wfile.write(FIXTURE)

    def log_message(self, *_args):
        pass


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sample", type=int, help=argparse.SUPPRESS)
    parser.add_argument("--renderer-ids", type=int, nargs="*", default=[], help=argparse.SUPPRESS)
    parser.add_argument("--package", type=Path, default=Path("dist/playground"))
    parser.add_argument("--shared-runtime", type=Path)
    parser.add_argument("--baseline-runtime", type=Path)
    parser.add_argument("--counts", type=int, nargs="+", default=[1, 5, 10, 20])
    parser.add_argument("--rounds", type=int, default=3)
    parser.add_argument("--observe-freeze-seconds", type=int, nargs="*", default=[],
                        help="Additional snapshots this many seconds after freezing")
    parser.add_argument("--visible-survivor-seconds", type=int, default=0,
                        help="After partial discard, show one survivor and observe natural reclamation")
    parser.add_argument("--output", type=Path, default=Path("explorer-memory-results.json"))
    args = parser.parse_args()
    if sys.platform != "darwin":
        parser.error("This sampler uses macOS proc_pid_rusage; use PSS on Linux")
    if args.sample:
        print(json.dumps(sample(args.sample, args.renderer_ids)))
        return
    if args.rounds < 1 or any(n < 1 or n > 20 for n in args.counts):
        parser.error("Use positive rounds and 1 through 20 simultaneously visible pages")
    if any(n < 1 or n > 600 for n in args.observe_freeze_seconds) or not 0 <= args.visible_survivor_seconds <= 120:
        parser.error("Use 1 through 600 seconds for freeze snapshots and 0 through 120 for a visible survivor")
    package = args.package.resolve()
    script = Path(__file__).with_suffix(".mjs").resolve()
    control_patch = script.parents[2] / "docs/benchmarks/explorer-per-instance-control.patch"
    result = {
        "schemaVersion": 1, "measuredAtUtc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "platform": platform.platform(), "architecture": platform.machine(),
        "measurement": "sum of macOS proc_pid_rusage ri_phys_footprint across the application process tree",
        "sampleCount": 5, "sampleIntervalSeconds": 0.5, "fixturePayloadBytesPerPage": 16 * 1024 * 1024,
        "fixtureDomNodesPerPage": 1500, "fixtureTimer": "1ms busy loop every 50ms",
        "fixtureSha256": hashlib.sha256(FIXTURE).hexdigest(),
        "freezeObserveSeconds": sorted(set(args.observe_freeze_seconds)),
        "visibleSurvivorSeconds": args.visible_survivor_seconds,
        "hardware": subprocess.check_output(
            ["sysctl", "hw.model", "hw.memsize", "hw.logicalcpu"], text=True).splitlines(),
        "control": ("same build with only ResolveExplorerIsolationId restored to unique odd IDs; not a historical full-version comparison"
                    if args.baseline_runtime else None),
        "controlPatchSha256": (hashlib.sha256(control_patch.read_bytes()).hexdigest()
                               if args.baseline_runtime else None),
        "runs": [], "runtimes": {},
    }
    with tempfile.TemporaryDirectory(prefix="tuffite-memory-") as temporary:
        root = Path(temporary)
        fixture = root / "application"
        shutil.copytree(package, fixture, symlinks=True)
        manifest_file = fixture / "tuffite.json"
        manifest = with_runtime_fixture(json.loads(manifest_file.read_text()))
        main_window = manifest["app"]["windows"][0]
        main_window["url"] = "playground://main/ui/dist/fixtures/memory.html"
        manifest["app"]["windows"] = [main_window]
        manifest_file.write_text(json.dumps(manifest, indent=2))
        (fixture / "assets/ui/fixtures/memory.html").write_text(
            "<!doctype html><title>Explorer memory benchmark</title><body></body>")
        runtime = fixture / "libtuffite.dylib"
        progress_file = args.output.resolve().with_suffix(".partial.json")
        progress_file.parent.mkdir(parents=True, exist_ok=True)
        variants = {"shared": args.shared_runtime or package / "libtuffite.dylib"}
        if args.baseline_runtime:
            variants["per-instance"] = args.baseline_runtime
        # Alternate pair order to reduce systematic warm-cache/thermal bias.
        for round_number in range(1, args.rounds + 1):
            for count in args.counts:
                order = list(variants) if round_number % 2 else list(reversed(variants))
                for variant in order:
                    selected = variants[variant].resolve()
                    shutil.copy2(selected, runtime)
                    result["runtimes"][variant] = {
                        "sha256": hashlib.sha256(selected.read_bytes()).hexdigest(),
                        "bytes": selected.stat().st_size,
                    }
                    port = free_port()
                    profile = root / f"profile-{round_number}-{count}-{variant}"
                    command = [str(fixture / "playground"), "--lang=en-US", "--tuffite-runtime-snapshot",
                               f"--remote-debugging-port={port}", f"--user-data-dir={profile}",
                               "--no-first-run", "--disable-features=CalculateNativeWinOcclusion",
                               "--disable-backgrounding-occluded-windows"]
                    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
                    threading.Thread(target=server.serve_forever, daemon=True).start()
                    print(f"round={round_number} pages={count} variant={variant}", flush=True)
                    with (root / "application.log").open("wb") as log:
                        process = subprocess.Popen(command, cwd=fixture, stdout=log, stderr=log,
                                                   start_new_session=True)
                        try:
                            wait_for_cdp(port, process, 30)
                            environment = os.environ.copy()
                            environment["TUFFITE_MEMORY_OPTIONS"] = json.dumps({
                                "port": port, "pid": process.pid, "variant": variant,
                                "count": count, "round": round_number, "python": sys.executable,
                                "driver": str(Path(__file__).resolve()),
                                "progressFile": str(progress_file),
                                "observeFreezeSeconds": sorted(set(args.observe_freeze_seconds)),
                                "visibleSurvivorSeconds": args.visible_survivor_seconds,
                                "origin": f"http://127.0.0.1:{server.server_port}",
                            })
                            completed = subprocess.run(["bun", "run", str(script)], env=environment,
                                                       text=True, capture_output=True,
                                                       timeout=max(180, max(args.observe_freeze_seconds, default=0)
                                                                   + args.visible_survivor_seconds + 180))
                            if completed.returncode:
                                raise RuntimeError(completed.stderr + "\n" + Path(log.name).read_text(errors="replace")[-4000:])
                            entry = json.loads(completed.stdout)
                            result["runs"].append(entry)
                            args.output.parent.mkdir(parents=True, exist_ok=True)
                            args.output.write_text(json.dumps(result, indent=2) + "\n")
                            progress_file.unlink(missing_ok=True)
                            print("  " + ", ".join(
                                f"{name}={stage['treeFootprintBytes']/1048576:.1f}MiB"
                                for name, stage in entry["stages"].items()), flush=True)
                        finally:
                            terminate(process)
                            server.shutdown()
                            server.server_close()
                            shutil.rmtree(profile, ignore_errors=True)
    print(f"PASS {len(result['runs'])} measured runs; {args.output}")


if __name__ == "__main__":
    main()
