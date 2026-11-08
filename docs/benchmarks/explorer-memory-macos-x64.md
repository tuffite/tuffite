# Explorer memory: process sharing and page lifecycle

## Results

Measured on a MacBookPro16,1 with 32 GiB RAM and 12 logical CPUs. Values below
are whole-application footprint in MiB, using the median of three fresh-profile
runs. The baseline is an allocation-strategy control, not a historical release.

[Full 24-run data](explorer-memory-macos-x64.json) records every stage, native
document/renderer ID, mapped OS PID and footprint sample.

| Pages | Explorer OS renderers, control → shared | Control | Shared | Savings |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 1 → 1 | 178.2 | 179.3 | -1.1 (-0.6%) |
| 5 | 5 → 1 | 466.7 | 388.9 | 77.8 (16.7%) |
| 10 | 10 → 1 | 834.6 | 668.4 | 166.2 (19.9%) |
| 20 | 20 → 1 | 1565.7 | 1254.2 | 311.5 (19.9%) |

The single-page ranges overlap: there is no meaningful single-page saving.
For 20 visible pages, control runs ranged from 1,538.2–1,667.8 MiB and shared
runs from 1,248.1–1,260.2 MiB. Process counts matched the expected values in
every run; memory totals vary with normal GC and cache timing.

| Shared pages | Hidden, running | Frozen | Discard floor(N/2) | Discard all | Reload all, seconds |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 178.5 | 177.9 | — | 101.1 | 0.44 |
| 5 | 388.9 | 389.0 | 378.2 | 105.9 | 1.89 |
| 10 | 661.9 | 662.0 | 636.3 | 112.4 | 3.68 |
| 20 | 1246.9 | 1246.3 | 1202.1 | 150.6 | 7.20 |

Freeze preserved document state and produced little immediate footprint change.
Partial discard did not return memory in proportion to the discarded page count:
discarding ten of twenty frozen documents reduced the tree by only 12.9–60.3 MiB
within the short sampling window. Discarding all twenty released their actual
renderer process in every run and reduced the tree to 139.8–159.7 MiB.
The application renderer, spare renderer, browser, GPU and network service remain.

Reload time above is for all documents, with cache-disabled synthetic pages; it
is not single-page wake latency or time to first painted frame. For twenty pages,
reloading all documents took a median 7.20 seconds. Discard therefore trades
substantial reclamation for document-state loss and reload work.

### Five-minute observation and visible survivor

One additional twenty-page run used the same optimized library and fixture.
This is a timing observation, not a three-run causal comparison against keeping
all pages hidden and unfrozen. [Observation data](explorer-memory-macos-x64-observation.json)
includes the full per-process records.

| Stage | Tree footprint, MiB | Explorer renderer footprint, MiB |
| --- | ---: | ---: |
| Visible | 1234.7 | 1030.9 |
| Hidden, running | 1228.4 | 1031.0 |
| Frozen, short wait | 1228.5 | 1031.1 |
| Frozen, about 60 seconds | 1116.9 | 989.2 |
| Frozen, about 300 seconds | 1115.6 | 987.5 |
| Ten discarded, ten still frozen | 1116.2 | 988.0 |
| Ten discarded; one survivor shown for ten seconds | 1152.6 | 1007.7 |
| All twenty discarded | 136.2 | 0.0 |
| All twenty reloaded | 1154.4 | 1015.5 |

While the documents stayed frozen, footprint declined by about 113 MiB over
five minutes. This includes ordinary background GC/cache reclamation; it does
not establish that freezing alone caused the decline. The pinned Chromium
source disables `kMemoryPurgeOnFreeze` by default on desktop, while its
`MemoryPurgeManager` schedules background purges with a one-to-four-minute
delay. No feature overrides or forced collections were used here.

Discarding half the documents after that wait still did not yield immediate
proportional reclamation. Showing one surviving page and waiting ten seconds
also did not reduce footprint in this run. The exact retained-object/allocator
cause was not profiled; these results do not establish a leak or an eventual
upper bound. Per-document discard inside a surviving shared renderer remains
an optimization/measurement target. Logical document disposal must not be
equated with immediate physical-memory return.

## Method

Measured on macOS x64 on 2026-11-08 with the packaged Playground and the local
non-component Release Framework. This profile uses size optimization, line-table
symbols and no ThinLTO. These measurements do not represent the production CI
ThinLTO profile or an entire historical application release.

The control restores the previous per-Explorer allocation strategy:
`ResolveExplorerIsolationId` returns a fresh odd ID (`NextDomainId() + 1`) for
each Explorer, preventing process-per-site reuse. The optimized build resolves a
shared domain normally. Both libraries otherwise use the same source, compiler,
resources, default storage partition, owner origin, permissions and fixture.
This avoids using different storage partitions as a proxy for process isolation.
The exact library hashes are recorded in the accompanying JSON.

The test serves identical synthetic pages from one ephemeral loopback HTTP
origin. Each page retains a touched 16 MiB `Uint8Array`, 1,500 DOM nodes, and a
timer with a 1 ms busy loop every 50 ms. The application has one otherwise empty
window; all Explorers fit in its viewport. This is a real renderer measurement
with a controlled workload, not a measurement of a particular business app.

For each of 1, 5, 10 and 20 pages, three fresh-profile runs of both variants are
performed, alternating variant order between rounds. Each stage settles for
two seconds (four after discard), then takes five samples 500 ms apart. Each
run records the median; the result tables use the median across three runs.

The metric is the sum of `proc_pid_rusage(RUSAGE_INFO_V0).ri_phys_footprint`
over the application's complete process tree, including the browser, GPU,
network service, application renderer, spare renderer and Explorer renderers.
It uses macOS charged footprint, including compressed memory, rather than a sum
of RSS mappings that repeats clean shared libraries. It is not Linux PSS or a
measurement of total system RAM freed. Renderer counts in the comparison table
count only the loaded Explorer documents' renderers.

DevTools attaches only to the application document: attaching to an Explorer
would intentionally protect it from freezing/discard. The test asserts native
renderer IDs and frozen/discarded states. It uses `freeze: never; discard: never`
for the visible and hidden-running stages, then `freeze: 0ms; discard: never` to
exercise freezing without waiting for the normal 30-second default. Explicit
`discard()` exercises the same native discard operation without waiting for the
15-minute default. No forced GC, synthetic memory pressure or renderer debugger
is used. The build process exits before the official measurements start.

Chromium's existing `--renderer-client-id` maps the native renderer IDs to OS
PIDs without changing the public API. The sampler records Explorer-only
footprint alongside the full tree and asserts that the originally loaded
Explorer renderer PIDs have exited after final discard.

After discarding every page, showing the elements must automatically reload all
documents, produce new document IDs, and reuse one renderer again. Restore time
is observed by 100 ms polling, so it should not be interpreted as a precise
per-page latency benchmark.

## Restoration regression

The first pilot exposed a missing last-page case. Discarding the last page
successfully released its renderer, but the connector stopped forwarding owner
visibility when its child view no longer existed. Showing the element therefore
did not reload it. The fix forwards visibility only for an attached, discarded
Explorer through Chromium's existing inner-WebContents delegate, retaining its
outer-window visibility cap. Ordinary frames keep their existing handling.

The packaged lifecycle smoke now covers an independent partition's only page
being discarded, its renderer exiting, and automatic restoration. The memory
matrix also asserts restoration after discarding every shared page.

## Reproduction

After building and packaging Playground, run from the Framework repository:

```sh
python3 playground/scripts/explorer-memory-benchmark.py \
  --package dist/playground \
  --shared-runtime /path/to/shared/libtuffite.dylib \
  --baseline-runtime /path/to/per-instance/libtuffite.dylib \
  --counts 1 5 10 20 --rounds 3 \
  --output explorer-memory-results.json
```

For the additional five-minute observation, run the shared variant alone:

```sh
python3 playground/scripts/explorer-memory-benchmark.py \
  --package dist/playground --shared-runtime /path/to/shared/libtuffite.dylib \
  --counts 20 --rounds 1 --observe-freeze-seconds 60 300 \
  --visible-survivor-seconds 10 --output explorer-memory-observation.json
```

Use Bun from the workspace's Mise environment. The exact diagnostic source
change is [explorer-per-instance-control.patch](explorer-per-instance-control.patch),
which is not part of the production patch series:

```sh
git apply docs/benchmarks/explorer-per-instance-control.patch
python3 x.py build --framework --target tuffite/framework:framework \
  --profile release --target-cpu x64 --non-interactive
cp ../out/TuffiteRelease/libtuffite.dylib /tmp/tuffite-per-instance.dylib
git apply -R docs/benchmarks/explorer-per-instance-control.patch
python3 x.py build --framework --target tuffite/framework:framework \
  --profile release --target-cpu x64 --non-interactive
```

Always reverse the diagnostic patch, including if a control build fails. Then
use the restored Release output as the shared runtime and the saved library as
the baseline runtime. No diagnostic switch or process-isolation attribute is
added to production. The driver copies the package to a disposable directory
and leaves the supplied package untouched. On a failure, a sibling
`.partial.json` preserves the completed stages.
