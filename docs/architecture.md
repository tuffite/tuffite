# Tuffite architecture

## 1. Goals

Tuffite provides a reusable Chromium distribution and Rust application model
for cross-platform desktop applications. Its primary goals are:

- Rust owns application behavior and lifecycle.
- Renderer pages use web technology and a typed, binary ShellAPI.
- Chromium is embedded through the `content` layer rather than the complete
  Chrome UI layer.
- `<explorer>` embeds ordinary `WebContents` without GuestView and participates
  in Chromium's normal surface composition.
- Applications consume a prebuilt, versioned Runtime instead of compiling
  Chromium for each application.
- Security decisions are explicit capabilities associated with a document and
  a `WebContents` role.
- Framework-specific Chromium patches remain small, buildflag-isolated, and as
  close as possible to upstream architecture.

## 2. Non-goals

- Reimplementing Chromium in Rust.
- Exposing `content::WebContents*` or any unstable C++ ABI to Rust.
- Providing arbitrary Node.js APIs in renderer processes.
- Treating every web page as trusted application UI.
- Making `<explorer>` a compatibility alias for Electron `<webview>`.
- Supporting ABI-stable dynamically loaded Rust extensions in the first release.
- Using CEF as the central abstraction. Tuffite needs Chromium integration that
  CEF does not expose at the required ownership and feature boundaries.

## 3. Component model

```text
Tuffite Framework
├── Framework Distribution and CLI
├── Tuffite Runtime (Rust control plane)
│   ├── application lifecycle
│   ├── window and WebContents model
│   ├── command router
│   ├── capability manager
│   ├── ShellAPI Host
│   ├── Network Broker
│   ├── resource resolver
│   ├── extension runtime
│   └── async task supervision
├── Tuffite Framework (C++)
│   ├── Chromium bootstrap and shutdown
│   ├── BrowserContext and StoragePartition ownership
│   ├── WebContents, Views, menus, and platform windows
│   ├── Mojo registration and renderer bindings
│   ├── Chromium Network Service adapter
│   ├── <explorer> browser integration
│   └── Runtime event adaptation
├── Tuffite FFI (stable C ABI)
├── custom Chromium
│   ├── Blink ShellAPI binding
│   ├── <explorer> element
│   └── buildflag-isolated upstream hooks
└── packaging, signing, and updater tooling
```

ShellAPI is not a peer execution layer. It is the renderer-facing interface
implemented by Runtime. The Shell Protocol remains a small independent package
so the renderer binding, macro expansion, schema generator, and protocol tests
do not depend on all of Runtime.

## 4. Process model

```text
Browser/control process
┌────────────────────────────────────────────────────────────┐
│ business-named executable (native sandbox/bootstrap)       │
│   Tuffite Framework / Chromium                               │
│       ⇅ Tuffite FFI                                        │
│   business Rust runtime DLL                                │
└────────────────────────────────────────────────────────────┘
                  │ Mojo / Chromium IPC
       ┌──────────┼───────────────┐
       ▼          ▼               ▼
 renderer      renderer       utility/GPU/network
 host UI       explorer       Chromium processes
 Shell Binding no ShellAPI
```

The business crate exports its registered application as a Rust `cdylib`.
Production packaging installs the native `tuffite_launcher` under the business executable
name. The application-named executable owns Chromium's Windows sandbox, loads
the business runtime, obtains its callback table, and enters `ContentMain`. Renderer,
GPU, network, and utility child launches re-enter that same business-named image.
Chromium C++ types never cross the C ABI.

The initial architecture keeps Rust Runtime in the browser process. A future
out-of-process Runtime would add failure isolation but also adds latency,
serialization, startup coordination, and another security boundary. It should
only be introduced after measurements justify it.

## 5. Threading and ownership

### 5.1 C++ ownership

The Framework exclusively owns and accesses Chromium C++ objects, including:

- `BrowserContext` and `StoragePartition`;
- `WebContents`, `RenderFrameHost`, and `NavigationController`;
- Views widgets, native windows, menus, and dialogs;
- Mojo remotes, receivers, and data-pipe endpoints tied to Chromium sequences.

Every operation that touches those objects is executed on its required Chromium
sequence. FFI calls enqueue work rather than allowing Rust to call Chromium
objects from arbitrary executor threads.

The Chromium UI sequence remains owned by Chromium's message loop. Runtime may
run an async executor on dedicated Rust threads, but it must not block the UI
sequence while waiting for a Rust future. Requests crossing either direction are
queued, carry request IDs, and complete asynchronously. Shutdown stops accepting
new work, cancels Runtime tasks, drains required UI-sequence destruction, and
only then releases BrowserContext and Chromium process state.

### 5.2 Rust ownership

Runtime owns application state, command implementations, capability policy,
task cancellation, extension instances, and the public Rust API. Rust objects never
contain raw Chromium pointers.

### 5.3 Handles

Cross-boundary objects use typed, generation-aware opaque handles. A handle can
be represented over the ABI as a `u64` containing a slot and generation, while
the safe Rust API wraps it in distinct newtypes such as `WindowId` and
`WebContentsId`.

Required properties:

- stale generations fail deterministically;
- a handle's kind is validated before lookup;
- destruction is idempotent at the request boundary;
- callbacks include the relevant handle and generation;
- no address, Chromium routing ID, or pointer becomes a public identity.

## 6. Tuffite FFI

FFI is a narrow transport boundary, not a feature layer. It exposes a stable C
ABI and keeps all C++ implementation details private.

Allowed ABI shapes:

- fixed-width integers and explicit enums;
- opaque `u64` handles;
- borrowed byte slices valid only for the duration of a call;
- owned buffers with an explicit release function;
- request IDs plus asynchronous completion events;
- versioned function tables negotiated during initialization.

Disallowed ABI shapes:

- C++ classes, STL types, exceptions, or references;
- Rust layout-dependent structs or trait objects;
- callbacks that may arrive on undocumented arbitrary threads;
- synchronous calls that re-enter Rust while holding Chromium locks;
- ownership inferred from naming rather than encoded in the API.

Every exported Rust entry point catches panics before returning to C++. Every C++
entry point catches implementation failures and converts them to ABI error codes
or asynchronous failure events. Neither Rust unwinding nor C++ exceptions may
cross Tuffite FFI. Production panic policy must be explicit: application-command
panics become failed commands where containment is sound, while corruption or a
panic in core Runtime may terminate the process and rely on crash recovery.

A representative bootstrap shape is:

```c
typedef struct tuffite_application {
  tuffite_abi_header header;
  tuffite_application_start_fn start;
  tuffite_application_dispatch_fn dispatch;
  tuffite_application_stop_fn stop;
  tuffite_utility_service utility;
  tuffite_prepare_browser_fn prepare_browser; // legacy optional ABI 2.1
  tuffite_prepare_process_fn prepare_process; // optional ABI 2.3
  tuffite_utility_runtime_fn utility_runtime; // optional ABI 2.3
} tuffite_application;

const tuffite_application* tuffite_get_application(void);
```

The exact API is generated from one ABI definition and verified in C++ and Rust
CI. The public Rust API sits above `tuffite-abi`; applications should not depend
on the FFI crate directly.

## 7. Application-defined utility services

Business code can register stateful Rust `Utility` implementations. A
service is invocable only when both the compiled application registration and
the validated `tuffite.json` allowlist contain its stable name. The browser
process launches the generic Mojo interface through `ServiceProcessHost`; the
implementation runs inside Chromium's utility sandbox and dispatches through
the process-local application callback table. Renderer code never chooses a
process type, executable path, or raw Mojo interface.

The browser owns one supervised host per service name. A healthy process is
reused, unary requests run with bounded concurrency (eight active and 64 active
plus queued), and an idle process is
closed after 30 seconds. A disconnect or 30-second invocation timeout fails the
active request exactly once: work already delivered to another process is never
replayed implicitly because it may have produced side effects. Requests that
were still queued may continue after an exponential restart backoff. Five
consecutive crashes open a 30-second circuit and reject queued/new work with a
resource-exhausted result. A successful invocation resets crash history.

Applications that need retry must choose it explicitly and should provide an
idempotency key in their typed request. Future policy can make timeout, queue,
concurrency, and idle values manifest-configurable without changing the generic
Mojo service contract.

Long-lived workloads use the separate Utility session contract rather than a
large timeout on `Invoke`. Sessions add bidirectional commands, ordered events,
credits/acknowledgement, health, and explicit close while retaining the same
supervised Chromium process boundary. See
[Long-lived utility sessions](utility-sessions.md).
Framework-owned restricted JavaScript extension sessions reuse this transport with
one bounded V8 isolate per session; see
[Restricted JavaScript extension runtime](javascript-extension-runtime.md).

## 8. Application model

The logical application main explicitly assembles process definitions:

```text
#[tuffite::main(browser = B, utilities(U1, U2))]
└── tuffite::app::Builder<B>
    ├── selected browser metadata from #[tuffite::browser]
    ├── commands for B
    ├── before_startup hook with process_kind and argv
    └── named utility metadata from #[tuffite::utility]
        └── tuffite_get_application(): immutable ABI table
```

There is no safe Rust Application trait or setup registration handle.
The definition factory runs once in each process loading the business library;
it must only assemble definitions. Browser startup creates `Browser` state,
and utility startup creates only the selected `Utility` instance. Their
states and globals live in independent address spaces.

```rust
use tuffite::{Browser, BrowserContext, ApplicationResult, CommandError};

struct ProjectBrowser;

#[tuffite::browser]
impl Browser for ProjectBrowser {
    fn new(context: BrowserContext) -> ApplicationResult<Self> {
        let _manifest = context.configuration();
        Ok(Self)
    }
}

#[tuffite::command(shell_api::project::Open)]
fn open_project(browser: &mut ProjectBrowser, path: String)
    -> Result<(), CommandError> {
    Ok(())
}

#[tuffite::main(browser = ProjectBrowser)]
fn main() -> tuffite::app::Builder<ProjectBrowser> {
    tuffite::app::Builder::new()
        .commands(tuffite::generate_handler![open_project])
        .before_startup(|launch| {
            launch.argv_mut().retain(|arg| arg != "--business-debug");
            Ok(())
        })
}
```

Application crates emit a business runtime `cdylib`. The native launcher owns
the OS main, loads the library and validates its ABI table. ABI 2.3 adds an
optional startup hook for every process role before ContentMain. It exposes
process_kind and argv; changing the executable or process role is rejected. Sandbox initialization
remains native-owned. See the [ABI lifecycle guide](../crates/tuffite-abi/README.md).

`BrowserContext` provides the validated manifest and trusted native capabilities.
Generated commands take mutable browser state and execute serially on a Rust
worker. Without a generated table, `app::Builder` uses `Browser::dispatch`.
Rust utility declarations use `#[tuffite::utility(name = "stable_name")]` on a
concrete Utility impl; only types listed in the main macro are exported.

JavaScript uses a native V8 backend declared with `utility::V8` and the same
`#[tuffite::utility(name = "parser")]` attribute. `V8::entry` declares a relative entry or HTTPS/file URL. `V8::resolve(url)`
returns Result<Option<ModuleSource>, Error> on supervised Browser workers.
Some overrides resources; None (the default) falls back to app.assets; Err
fails creation. Relative paths use the app scheme and a service.utility host.
Native asynchronously requests static imports over Mojo through ABI 2.4,
caches compiled modules per session, then links synchronously. Both Rust and
V8 types are listed in main. Browser opens the named entry through
`context.utilities().create_v8(...)`, and the utility process selects its backend
using `Application.utility_runtime(name)` before session creation.

`.before_startup` runs for every process role before Chromium ContentMain, after
OS process creation. Native supplies read-only `ProcessKind` and editable argv,
and rejects executable or role changes. The utility service name is selected
later by Mojo Start(name). ABI 2.3 retains the browser-only ABI 2.1 hook for older
providers.


## 9. WebContents roles

Every `WebContents` has an immutable role used in authorization:

| Role | Purpose | ShellAPI default |
| --- | --- | --- |
| `App` | Trusted application UI loaded from its manifest scheme | Eligible, subject to capabilities |
| `Explorer` | Embedded browsing surface created by `<explorer>` | Denied unless its committed origin is explicitly authorized |
| `DevTools` | Chromium DevTools UI | Denied except explicit internal tooling |
| `Auth` | Isolated login or identity flow | Denied |
| `Utility` | Framework-controlled hidden content | Denied unless a narrow internal policy permits it |

ShellAPI authorization requires a live primary document identity, an allowed
committed origin, and a declared capability. The browser enforces this for each
outer or Explorer `WebContents`; renderer-side injection is only an early
visibility filter. No entry grant is implicit. Each origin needs a matching
`app.security.capabilities` rule for the requested operation.

## 10. Application resources

Packaged application resources use a validated, secure application scheme rather
than `file://`. The scheme handler must:

- canonicalize and validate paths before lookup;
- reject traversal and ambiguous encoded paths;
- provide deterministic MIME types and CSP headers;
- support immutable hashes for production assets;
- separate development proxy behavior from production packages;
- assign application origins intentionally, for example `playground://main/`.

The current implementation serves arbitrary regular assets under the fixed,
validated package `assets` directory, rejects absolute/parent/encoded-separator paths, derives MIME types,
and adds no-sniff and a restrictive default CSP. Production packaging can add
immutable per-asset hashes without changing the application URL contract.

Remote HTTP content is never promoted to the `App` role merely because a local
page navigated to it. Navigation policy either blocks the transition, opens a
new `Explorer`, or creates a new non-privileged WebContents.

Runtime-originated and privileged renderer HTTP requests use Tuffite's Network
Broker and Chromium Network Service. See [Networking](networking.md). Ordinary
renderer `fetch()` continues to use standard web-platform behavior; it is not
silently replaced by ShellAPI.

Top-level windows are browser-owned objects addressed by opaque process-local
IDs. The `window.tuffite.window` facade is generated from the same builtin
TypeScript contract as networking and utility calls, and every operation is
checked against `app.security.capabilities[].shellApi`. An authorized Explorer resolves to its
outermost application window when an ID is omitted. See
[Window management](window-management.md).

## 11. Repository layout

Target top-level layout:

```text
crates/
  tuffite/                   public API plus optional build and CLI features
  tuffite-runtime/           Rust control plane, handles, and ShellAPI Host
  tuffite-macros/            #[tuffite::command(endpoint)] handler bindings
  tuffite-abi/               internal C ABI bindings
src/tuffite/                 Chromium-side customization root
  framework/
    abi.h
    explorer/               browser host, Mojo contract, and tests
    shell_api/              browser host, renderer binding, and Mojo contract
    utility/                host, client, service, and Mojo contract
    window/                 window manager and platform adapters
    app/
    browser/
    common/
    renderer/
  third_party/
    blink/renderer/core/    sources compiled directly into Blink core
    blink/renderer/bindings/ build-list integration for generated bindings
  network/
    browser/                 Network Service adapter and policy enforcement
    common/mojom/            request, response, and stream contracts
  utility/
    utility_service_host.*   ServiceProcessHost lifecycle manager
    utility_service.mojom    generic binary service contract
    utility_service.*        utility-process implementation
tools/
  protocol/
  packaging/
playground/              complete Rust + renderer + packaging example
```

Generation-aware handles live in `tuffite-runtime`; a separate foundation crate
is not justified until multiple independently versioned consumers need it.

## 12. Chromium customization policy

- Feature implementations live under `src/tuffite/<feature>/`.
- Small, tightly coupled features stay flat; add layer subdirectories only
  when their size or independent ownership justifies the hierarchy.
- Upstream Chromium files contain only minimal construction, registration, or
  build hooks.
- Every behavior-changing hook is guarded by the generated Tuffite buildflag.
- A disabled build must omit feature sources, generated bindings, Mojo targets,
  registrations, and behavioral changes from the Ninja graph.
- Downstream Chromium patch regions that cannot be fully conditionally compiled
  are bracketed with `TUFFITE_BEGIN` / `TUFFITE_END` markers.

The integration buildflag is:

```text
TUFFITE_IN_CHROMIUM
```

Its GN arg is `tuffite_in_chromium`. Explorer is compiled as part of that
integration and exposed per application through the `TuffiteExplorer` Blink
runtime feature plus manifest origin policy.

### Utility contexts and state ownership

`Utility::new(UtilityContext)` constructs service state in the child. Only the
selected `app.utilities.<name>.configuration` JSON crosses Mojo/ABI 2.5 (64 KiB
maximum); request payloads and session init remain separate. Context clones own
the configuration and observe service stopping. Restart reconstructs child state.

`V8::new(V8Context)` constructs one Browser-side resource provider lazily per
name per Browser runtime. `entry(&self)` and `resolve(&self, url)` can use its
instance fields. The context exposes service configuration, the Browser manifest,
executor, typed shared state, and stopping. `BrowserContext::set_state(Arc<T>)`
shares application state with provider `state::<T>()` inside Browser only.
Concurrent provider calls require synchronized mutable fields. Each JS session
has a distinct Utility-side isolate; session-specific resolver state is not
available through the service/name/URL callback.
