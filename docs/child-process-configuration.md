# Child-process configuration transport

Status: implemented transport; full linking and packaged runtime validation are deferred.

## Decisions

Split configuration by lifetime and initialization deadline:

- A small immutable process bootstrap snapshot travels in read-only shared memory.
- A typed document exposure snapshot travels in the Mojo messages that create or
  commit that document, before parsing or author script execution.
- Browser-owned services remain the authority for binding and executing native
  operations. Renderer exposure is not an authorization credential.

The target production child command line contains one Tuffite bootstrap handle
switch, for example `--tuffite-bootstrap-handle=...`, rather than scheme/origin
lists. Chromium's own process, sandbox, feature and Mojo transport switches stay.
A shared-memory switch still contains handle metadata; the goal is not zero
command-line arguments or secrecy from the operating system.

## Process bootstrap snapshot

Initially include only application URL schemes. Schemes are process-wide and
must agree across renderer, GPU, utility and zygote processes. The snapshot must
not contain origin allowlists, Shell API grants, resource paths, development
credentials or an entire manifest. Do not add fields simply to unify transports.

Use a bounded binary format with a magic number, format version, payload size,
and length-prefixed scheme array. A small `base::Pickle` codec avoids dependence
on initialized Mojo services. Validate the format, maximum size/count, string
lengths and scheme syntax before registration; reject inconsistent input.
The codec caps input at 1 MiB, 4096 unique schemes, and 1024 bytes per
scheme, rejecting invalid syntax, duplicates, nonstandard Pickle headers and
trailing payload. These are transport limits; existing manifest validation still
requires 1..16 application schemes of at most 64 bytes. This transport migration
does not change the manifest's public schema limits. Do not serialize native C++ object layouts.

The browser creates a read-only region from the package-owned early scheme
configuration and retains it for its lifetime. Children get duplicated handles
to the same immutable snapshot. The browser retains no writable mapping after
publication. The browser itself and single-process tests use the same immutable
value object without synthesizing command-line switches.

Consume it in BasicStartupComplete, before RegisterContentSchemes. In the pinned
Chromium source BasicStartupComplete precedes RegisterContentSchemes, while
PreSandboxStartup follows it and is too late. Do not make scheme registration
wait for Mojo or read the application manifest from a sandboxed child.

Use base::shared_memory::SharedMemorySwitch and ReadOnlySharedMemoryRegionFrom:

- Windows: share a read-only region handle through LaunchOptions.
- macOS: use Mach port rendezvous.
- Linux direct launches: register the shared descriptor.
- Linux zygote: seed the zygote with the same immutable scheme snapshot before
  its scheme registry is locked; cover fork descriptor/lifetime handling and
  inherited initialized state explicitly. The normal child launcher is not the
  only path to audit.

AppendExtraCommandLineSwitches cannot accomplish handle transfer alone because
it does not receive LaunchOptions. GetAdditionalMappedFilesForChildProcess is
not a cross-platform replacement; the pinned implementation does not call it
on macOS. Add a narrow, build-guarded Content integration alongside existing
shared-memory launch transfers. Preserve launch sequence ownership and keep the
snapshot alive until the launcher completes. Allocate distinct rendezvous and
descriptor keys and verify they cannot collide with Chromium's keys.

Missing/invalid bootstrap data in a production Tuffite child is a startup error.
Tests explicitly provide a snapshot; they do not silently enable legacy argument
fallback. Scheme configuration cannot change after the process registry is
locked. Profile and document permission changes do not replace this snapshot.

## Document exposure snapshot

Initially send only two decisions, computed by browser-owned manifest/role
services for the effective document origin:

```text
DocumentConfiguration {
  expose_shell_api = false
  enable_explorer = false
}
```

Do not send whole origin pattern lists or native method grants. Preserve the
current Shell API main-frame restriction and the existing owner/role rules.
Use the effective committed origin, including inherited/opaque origin rules,
rather than reconstructing identity from a URL string.

The implemented optional `tuffite_configuration` is part of
`blink.mojom.PolicyContainerPolicies`, carried by the existing document creation
and NavigationClient.CommitNavigation Mojo messages. Existing document tokens
and PolicyContainer ownership provide routing; no duplicate token field or
additional configuration round trip is introduced. Missing configuration means
denied exposure. Failed navigations retain denied defaults.

Both Content's and Blink's WebPolicyContainer conversion functions preserve the
two exposure bits. Initial about:blank/srcdoc documents and popups use the
existing PolicyContainer inheritance paths; opaque origins are denied again at
the renderer reader. Browser-created initial frames and popup replies are
configured before sending their existing creation messages.

The resolved snapshot lives in the current document's PolicyContainer, not the
RenderProcess or a reusable RenderFrame's last configuration. Install it before DidCreateScriptContext and before the
parser or document.createElement can select HTMLExplorerElement. DispatchBindings
and HTMLExplorerElement::IsEnabled then consult this document value. Keep the
TuffiteExplorer Blink feature as a process-wide implementation gate; browser
policy determines exposure for each Document. Always-on compilation of a feature
is not a grant to use it.

DocumentToken is used for matching/routing, not as a bearer authorization token.
A missing document policy defaults to denied.
Browser services still check the sender's current document, role and grants on
bind/call; outstanding requests and callbacks must not outlive that document.

### Initial documents and lifecycle

Normal navigation commit plumbing alone is insufficient:

- Browser-created initial documents: include the initial policy in the frame/view
  creation data before any script context can be created.
- window.open: return the initial document policy with CreateNewWindowReply and
  its existing DocumentToken. Account for opener/noopener and sandboxed origins.
- Renderer-created initial about:blank/srcdoc: preserve synchronous creation.
  Permit only a conservative inherited subset when creator policy, effective
  origin, sandbox restrictions and frame-role rules establish it locally. Unknown
  cases remain denied. Validate these rules against browser authorization; if
  exact behavior cannot be preserved, extend the creation path rather than
  installing an optimistic policy and correcting it after scripts have run.
- Cross-document navigation/redirect/process reuse: compute a new snapshot for
  the final document. Never reuse a frame's old exposure bits.
- Same-document navigation: retain the existing Document snapshot.
- BFCache restore: the cached Document keeps its snapshot. Native operations
  always consult current browser grants; live revocation does not rely on DOM
  interface removal. Stronger immediate exposure revocation would require an
  explicit BFCache eviction/update policy and is outside the initial migration.
- Freeze/resume and detach/attach: preserve the existing Document snapshot;
  transfer continues to enforce the current browser-side compatibility checks.
- Discard/reload: treat the replacement as a new Document and recompute policy.
- Prerender: keep the pinned Chromium restrictions on Explorer/inner contents;
  transport changes must not accidentally expand supported behavior.

## Ownership and patch boundaries

Keep the declaration model, generic asset/access rules, bootstrap codec and
document snapshots in the single `tuffite/components/manifest/` component.
Application startup loading, resolution and EffectiveManifest ownership belong
to `framework/app/services/manifest/`.
Framework retains ManifestService lifetime and browser role/origin resolution;
URLService owns runtime resource resolvers, and `framework/common/resource.h`
provides the shared in-memory resource value. Renderer targets depend only on
the document Mojom/bridge, not the manifest loader or Framework services.
Use dedicated browser/content, renderer/content and renderer/blink integration
subdirectories for code compiled into Chromium targets. Keep Mojom definitions
under Tuffite ownership, with dependencies limited to foundational token/types
so the import into Content does not create a dependency cycle.

The minimum Chromium changes are launch-handle plumbing and document
creation/commit plumbing. Guard GN/C++ integration with tuffite_in_chromium or
BUILDFLAG(TUFFITE_IN_CHROMIUM); mark unavoidable unconditional Mojom/import changes
with narrow balanced TUFFITE boundaries. Supply source/dependency lists through
feature-prefixed variables in the integration directory's tuffite_in_chromium.gni.
Do not add Tuffite source lists directly to Chromium build files.

## Migration and acceptance

1. Add and validate the bootstrap codec, cross-platform handle plumbing and early
   reader. Switch all scheme consumers and remove tuffite-app-schemes.
2. Add document snapshots across commits and initial-document paths. First verify
   semantic parity with the existing browser policy, without adding extra API
   exposure. Shadow comparisons should cover all transition cases above.
3. Switch Shell API/Explorer renderer readers to Document snapshots. Remove both
   origin switches, serialization methods and renderer pattern parsing.
4. Remove single-process switch injection. Keep development/test controls in
   their separately gated startup paths; do not put credentials in this snapshot.

Acceptance checks:

- Application schemes work before the first URL/origin parsing on every platform,
  including zygote and child restart paths. Invalid handles/codec inputs fail.
- First inline script and parser-created Explorer see the correct exposure,
  with no timing-dependent enablement or new document round trip.
- Allowed and denied documents coexist in one reused renderer without leakage.
- Initial empty documents, srcdoc, sandbox/noopener popups, redirects, error
  pages, BFCache, transfer and discard/reload preserve the intended behavior.
- Forging renderer exposure state cannot authorize browser-native operations.
- Production child arguments contain no full origin lists or development tokens.

This is primarily an initialization, ownership and command-line hygiene change.
Scheme lists are small: shared-memory handles/mappings have overhead, so do not
promise memory or startup speed savings without measurements. Measure first-frame
latency, process-tree footprint and IPC/argument bytes against a fixed build.

## References

- Local launch transport: base/memory/shared_memory_switch.h and
  content/browser/child_process_launcher_helper.cc.
- Local initialization order: content/app/content_main_runner_impl.cc.
- Local document delivery: content/common/navigation_client.mojom and
  content/common/frame.mojom.
- [Chromium Mojo ordering guidance](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/mojo_and_services.md).

## Local checks and deferred runtime checks

The configuration native suite covers large scheme lists, invalid snapshots,
unknown/opaque document denial, document policy replacement, and the
Content/Blink policy conversion boundary. It is included in the Framework native
test group as `configuration_unittests`.

The packaged regression fixture records exposure in its first inline script,
checks synchronous initial blank children and opaque sandbox frames, navigates
between allowed/denied origins, returns through history, and loads the second
business scheme. On POSIX it also checks child arguments for removed lists and
development credentials without logging argument contents.

After rebuilding and repackaging Framework and Playground, run:

```sh
python3 x.py test --suite native --profile release --target-cpu x64 --non-interactive
python3 playground/scripts/smoke.py --package dist/playground --configuration --timeout 180
```

Runtime scenarios must use one freshly rebuilt binary/resource set. Do not run
against old packages and report them as validation of this transport. Linux
zygote and Windows handle paths require their platform CI runs.
