# Tuffite Playground

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
Business handlers bind their generated endpoint with
`#[tuffite::command(shell_api::...::Endpoint)]`, and a shared `main()`
explicitly lists the handlers in `tuffite::generate_handler![...]`.
`lib.rs` marks that builder with `#[tuffite::main(browser = PlaygroundBrowser, utilities(echo_utility::EchoUtility, v8_utility::Parser, typed_utility::Worker))]`; packaging installs
the native `tuffite_launcher` under the business executable name. The endpoint attribute keeps
the full channel, stable ID, parameter, and return type checks close to the
function while Builder registration stays concise and contains no strings.

`tuffite.json` declares explicit `app.assets[]` routes, startup `app.windows[]`,
codegen in `build.mojomSources`, runtime settings in `app.network`, and
origin-scoped operation grants in `app.security.capabilities[]`. Artifact and
preview documents share read-only framework grants while keeping independent
origins. The permissions lab also checks business grant union and navigation
revocation. Explorer navigation and network URL scopes support host/path
patterns; filesystem scopes use logical paths such as `data/*`.
`shell_api.mojom` is the business API contract; generated Rust and JavaScript
bindings live in `OUT_DIR` and `generated/`. Hand-written Web sources live in
`ui/`; Vite emits production assets to `ui/dist/`. The reusable framework facade is
in `packages/tuffite`. Utility service availability is registered explicitly by
`#[tuffite::main(..., utilities(echo_utility::EchoUtility, v8_utility::Parser, typed_utility::Worker))]` in `src/lib.rs`; it is not duplicated in the
manifest.

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
CDP on an isolated `mojom-smoke.html` page; the Framework smoke suite includes
this check. `echo.roundTrip` is routed by the native broker to the independent
Rust Worker via generated `Echo` and `Aux` traits. Browser registration uses
`#[tuffite::browser(interfaces(shell_api::Commands, shell_api::Diagnostics))]`;
Utility registration uses `#[tuffite::utility(interfaces(shell_api::Echo, shell_api::Aux))]`
on the Worker struct. Both Utility interfaces share one instance; the codec smoke
checks cross-interface state and a process ID distinct from the Browser. Permissions use the original
`playground.Echo.RoundTrip` spelling. The capability smoke allows that endpoint
while denying `Echo.Restricted` and `Aux.RoundTrip`, and the codec fixture rejects
raw invoke/cancel bypasses, forged IDs and mismatched schemas.

## Rust and V8 utility examples

- [`src/echo_utility.rs`](src/echo_utility.rs) declares the Rust service with
  `#[tuffite::utility(name = "echo")]`; the main attribute lists its type explicitly.
- [`src/v8_utility.rs`](src/v8_utility.rs) declares `#[tuffite::utility(name = "parser")]` on an impl of
  `tuffite::utility::V8`; `setup(builder)` selects the entry through `builder.entry(...)` and `resolve(&self, request)`
  supplies `sdk/generated.mjs`, an HTTPS module URL and a file module URL
  with generated text (without ambient network or file reads) and returns None for the
  entry/runtime sources, which load through an explicit `app.assets` route.
  `bundle.resources` includes that module directory.
  [`entry.mjs`](src/v8_utility/extension/entry.mjs) exports `describe` and `spin`;
  [`runtime.mjs`](src/v8_utility/sdk/runtime.mjs) demonstrates relative ESM imports,
  imports the generated module and inspects the restricted realm.
- [`src/lib.rs`](src/lib.rs) declares `#[tuffite::browser]` and runs the package
  through `BrowserContext::utilities().create_v8("parser", ...)` in the
  `check_javascript_extension` handler. The Playground checks static imports, shared and cyclic dependencies,
  blocked ambient authority and eval, then cancels `spin` through Mojo.

Rust and V8 definitions use the same named utility attribute and are listed
explicitly in main. Their traits (`Utility` and `V8`) select the backend. V8 runs in the Framework-owned native backend selected
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
