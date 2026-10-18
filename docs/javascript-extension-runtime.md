# Restricted JavaScript extension runtime

## Purpose

Tuffite provides a framework-owned runtime for small JavaScript/TypeScript
protocol adapters. It is intended for Agent drivers, message-channel adapters,
and similar frequently changing integrations. It is not a general Node.js
runtime and does not replace trusted Rust services.

Each extension runs as a long-lived `UtilitySession` in a Chromium Utility process.
The browser backend remains the policy authority and owns every privileged resource.
Its `BrowserContext::utilities().create_v8(name, ...)` opens a named definition
registered with `#[tuffite::utility(name = "parser")]` on an impl of
`tuffite::utility::V8`. `new(V8Context)` constructs a Browser-side provider on first use.
`entry(&self)` returns the logical entry path and
`resolve(&self, url)` returns `Result<Option<utility::v8::ModuleSource>, utility::v8::Error>`.
The native utility loader requests its entry and discovered static imports over
Mojo. Browser dispatches each canonical application, HTTPS or file URL to a supervised Rust worker
through ABI 0.1.0 `Application.resolve_v8`. ModuleSource accepts embedded, owned
file-loaded or generated source text. Sources are copied and compiled once per
session; utility processes never open these files themselves. Definitions are
listed explicitly in main, alongside Rust services. Some(source) overrides resources; None falls back to explicit app.assets
Origin/path routes; Err fails without fallback. The default resolve returns None.
Fallback uses the shared packaged resource path resolver and bounded native
worker reads. HTTPS/file require a resolver response and never load ambient I/O.
The native Framework selects V8
through `Application.utility_runtime(name)`. Rust entries implement `Utility`
and V8 resource definitions implement `V8`; they share the registration model. The older
javascript_extensions accessor is deprecated.

## Provider context and state

One Rust provider per registered name is reused within a Browser runtime; each JS
session still gets an independent native isolate. `V8Context` provides the name,
service-local configuration from `app.utilities.<name>.configuration`, the full
application manifest, Browser executor, and a cooperative stopping signal.
`BrowserContext::set_state(Arc<T>)` installs typed state retrieved with
`V8Context::state::<T>()`; these Arcs stay in Browser. Provider methods can run
concurrently and constructor failure is retained for that runtime. Session init
is independent; the resolver currently has no session-specific context.

## Host boundary

The Browser supplies only explicitly registered resources. `resolve(url)` must
map logical names to trusted embedded sources or an application-controlled store.
If reading a directory, the application must enforce its canonical root and
symlink policy. The Framework rejects invalid or foreign resource URLs before invoking the
resolver. Resolver errors or panic fail creation. Browser stop joins admitted
resource workers before unloading their function code.

The Utility process receives service-scoped resource URLs and source text only. It
cannot discover or open host files. The V8 realm exposes standard ECMAScript
objects but no DOM, `process`, `require`, `fetch`, raw sockets, environment,
database, secrets, or OS handles.

## Module and call contract

The new session init envelope is JSON with `schemaVersion: 2` and `entry`.
The legacy schemaVersion 1 envelope with `modules` remains supported.
Module identities are canonical application-scheme, HTTPS or file URLs.
Relative entry paths use `<app-scheme>://<name>.utility/`. Static relative and
absolute URL imports are normalized before resources are requested. Compilation discovers the
graph asynchronously; synchronous V8 linking only reads the compiled cache. Bare package specifiers, unsupported schemes, import attributes and dynamic
`import()` are rejected. HTTPS/file module URLs are passed to the application
resolver, which decides whether and how to retrieve each resource. String code
generation (`eval` and `Function`) and WebAssembly generation are disabled.

The API v1 command envelope is `{ "method": string, "args": JsonValue[] }`.
The method is looked up on the entry module's object-valued default export and
must return a JSON-serializable non-Promise value. Async work is represented as
declarative Host operations; Rust performs it and delivers the result in a
later command. This avoids embedding another event loop or granting ambient I/O.

## Resource and concurrency policy

- one V8 isolate and one sequenced worker per extension session;
- 32 MiB V8 heap target per realm;
- a 30-second startup timeout, one request per module URL per session;
- at most 64 modules, 512 KiB per module, and 1 MiB per module graph;
- at most 1 MiB per command/response and 16 JSON arguments;
- at most 16 live sessions per Utility service and 64 outstanding commands per
  session;
- commands are ordered within one session; sibling sessions can run in
  parallel;
- `UtilitySession::Cancel(command_id)` interrupts active V8 execution with
  `TerminateExecution` and marks queued commands cancelled;
- close revokes new work, interrupts active execution, drains the sequence, and
  destroys the realm on its owning sequence.

The current API deliberately rejects Promise-returning methods. If future
extensions require in-realm promises, the runtime must add an explicit bounded
microtask/timer policy rather than silently inheriting browser behavior.

## Verification

The packaged Playground fixture defines a seven-module graph with generated, shared, cyclic and HTTPS/file dependencies, resolves a static
relative import, invokes a default-export method, round-trips JSON, verifies
that `process`, `require`, and `fetch` are absent, verifies that `eval` throws,
then invokes an infinite loop and cancels it across Mojo. Run the smoke with a rebuilt ABI 0.1.0 Distribution to verify on-demand loading.
The same smoke also
proves bounded concurrent unary Utility calls and a long-lived credit/ACK
session across the 30-second idle boundary.
