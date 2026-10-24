# Application-defined utility services

## 1. Purpose

Tuffite lets an application run selected Rust workloads in Chromium utility
processes. Typical uses include document parsing, indexing, codecs, native
library isolation, and CPU-heavy work that should not execute on the browser UI
sequence.

This is a managed framework capability, not permission to construct arbitrary
Chromium command lines. Business renderer code never launches a process
directly, chooses an executable, or binds a raw Mojo interface.

## 2. Trust and registration

A service is available only when both declarations agree:

1. the Rust application statically registers a concrete implementation;
2. the signed application manifest allowlists the same stable service name and
   sandbox class.

Rust registration:

```rust
use tuffite::{
    ApplicationResult, CommandError, CommandResult,
    UtilityRequest, Utility,
};

struct ProjectIndexer;

#[tuffite::utility(name = "project_indexer")]
impl Utility for ProjectIndexer {
    async fn new(context: tuffite::utility::UtilityContext) -> ApplicationResult<Self> {
        Ok(Self)
    }

    async fn invoke(&self, request: UtilityRequest) -> CommandResult {
        if request.is_cancelled() {
            return Err(CommandError::Failed("cancelled".to_owned()));
        }
        // Decode and return the application's versioned binary protocol.
        Ok(request.payload().to_vec())
    }
}

#[tuffite::main(browser = AppBrowser, utilities(ProjectIndexer))]
fn main() -> tuffite::app::Builder<AppBrowser> {
    tuffite::app::Builder::new()
}
```

The utility attribute defines a stable name and factory; the main macro selects
which implementations are exported and checks duplicate names at compile time.
The definition factory runs separately in each process. It constructs no
process state: browser and utility startup call `Browser::new` and
`Utility::new`, respectively. The signed manifest independently authorizes
launching a registered service. Renderer access is constrained by ShellAPI
origin authorization.

## 3. Process and call path

```text
authorized primary document
  -> ShellAPI binary request
  -> browser origin and manifest checks
  -> UtilityServiceManager
  -> Chromium ServiceProcessHost
  -> generic tuffite.mojom.UtilityService
  -> application callback table in the utility process
  -> registered Rust Utility instance
```

The packaged application executable is relaunched with Chromium's utility-process switches.
It loads the version-matched business runtime library, installs the immutable
application callback table, and enters `ContentMain`. The browser and utility
processes each load the same business runtime library in their own address
space; they never share Rust state.

Mojo transports a byte array and the Rust FFI transports borrowed byte slices.
Payload encoding belongs to a versioned application or generated framework
schema; JSON is not introduced by the transport.

## 4. Current implementation

The browser owns one reusable host per allowlisted service. Each host launches
a sandboxed utility process on demand, permits up to eight concurrent unary
invocations, bounds total active and queued work at 64 requests, and shuts an
idle process down after 30 seconds when it has no live session. Each invocation
has a non-zero request ID, a cancellation token, and a 30-second deadline. The
Rust callback returns immediately after accepting work; one one-shot reply is
delivered later without blocking the Chromium utility sequence. A browser
timeout requests cooperative cancellation and tears down the unhealthy process.
A disconnect fails active work exactly once and never replays it implicitly;
queued work waits for exponential restart backoff. Five consecutive crashes
open a 30-second circuit, while a successful invocation resets crash history.

This gives the demo real process isolation and supervised Rust-to-Mojo-to-Rust
routing. `window.tuffite.utility.invoke(name, requestId, Uint8Array)` and
`window.tuffite.utility.cancel(name, requestId)` are generated from the
built-in TypeScript contract and uses the same
`window.tuffite.invoke("tuffite.utility.invoke", payload)` Mojo route as every
other built-in API. There is no fetch URL bridge or JavaScript special case.

`Utility` is `Send + Sync`, and unary `invoke` borrows `&self`; a service
uses atomics or its own fine-grained locks for shared state. The framework does
not hold one service-wide mutex across calls. Long-lived session commands remain
serialized within each session so adapter state and event order are predictable;
sibling sessions and unary calls can make progress independently.

The Playground `EchoUtility` owns an invocation counter. Its smoke test
performs two calls and requires responses numbered 1 and 2, demonstrating that
the host reuses the same service instance and preserves its state. The service
also verifies delayed asynchronous completion, two concurrent delayed calls,
and cooperative cancellation. It implements `shutdown()` as the explicit hook
for flushing state and releasing resources when the supervisor retires the
process.

## 5. Extended service model

The browser-side manager will own named instances instead of exposing raw
processes. The public concepts are:

- service definition: stable name, protocol version, sandbox profile, limits,
  restart policy, and whether instances are singleton, per-window, or pooled;
- service instance: opaque generation-aware ID and lifecycle state;
- request: ID, deadline, cancellation token, bytes or data-pipe attachments;
- health: starting, ready, draining, stopped, crashed, or backoff;
- policy: maximum processes, in-flight requests, queue depth, memory budget,
  idle timeout, and restart window.

Current safe Rust shape (the public trait requires `Send` futures; implementations
can use `async fn` directly):

```rust
pub trait Utility: Send + Sync + 'static {
    fn new(context: UtilityContext)
        -> impl Future<Output = ApplicationResult<Self>> + Send where Self: Sized;
    fn invoke(&self, request: UtilityRequest)
        -> impl Future<Output = CommandResult> + Send;
    fn create_session(&self, request: UtilitySessionRequest)
        -> ApplicationResult<Box<dyn UtilitySession>>;
    fn shutdown(&self, context: ShutdownContext)
        -> impl Future<Output = ApplicationResult<()>> + Send;
}
```

The child reports Ready only after asynchronous construction succeeds. There is
no separate Utility `initialize` hook. Runtime adapters erase concrete futures
internally; applications do not use `UtilityFuture` or `LifecycleFuture`.

`UtilityContext` owns the service name and service-local startup configuration,
and reports shutdown through `is_stopping()` or the awaitable
`stop_signal().cancelled()`. Put business state in the service
instance; session init and session state remain separate. Declare startup JSON in
`app.utilities.<name>.configuration` (64 KiB serialized maximum). ABI 0.1.0 copies
only that configuration over Mojo when a child starts, including after restart.
An omitted configuration yields empty bytes. This map configures services already
registered in main; it cannot register a new service or share Browser pointers.

`UtilityRequest` owns its payload and exposes cooperative cancellation plus the
host deadline, so no borrowed request pointer outlives the ABI call. Async here
means asynchronous request completion. Unary calls are admitted with bounded
service-level concurrency (eight active and 64 active plus queued); they share a
read lock only as a lifecycle barrier and are not serialized by the framework.
An implementation that mutates shared state must use atomics or a suitably
fine-grained lock. `shutdown(context)` runs after admission closes and admitted calls finish.
Its future has a 30-second cleanup budget, separate from the ordinary-work stop
signal. The supervisor can terminate a child that does not cooperate; Rust code
blocking inside a future poll cannot be safely interrupted in-process. Implementations must yield or check cancellation during long work.
Responses remain bounded to 1 MiB. Large or continuous payloads require a
service-specific Mojo shared-buffer/data-pipe contract with explicit
backpressure; they are not encoded as application events or a generic ShellAPI
`AsyncIterable`.

Long-lived stateful workloads use a separate Utility session contract instead
of weakening these unary guarantees. A session has explicit create/send/cancel/
close operations and an ordered, credit-controlled event sink. The trusted Rust
host acknowledges events only after validation and any required persistence; it
may then project a revision/delta through best-effort renderer application
events. See [Long-lived utility sessions](utility-sessions.md).
The Rust/C ABI, Mojo transport, Host API, restricted V8 plugin loader, and
long-running Playground fixture are implemented.

## 6. Security invariants

- only primary documents whose committed origins are explicitly authorized may
  request a service; an Explorer or remote page is denied unless the business
  manifest deliberately grants that origin;
- both static Rust registration and manifest policy are required;
- service names, payload sizes, concurrency, deadlines, and output sizes are
  bounded before work is accepted;
- the browser owns launch, receiver binding, cancellation, and termination;
- utility crashes never cause automatic unbounded restart loops;
- sandbox relaxation is an explicit per-service reviewed capability, not an
  argument supplied by JavaScript;
- service handles are generation-aware and stale handles fail closed;
- panics and malformed replies are contained at the FFI boundary;
- logs and crash reports identify service and request IDs without recording
  sensitive payload bytes by default.

## 7. Production work still required

Before this API is stable, add configurable timeout/restart policies and pools,
per-service memory/CPU limits, dedicated attachment pipes where concrete
services need them, platform sandbox profiles, metrics/tracing, and integration
tests covering renderer revocation, non-cooperative cancellation, timeouts,
utility crashes, backoff, cross-process cancellation, and circuit recovery.
Duplicate request-ID rejection, bounded unary concurrency, long-lived sessions,
credit exhaustion, acknowledgement, and the 30-second idle boundary already
have implementation coverage.

### Startup configuration builders

Rust Utility and V8 implementations may override the synchronous static hook
`fn setup(builder: Builder) -> ApplicationResult<Builder>`. Use `utility::Builder`
for Rust and `utility::v8::Builder` for V8. Framework supplies the named service's
manifest configuration; `configuration_bytes()` reads it and the consuming
`configuration(bytes)` method replaces service-local data. Framework validates
the 64 KiB limit and freezes the returned builder before constructing state.
V8 defaults to `entry.mjs`; select another module using `builder.entry(path)`.
Entry validation runs before provider construction, once per lazy provider.
These setup methods do not start services, perform async initialization, or grant
capabilities. Manifest permissions remain enforced by native policy.
