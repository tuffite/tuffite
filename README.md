# Tuffite in Chromium

Tuffite is an **experimental source release**, intended for Framework development
and evaluation. Production security hardening and supported SDK/Distribution
releases are still in progress; see [production readiness](docs/production-readiness.md).

This repository owns the native Tuffite code compiled inside Chromium and the
small patch series required to connect it to Chromium-owned targets. In the
standard gclient layout it is checked out at `src/tuffite`; Chromium remains
the parent repository at `src`.

The repository owns:

- `framework/`: browser, renderer, utility, window, shell, and ABI code;
- `components/`: reusable features, including Explorer Content/Blink integration;
- `crates/`, `framework/public/abi.h`, and `packages/`: the Rust SDK, public ABI, generator,
  CLI, and JavaScript runtime;
- `playground/`: the end-to-end application fixture and smoke test;
- `build/`, `scripts/`, `x.py`, and `docs/`: native profiles, build/package tooling,
  and framework documentation;
- `tuffite_in_chromium.gni` and the native GN targets;
- `DEPS`: the exact Chromium baseline and the patch-application hook;
- `patches/chromium`: ordered `git format-patch` output for Chromium-owned
  files.

The outer Tuffite repository at the gclient workspace root owns only checkout
and developer-environment orchestration (`.gclient`, `mise.toml`, editor
configuration, and the documentation website submodule). Its
`.gclient` pins this repository to an exact commit; its `DEPS` alone declares
the Chromium URL and revision. No additional native lock
file is used.

## Development

Prepare tools and perform the first pinned checkout from the outer workspace:

```console
mise trust
mise run setup
```

Then work from `src/tuffite`. The three daily entries are:

```console
python3 x.py roll         # Reconcile dependencies and patches after branch changes
python3 x.py build --run  # Build local Framework + Playground and launch Vite HMR
cargo spo            # Inspect, edit, group, export, and recover patch stacks
```

`python3 x.py` prints help. Explicit commands and parameters support
non-interactive CI. `cargo spo export --all --verify-apply` saves Chromium commit
stacks into this repository; `cargo spo check --all --strict` verifies canonical
output and replay. One mailbox file may contain multiple consecutive commits.
The first-checkout gclient hook remains Python-standard-library + Git only.

See [Framework development](docs/development.md) for first checkout, local
builds, roll recovery, patch editing, packaging, upload, and CI commands.

## GN boundary

The checked-in build profiles set both:

```gn
tuffite_in_chromium = true
root_extra_deps = [ "//tuffite:tuffite" ]
```

`root_extra_deps` connects the Tuffite distribution to Chromium's existing root
graph without patching Chromium's root `BUILD.gn`. Chromium-owned BUILD files
only import the component integration directory's `tuffite_in_chromium.gni` and consume its
`tuffite_`-prefixed variables inside `if (tuffite_in_chromium)` blocks. Directly
compiled sources use source-absolute `//tuffite/...` paths.

## Service interfaces and scoped IoC

The native implementation exposes a language-neutral C ABI. Host SDKs live in
this repository; business code remains in downstream application repositories.
`.*` denotes `.h` and `.cc`;
BUILD.gn and layer-specific DEPS files are omitted below.

```text
src/tuffite/
|-- BUILD.gn / DEPS / README.md / tuffite_in_chromium.gni
|-- components/                         # No framework includes
|   |-- dispatch/
|   |   |-- common/ dispatcher.mojom / message.h / status.h / codec.* / switches.h
|   |   |-- browser/ dispatch_host.* / rpc_router.* / request_registry.*
|   |   |           / rpc_handler.h / host_dispatch.h
|   |   |-- renderer/ dispatch_bindings.*
|   |   `-- test/ codec_unittest.cc / request_registry_unittest.cc
|   `-- explorer/ README.md
|       |-- common/ explorer.mojom
|       |-- browser/ explorer_host.* / explorer_surface_host.* / explorer_router.*
|       |           / explorer_policy.h / explorer_policy_provider.h
|       |           / explorer_surface_data.* / explorer_browser_support.*
|       |           / explorer_partition.* / explorer_partition_snapshot.*
|       |           `-- content/ explorer_isolation.* / tuffite_in_chromium.gni
|       |-- renderer/ explorer_surface_bindings.*
|       |           `-- blink/ html_explorer_element.* / explorer_dialog.*
|       |                     / explorer_event_propagation_monitor.* / *.idl
|       |                     / tuffite_in_chromium.gni
|       `-- test/ explorer_browser_support_unittest.cc / explorer_{test,target}.html
|-- framework/
|   |-- public/ abi.h
|   |-- bootstrap.cc / runtime.cc
|   |-- common/ app_manifest.* / capability_pattern.* / constants.h / content_client.*
|   |          / utility/utility_service.mojom / utility_service_name.h
|   |-- app/ application_context.* / main_delegate.*
|   |   |    abi_host_bridge.* / process_launch.*
|   |   `-- services/ application_dependency* / application_keyed_service_factory.h
|   |                  asset/ / policy/ / remoting/ / utility/
|   |-- browser/ browser_context.* / browser_main_parts.* / content_browser_client.*
|   |   |        web_contents_role.* / web_contents_view_delegate.*
|   |   `-- services/ client_hints/ / context_menu/ / devtools/ / download/
|   |                  file_chooser/ / javascript_dialog/ / network_context/
|   |                  permission/ / shell_api/ / webauthn/ / window/
|   |-- renderer/ content_renderer_client.*
|   |-- utility/ content_utility_client.* / utility_service.* / js_extension_session.*
|   `-- test/ abi_host_bridge_unittest.cc
|                 / utility_service_unittest.cc / utility_test_main.cc
|-- patches/ config.json / chromium/.patches / chromium/*.patch
`-- scripts/ # Python Framework development commands
```

ShellAPI handlers stay in `framework/browser/services/shell_api/commands`. Native window ownership and
platform implementation live in the context's WindowService directory.
DevTools is a BrowserContext service. Utility supervision is an application
service.
There is no extra protocol, composition, integration, or environment service.

### Component and service responsibilities

**Dispatch** owns the reusable RPC envelope, request IDs, response statuses,
encoding, cancellation messages, routing machinery, and connection-scoped
request registry. Its internal request identity is `(connection, request_id)`;
caller-supplied IDs are not authorization credentials. Each request reaches
one terminal state: completed, cancelled, or timed out. Cancellation requests
backend cancellation and stops waiting for the result; it does not promise to
undo side effects. Disconnect cancels the connection's pending requests and
subscriptions. The codec uses explicit lengths and byte order, rejects invalid
or unsupported messages, and bounds payload and in-flight request sizes.
Frame validation and request-ID rewriting stay in the same codec API, so ABI
bridges do not duplicate magic offsets or partially validate envelopes.

Dispatch does not know the application manifest, host implementation language,
or builtin capability implementations. `RpcHandler` supplies authorization and
invocation; `HostDispatch` supplies the host callback port. The framework
implements both contracts. Renderer bindings communicate through Mojo rather
than sharing browser Service pointers. The codec uses one `CERM` frame prefix
with explicit request, response, and cancellation kinds. Mojo carries
structured requests and cancellation. Cancellation decoding is available to
other transports; it does not imply that the current host ABI consumes a
cancellation frame or rolls back a cancelled operation.

**Explorer** owns embedded WebContents behavior, browser-maintained surface
relationships, navigation, messages and bubbling, dialogs, and partition
snapshot integration. It defines effective policy types,
`ExplorerPolicyProvider`, and per-context routing state. `ExplorerPartition` is the single parser for
partition modes and names. `ExplorerRouter` owns registration, lookup, and bounded
document queues behind methods rather than exposing mutable containers. Application roles,
manifest parsing, and ShellAPI trust rules remain in the framework. Explorer
surface data must not depend on the framework's `WebContentsRoleData`. Blink
element/IDL sources live in `components/explorer/renderer/blink`; Content
isolation sources live in `components/explorer/browser/content`. They retain
their Chromium compilation targets, with private dependency exceptions scoped
to these integration directories. Each integration directory owns its
`tuffite_in_chromium.gni` contracts, imported directly by Chromium build files.

**ApplicationPolicyService** borrows the application owner's immutable
configuration and policy decisions, including Explorer capability resolution.
Its implementation implements the component's `ExplorerPolicyProvider`
interface. Dynamic caller identity and effective page policy are supplied from
browser-owned state; the service does not retain page pointers, routing
queues, or window instances.

**ShellApiService** owns the context's RPC Router and integrates builtin
handlers, per-call authorization, subscriptions, and host namespace
forwarding. Dispatch's codec encodes RPC envelopes. Generated route adapters
encode and decode typed business payloads through
`components/dispatch/browser/shell_api_protocol`; business handlers never read
or append wire bytes. `ShellApiServiceImpl` does not depend on those primitives
directly. Window handlers use the typed WindowService interface. Application-scoped
handlers resolve interfaces from their explicit service context; only
BrowserContext composition entry points query BrowserContext factories. Window,
storage, and Utility operations remain their respective implementations'
responsibility.

### Generated builtin ShellAPI protocol

Builtin ShellAPI declarations are split by domain (`common`, `dialog`, `fs`,
`log`, `network`, `path`, `storage`, `utility`, and `window`) and listed
explicitly in `packages/tuffite/tuffite.json`. The ordered `definitions` list is
the deterministic input set; generation does not depend on filesystem glob
order. The existing TypeScript parser merges the files as one schema, validates
duplicate definitions and cross-file references, and emits all language outputs
in one run.

This repository owns the declarations and the single cross-language generator.
`python3 x.py build --framework` incrementally builds the host generator. GN resolves that
executable at its stable repository-relative Cargo output path; no generated
GN args are required. The `shell_api_protocol` invocation declares the same
ordered `.d.ts` inputs needed for GN's static output graph, derives their stems
and mirrored `root_gen_dir` directory itself, and runs the generator once.

Generated C++ is split into dependency-safe layers: checked-in typed protocol
support under Dispatch and one generated pair per schema. Its namespaces
mirror the selected `.d.ts` API root: top-level declarations stay directly
under `tuffite::shell_api::generated`, while domain routes and nested declarations
use namespaces such as `generated::fs` and `generated::window`. This preserves
schema ownership and prevents unrelated domains from competing for flat type
names.

Business implementations under `framework/browser/services/shell_api/commands` receive typed
requests and return typed results. They never include the protocol codec or
append/read wire bytes directly. Each declared route has one generated adapter
and one distinct business entry point. Shared implementation helpers may take
an operation enum or typed request, but no handler performs a second dispatch
using route strings. `BuiltinRoutes()` is generated from the schema and is the
only builtin route table. The low-level codec under
`components/dispatch/browser/shell_api_protocol` preserves the generated
business payload wire format; the former Framework-local codecs and duplicated
route table have been removed.

**UtilityService** owns the browser-side Supervisor and coordinates shutdown.
The Supervisor owns per-service hosts and handles queues, sessions,
backpressure, timeouts, crash recovery, restart backoff, and circuit breaking.
An injected `UtilityProcessLauncher` separates supervisor logic from Chromium
process launch; the supervisor uses the Mojo service contract directly. The
Utility-process implementation and ABI execution backend remain in
`framework/utility/`; JavaScript sessions are implemented alongside the utility
process service. Extracting a reusable supervisor component is deferred until
a concrete second consumer justifies that boundary.

**WindowService** owns the registry and coordinates window policy, creation,
operations, and platform backends. Window destruction unregisters identity
explicitly instead of relying on full-list reconciliation. Explorer dialog
placement uses the injected policy and browser-owned window integration. The
registry and backend are internal owned objects, not additional keyed
services.

**DevTools** keeps its frontend, Chromium delegate, and optional
remote-debugging server together. It follows Chromium ownership contracts
without an extra Service/Impl/Factory layer. Process-wide debugging startup
and shutdown remain application lifecycle responsibilities.

### Interface programming and composition

Core implementations receive explicit constructor dependencies. Service
interfaces describe typed capabilities; components use their own narrow Ports
and Delegates. Factories select implementations, inject dependencies, and
state `DependsOn()` relationships. Application-scoped consumers resolve an
interface from their explicit `ApplicationContext` with
`GetService<T>()`; they do not include the concrete Factory. This is scoped
lookup, not a process-global `Get<T>()` service locator. BrowserContext
factories remain composition and binder entry points.

Components must not depend on framework headers. Browser, renderer, Utility,
and shared protocol code compile into separate targets. `common` directories
contain only genuinely shared contracts; they cannot include process-specific
APIs. A `core` directory is added only for demonstrated Content-independent
reuse, not mechanically for every feature. An additional sibling application
scenario can reuse components with its own policy, handlers, delegates, and
backends.

Service directories contain each capability's complete browser implementation.
Only interface targets are exposed broadly; composition can depend on
factories, and private implementation/backend targets use narrow visibility.
Plain helper objects do not need factories. Preserve explicit dependency and
construction code rather than hiding it in macros or automatic registration
templates.


### Scope, ownership, and application KeyedService lifecycle

"Global" means that the factory registry and dependency manager have stable
process-lifetime addresses. Service instances remain scoped to one
`ApplicationContext`; they are not process-wide singletons.

`MainDelegate` owns the host bridge, MainDelegate, frozen manifest, and one
application service context. `ApplicationDependencyManager::Get()` is a
`base::NoDestructor` singleton. Each
`ApplicationKeyedServiceFactory<Service>` registers its service-interface
type with that manager and remains an explicit node in Chromium's dependency
graph. The manager's type registry stores factories only. Chromium's keyed
factory storage owns the per-context service instances.

Application consumers use:

```cpp
UtilityService* utility =
    application_context.GetService<UtilityService>();
```

They include `ApplicationContext` and `UtilityService`, not
`UtilityServiceFactory`. Concrete factories are referenced by the central
`ApplicationContext` constructor and by
other factories when declaring `DependsOn()` edges.

`BrowserContextKeyedServiceFactory` subclasses continue to own ShellAPI
and Window services per context. `BrowserContext` owns ordinary
ExplorerRouter state and retains one non-owning reference to its application's
service context. `BrowserContext::GetFrom()` centralizes the framework
downcast from Chromium BrowserContext or RenderFrameHost. Renderer and Utility
processes communicate over Mojo and never borrow browser-process service
pointers.

Factories are built before the first application context becomes live.
`ApplicationDependencyManager::CreateServices()` marks the context live and
lets Chromium create required services. Destruction calls
`DestroyContextServices()`; Chromium performs two-phase shutdown and
dependency-ordered destruction. Tuffite does not own another service-instance
map.

Application and BrowserContext services are separate dependency graphs. A
cross-scope dependency is resolved while constructing the shorter-lived
BrowserContext service and injected as an interface. It must not be represented
with a cross-manager `DependsOn()` edge.

The normal lifecycle is:

```text
Starting -> Running -> Quiescing -> Stopped

1. Freeze configuration and build every application Factory singleton.
2. Create ApplicationContext and its keyed services.
3. Create BrowserContexts and inject borrowed application interfaces.
4. Close admission; cancel or drain RPC, ABI, session, and Utility work.
5. Destroy windows, documents, BrowserContext services, and BrowserContexts.
6. Destroy application keyed services through ApplicationDependencyManager.
7. Destroy ApplicationContext, then release manifest and bridge state.
```

`BrowserMainParts` receives only `ApplicationContext&`; policy
is resolved from that context when an interface is required. `MainDelegate`
does not expose a duplicate `policy()` accessor.

Tuffite fixed-address process-lifetime singletons use `Get()`, including its
dependency manager and keyed-service factories. `GetInstance()` is reserved
for instance pointers that may change or be null at runtime. Chromium-owned
classes retain their upstream naming.

KeyedService reduces manual ownership and destruction-order errors but does not
eliminate use-after-free. Async callbacks still use WeakPtr or scoped request
handles; shutdown stops timers, detaches observers, invalidates weak callbacks,
and resets remotes. Document tokens continue to isolate request IDs and
cancellation across pages.

Detailed factory, lookup, dependency, cross-scope, and testing rules live in
`framework/browser/services/README.md`.

### Host ABI and testing

The ABI exposes opaque handles and capability functions, plus host callbacks
and explicit stop/completion contracts. It does not expose Service pointers,
Factories, keys, Rust Futures, or Rust ownership types. Callback tables and
their user-data contexts remain valid until all admitted calls and callbacks
finish; handle validation alone cannot protect a dangling callback context.
Host libraries must remain loaded until no callback can invoke their code. An
outer language SDK implements its own task scopes, async wrappers, and safe
handle access without taking ownership of native services.

Tests construct request registries and UtilityService implementations
directly. The utility fake launcher exercises real Mojo messages with mock
time, including crash failure, timeout completion, and shutdown without
restart. Host bridge tests cover failed-start cleanup of queued calls and
rejection after quiescing. Application tests cover idempotent partial-startup
shutdown plus isolation of Policy and Utility instances across two live
application contexts. Codec tests cover malformed data and wire compatibility.
Explorer parser tests cover every partition mode and invalid names.
Smoke and compositor observers live in `browser_test_observers.*`, keeping
`BrowserMainParts` focused on lifecycle composition. Existing Explorer support and
capability-pattern tests remain in the native `unittests` group.

From the outer workspace, reusing the release cache:

```console
python3 x.py build --target tuffite/framework:unittests tuffite/framework:distribution
# Run the six *_unittests executables under src/out/TuffiteRelease.
python3 x.py package --skip-build
cargo run --package tuffite --bin tuff -- package --manifest playground/tuffite.json
python3 x.py test --suite smoke
```

Playground smoke exercises the packaged multiprocess application, including
RPC, network/storage, windows, Explorer
navigation/dialogs/partitions/messages, and Utility async calls, concurrency,
persistent sessions and JavaScript. The unit tests do not claim exhaustive
multi-context or UAF coverage. Future changes to lifecycle graphs should add
targeted factory isolation and teardown cases rather than relying solely on
smoke.


### Service refactor verification (2026-09-21)

The implementation was incrementally built and linked through
`tuffite/framework:distribution` in the existing
`src/out/TuffiteRelease` cache. The ABI/Application test executable passed
all 4 cases, including isolation of Policy and Utility services across two
Applications. The Utility executable passed all 4 shutdown, crash, owner
isolation, timeout, and backoff cases.

The Framework package verified 18 runtime files. The rebuilt Playground package
passed the full multiprocess smoke suite with `status: ok`, including
ShellAPI, network/storage, windows, Explorer rendering/navigation/dialogs/
partitions/messages, Utility async/concurrent/session paths, JavaScript
extension, and normal process exit.

`git diff HEAD --check` passed after normalizing modified text files to the
repository's LF convention. Chromium's clang-format was applied to every C++
file changed in the follow-up refactor.

### Startup composition and entry linkage

`framework/app` contains MainDelegate, ApplicationContext and the ABI host bridge.
MainDelegate is the process composition root for every Chromium role; only the
browser role creates the application keyed-service graph. It owns the Content
clients and application state in `app:app`. ContentBrowserClient and
BrowserMainParts compile in `browser/`; Chromium owns BrowserMainParts.

`framework/main.cc` owns the thin `tuffite_launcher` executable entry and Windows
sandbox initialization. `framework/bootstrap.cc` owns host library loading in
the shared `:framework` runtime, avoiding a second allocator in the executable.
`runtime.cc` is the DLL-side C ABI entry, compiled in `:core` with
`TUFFITE_FRAMEWORK_IMPLEMENTATION`; it supplies the exported `tuffite_run_application`
and constructs and runs Application. These two translation units preserve the
executable/runtime sandbox boundary without a separate entry archive.


### Application/bootstrap and service lookup

The public runtime entry is `tuffite_run_application`.
`main.cc` initializes executable-level sandbox state and enters the shared
launcher implementation; `runtime.cc` creates `MainDelegate`.
`app/abi_host_bridge.*` adapts the host ABI and coordinates host start,
stop, and callback delivery.

Posted ABI work retains the owning `ApplicationContext`, then calls
`GetService<UtilityService>()` on the browser sequence. It does not retain a
UtilityService pointer across shutdown. `MainDelegate` destroys its keyed
services before destroying the context, manifest, or bridge.

Framework build ownership follows the implementation directories:

- `framework/BUILD.gn`: shared runtime and executable entries, resource repacking,
  distribution, and the aggregate native test entry.
- `public/BUILD.gn`: the public C ABI header target.
- `common/BUILD.gn`: manifest/content metadata and capability matching.
- `browser/BUILD.gn`: browser integration, service implementations, and Shell API
  handlers; the existing browser/handler dependency boundary remains intact.
- `browser/services/BUILD.gn`: service interface contracts.
- `app/BUILD.gn`: Application composition and host ABI bridge contracts.
- `renderer/BUILD.gn` and `utility/BUILD.gn`: process-specific clients/runtime.
- `test/BUILD.gn`: framework integration and service tests.

External build/package entry labels remain `//tuffite/framework:distribution`
and `//tuffite/framework:unittests`. Internal targets reference their owning
subdirectories directly; there are no forwarding groups for old internal labels.

Final directory/build split verification (2026-09-19): Shell API handlers live
under `framework/browser/services/shell_api/commands`, and application composition/integration lives
under `framework/app`. All native include paths were checked against source or
generated headers. `gn check out/TuffiteRelease "//tuffite/framework/*"` passed;
the existing TuffiteRelease graph successfully compiled and linked distribution
and native test targets. All 22 native tests passed. Distribution packaging
verified 18 runtime files, and the rebuilt Playground package passed all 22
multi-process smoke checks and exited normally with status 0.

## License

Tuffite-owned code is licensed under [MIT OR Apache-2.0](LICENSE). Chromium and other
third-party code retain their original licenses and copyright notices. See
[third-party notices](THIRD_PARTY_NOTICES.md).

Public registry version `0.0.0` is a placeholder, not the current SDK. Use the
source workspace and Playground until matching SDK crates and Framework
Distributions are published. See [public release preparation](docs/public-release.md).
