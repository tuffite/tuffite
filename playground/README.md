# Tuffite Playground

The [profile composition lab](ui/profiles.html) uses separate
`Application` and `Browser` trait implementations with grouped interface registration. Create a
Work profile and switch the existing window between independently stored page
values and Rust counters. Profiles are loaded on demand. The application
composes `profiles.open`, `browser.views().create` and `windows.set_content`;
the window service has no profile-switch operation. See
[the lifecycle and ownership model](../docs/profiles.md).

`python3 playground/scripts/smoke.py --package dist/playground --profiles`
checks lazy loading, shared application state, profile state/storage/event
isolation, window identity, busy-context rejection and persistent reopening.

This root-level directory is a complete Tuffite application and integration
fixture. It demonstrates typed business commands, a supervised Rust utility
service, multiple windows, Explorer isolation, Network Service requests,
capability-bounded path/filesystem access, application storage, Chromium
logging, and Rust work scheduled through Chromium's ThreadPool.
It also serves a 256 KiB single asset to exercise backpressured app-resource
delivery, awaits asynchronous/cancellable Rust Utility invocations, and
verifies low-rate application events delivered to the renderer. Continuous
byte streams deliberately use dedicated transports such as PTY/DataPipe rather
than a generic ShellAPI `AsyncIterable`.
It also opens a long-lived Utility session with one event credit, acknowledges
the first ordered event in Rust, stays alive beyond the unary service's
30-second idle boundary, receives a second event, and verifies the close
summary. Only the final result is projected to the renderer as an application
event.
The fixture also opens a restricted JavaScript extension session, loads a
two-module in-memory ESM graph, verifies static relative imports and JSON calls,
checks that ambient process/network APIs and `eval` are unavailable, and
cancels an intentionally infinite extension call across Mojo.
The crate root declares the native `Embedding`: its Application, per-profile
Browser and named Utility implementations. Each backend implements generated
ShellAPI traits and registers its own `.interfaces(...)` in setup. Packaging
installs the native `tuffite_launcher` under the business executable name.

`tuffite.json` contains identity, codegen/build inputs and the explicit
`bundle.resources` file inventory. Runtime declarations belong to
[`src/application/configuration.rs`](src/application/configuration.rs): schemes,
resource URL maps, startup windows, network settings and origin-scoped rules.
Artifact and preview documents keep independent origins and restricted grants.
The permissions lab checks grant union, resource scopes and navigation revocation.

`shell_api.mojom` is the business API contract. Generated Rust bindings are exposed
through [`src/shell_api.rs`](src/shell_api.rs); generated JavaScript lives in
`generated/`. Hand-written Web sources live in `ui/`; Vite emits `ui/dist/`.
`ui/public/isolated/` keeps native ESM, CSS modules and images together. Standard
public-resource staging preserves their paths. Rust configuration declares the
packaged directory and entry; no generated Web output is compiled into Rust.
See [isolated UI integration](../docs/isolated.md).

## Rust module layout

```text
src/
  lib.rs                      Native embedding and module declarations
  application.rs              Application state and lifecycle
  application/
    configuration.rs          Runtime schemes, resources, windows and policy
    profiles.rs               Application-scoped Profiles API and window composition
  browser.rs                  Per-profile Browser state and lifecycle
  browser/
    commands.rs               Browser commands and task-pool example
    diagnostics.rs            Profile identity and independent counters
    utility_sessions.rs       Rust Utility session client example
    javascript_sessions.rs    V8 session client example
    isolated_ui.rs            Dynamic registration lifecycle probe
  utilities.rs                Named service module facade
  utilities/
    echo.rs                   Named binary/session Rust service
    worker.rs                 Typed Echo/Aux service implementation
    parser.rs                 Browser-side V8 resource provider
  shell_api.rs                Generated contract facade
  logging.rs                  Shared process tracing setup
  codec_tests.rs              Mojom wire compatibility fixture
ui/
  fixtures/                   Test-only documents; history uses query-selected states
  public/isolated/             Native ESM, CSS modules and image assets
  public/utilities/parser/     V8 service entry and ESM dependencies
```

Application calls import `commands` from `/shell_api.js`; generated bindings no
longer install `window.playground`. `createPlaygroundApi(transport)` creates an
independent client for a custom transport. Framework methods are imported from `/tuffite.js` as an ES module;
no business methods are installed on the native global bridge. `BaseDirectory`
is a numeric enum exported by that module. The native Rust compiler
frontend is the standalone `tuffite-mojom` crate; SDK binding generators live in
`tuffite::build` and are shared by the CLI and Cargo build helper.

The contract imports `mojom_data.mojom` to exercise Mojom v0 field packing,
64-bit integers, nested structs, UTF-8 strings and byte arrays. Startup checks
typed JavaScript calls into Browser Rust and reuses the same generated data
codec in Utility Rust. `python3 playground/scripts/smoke.py --package
dist/playground --mojom` verifies those calls and numeric boundaries through
CDP on an isolated `fixtures/mojom.html` page; the Framework smoke suite includes
this check. `echo.roundTrip` is routed by the native broker to the independent
Rust Worker via generated `Echo` and `Aux` traits. Browser setup uses `.interfaces(...)` with generated
`shell_api::interfaces` markers;
Utility setup uses the same `.interfaces(...)` chain on `utility::Builder<Self>`. Both Utility interfaces share one instance; the codec smoke
checks cross-interface state and a process ID distinct from the Browser. Permissions use the original
`playground.Echo.RoundTrip` spelling. The capability smoke allows that endpoint
while denying `Echo.Restricted` and `Aux.RoundTrip`, and the codec fixture rejects
raw invoke/cancel bypasses, forged IDs and mismatched schemas.

## Rust and V8 utility examples

- [`src/utilities/echo.rs`](src/utilities/echo.rs) declares the Rust service with
  `Utility::setup(builder.name("echo"))`; the embedding lists its type explicitly.
- [`src/utilities/parser.rs`](src/utilities/parser.rs) declares `V8::setup(builder.name("parser"))` on an impl of
  `tuffite::utility::V8`; `setup(builder)` selects the entry through `builder.entry(...)` and `resolve(&self, request)`
  supplies `sdk/generated.mjs`, an HTTPS module URL and a file module URL
  with generated text (without ambient network or file reads) and returns None for the
  entry/runtime sources, which load through an explicit `app.assets` route.
  `bundle.resources` includes that module directory.
  [`entry.mjs`](ui/public/utilities/parser/extension/entry.mjs) exports `describe` and `spin`;
  [`runtime.mjs`](ui/public/utilities/parser/sdk/runtime.mjs) demonstrates relative ESM imports,
  imports the generated module and inspects the restricted realm.
- [`src/browser.rs`](src/browser.rs) implements `tuffite::Browser`.
  [`javascript_sessions.rs`](src/browser/javascript_sessions.rs) runs the example
  through `BrowserContext::utilities().create_v8("parser", ...)` in the
  `check_javascript_extension` handler. The Playground checks static imports, shared and cyclic dependencies,
  blocked ambient authority and eval, then cancels `spin` through Mojo.

Rust and V8 definitions use the same named utility attribute and are listed
explicitly in the embedding. Their traits (`Utility` and `V8`) select the backend. V8 runs in the Framework-owned native backend selected
by the registered service name. Parser is a reusable Browser-side Rust provider
constructed with V8Context; its generated module uses service configuration and
typed state installed by BrowserContext::set_state. Each session has an independent
JS isolate. Echo receives and validates its own UtilityContext configuration over
Mojo; it retains that context and observes service stop. The smoke fails if either
context path is missing. Both examples start supervised Chromium utility processes on demand.
The concurrency probe requires two unary calls to rendezvous in the same service;
it does not infer serialization from elapsed time. The smoke runner overrides
only its deployed copy of the Explorer background for a stable visual probe,
independent of interactive light/dark themes.

Rebuild the local Distribution before running the Playground smoke suite after
ABI changes.

## Framework developer workflow

From the Framework checkout (`src/tuffite`) with the outer Mise environment active:

```console
python3 x.py build          # Local native Distribution + Web assets + Rust library
python3 x.py build --run    # Refresh local native Distribution and launch Vite HMR
cargo run --package tuffite --bin tuff -- package --manifest playground/tuffite.json
python3 x.py test --suite smoke
python3 x.py test --suite explorer
```

The development command reuses incremental native/Cargo builds, generates
bindings, refreshes the development package, starts Vite, and launches the native
application. UI edits use HMR. Rerun after Framework/native changes to incorporate
them. Application commands receive the local Distribution explicitly and cannot
silently fall back to a downloaded Framework.

For application-only use with an explicitly selected prebuilt Framework,
`cargo tuff build playground/tuffite.json` and
`cargo tuff dev --manifest playground/tuffite.json` remain available.

See [Framework development](../docs/development.md) for profiles, CI, packaging,
patch workflows, and recoverable dependency rolls.

## Isolated Web Components

Rust application setup declares the packaged ESM snapshots. Browser
initialization separately exercises dynamic registration, replacement and
stale-generation rejection. Runtime declarations stay out of `tuffite.json`.
Open `isolated.html` from the main lab to run two isolated worlds with the same
component tags. See [DOM isolated worlds](../docs/isolated.md) for the contract
and `--isolated-worlds` packaged smoke command.
