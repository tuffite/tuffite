# Explorer process sharing and background lifecycle

Explorer uses a separate WebContents and browsing context. By default, compatible
Explorers from the same owner origin share a renderer for the same Chromium
security principal. Storage partitions, permission policies, site/origin locks,
sandboxing and cross-origin isolation still constrain sharing. Sharing does not
merge DOMs, JavaScript realms, session storage namespaces or navigation history.
The application renderer remains outside the Explorer isolation domain.

Renderer allocation is an internal framework/Chromium policy rather than an
Explorer attribute. Related popups inherit their source domain. Process sharing
increases the scope of renderer crashes and main-thread contention. The existing
`isolated` attribute controls parent/top browsing-context exposure; it does not
request a dedicated process.
Process domains are scoped to the BrowserContext, owner origin and resolved
immutable Explorer grants. A Chromium process lock, not a URL string comparison, decides
whether two SiteInstances can actually share a process.

Mutable `policy` declarations do not select a renderer process. Process sharing
uses the owner origin and immutable capability/partition constraints; changing
menus, delegation or idle schedules cannot fragment that process domain.

## Automatic freeze and discard

```html
<!-- Default schedules: freeze after 30s, discard after 15m hidden idle. -->
<explorer src="https://example.test/dashboard"></explorer>

<!-- Freeze background editors, preserving their document in memory. -->
<explorer policy="discard: never;" src="https://example.test/editor"></explorer>

<!-- Keep background tasks running. -->
<explorer policy="freeze: never; discard: never;" src="https://example.test/worker"></explorer>
```

`policy` is mutable. `freeze` and `discard` independently accept `never`,
`default`, or a duration in `ms`, `s`, `m` or `h` (0ms through 24h). Both timers
start from the same eligible hidden-idle interval. Activity and native
protections postpone automatic work; visibility restores frozen or discarded
pages. `discard: default` can reclaim eligible pages earlier under memory
pressure; an explicit duration remains a minimum interval.

Declaring both automatic actions `never` stops their polling timer. Changing
schedule values starts a fresh idle interval. Explicit `discard()` remains a
separate request: automatic `never` does not block it, while visibility, active
capture, dialogs, related pages and other native protections still apply.
Discard preserves navigation history and persistent storage, drops the current
page's JS/DOM state, and reloads when shown. A replacement document receives a
new document ID; a shared renderer and visible siblings stay alive.

## Resource packs

`tuffite.pak` contains language-independent resources. `tuffite_locales/en-US.pak`
contains only localized strings. Startup loads the two packs separately in every
process. This preserves the existing fixed English locale; this change does not
add language selection or remove ICU data.

The original resources copied one complete pack to both paths. A local x64
sample contained 10,221,710 bytes across the two files. Repacking the same inputs
produced 5,097,506 bytes common resources and 13,367 bytes English strings. Their
resource IDs are disjoint and their merged ID-to-byte mapping equals the original
pack. This is a resource-only sample, not a measurement of a newly linked app.

## Validation

Browser-backed tests are in `explorer_lifecycle_unittests`; a packaged integration
scenario checks actual renderer IDs, cross-site navigation/back, partition separation, hidden freezing, keep-alive, page discard and restoration.
It attaches DevTools only to the application so child-page debugging does not
intentionally suppress suspension.

Build and package the Framework normally, repackage Playground, then run
(from the Framework checkout):

```sh
python3 x.py build --framework --target tuffite/framework/test:explorer_lifecycle_unittests --profile release --target-cpu x64 --non-interactive
../out/TuffiteRelease/explorer_lifecycle_unittests
python3 playground/scripts/smoke.py --package dist/playground --explorer-lifecycle --timeout 180
```

Use a newly packaged app, including both newly generated packs, for the smoke run.
Do not mix a new binary with the old repeated resource packs. Additional release
checks should include existing Explorer transfer, ancestry, reload, resource and
tab suites. Measure full process-tree physical footprint/PSS and CPU with
1/5/10/20 visible and hidden Explorers, plus wake latency, before claiming memory
or CPU improvements. Shared library mappings must not be double-counted as RSS.

The reproducible macOS process-allocation and lifecycle measurement is described
in [Explorer memory benchmark](benchmarks/explorer-memory-macos-x64.md), including
the per-instance diagnostic control, process-tree sampling and last-renderer
restoration regression.
