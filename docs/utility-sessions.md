# Long-lived utility sessions

## 1. Decision and current status

Tuffite Utility processes will support two distinct workloads:

1. `invoke`: the existing bounded unary request/response API for finite work;
2. `UtilitySession`: a long-lived, bidirectional, supervised service instance for
   extension runtimes, language services, parsers with incremental state, and other
   process-isolated workloads.

A session is not a long unary invocation. It has an explicit create handshake, a
generation-aware handle, independent commands and ordered events, credit-based
flow control, cancellation, heartbeat/health, and a terminal close result. Its
lifetime is controlled by policy rather than the unary 30-second deadline.

Playground demonstrates a JavaScript extension in a sandboxed Utility process,
with a trusted Rust backend supervising its session. Application backends own
processes, APIs, permissions, sandbox grants, secret references, and durable state.

The safe Rust Host/service APIs, FFI 2.0 callback tables, Mojo session wiring,
browser/Utility supervision, credit/sequence flow controller, and restricted
V8 ESM loader are implemented. Playground validates a real cross-process
session that emits one
event, waits for Rust acknowledgement, remains alive past the unary 30-second
idle boundary, emits a second event, and closes with an ordered summary. The
Playground also executes a two-module extension fixture with static relative
imports and verifies that `eval`, Node globals, and direct network access are
unavailable.

## 2. Process and ownership model

```text
trusted Rust application / DriverHost
  - authenticates principal and extension generation
  - owns OS resources and durable state
  - validates every host API operation
               | commands / credits / close
               v
browser UtilitySessionHost
  - launches and supervises Chromium utility process
  - bounds queues, frames, memory credits, restart policy
               | versioned Mojo session protocol
               v
sandboxed Utility process
  - runs registered Rust service or restricted JS adapter runtime
  - owns only in-memory adapter state
  - emits ordered session events
```

The Utility process is an isolation and execution boundary. It is not the policy
authority. An extension manifest describes requested capabilities, but Rust grants
the intersection of that declaration, application policy, workspace policy,
user approval, and the current session principal.

## 3. Mojo protocol

The implemented protocol separates the service control interface from the event
sink:

```mojom
interface UtilitySessionClient {
  OnEvent(uint64 session_id, uint64 sequence, uint32 kind,
          array<uint8> payload);
};

interface UtilitySession {
  Send(uint64 command_id, array<uint8> command)
      => (int32 status, array<uint8> response);
  Acknowledge(uint64 committed_sequence, uint32 credits);
  Cancel(uint64 command_id);
  Close(uint32 reason) => (int32 status, array<uint8> summary);
};

interface UtilityService {
  Start(string name) => (int32 status);
  Invoke(uint64 request_id, array<uint8> request)
      => (int32 status, array<uint8> response);
  Cancel(uint64 request_id);
  CreateSession(uint64 session_id, array<uint8> init,
                uint32 initial_event_credits, uint32 max_event_bytes,
                pending_remote<UtilitySessionClient> client,
                pending_receiver<UtilitySession> session)
      => (int32 status);
};
```

`UtilitySession` is deliberately a separate Mojo interface. `UtilityService`
creates a new instance and hands the browser a dedicated receiver; after that,
`Send`, `Acknowledge`, `Cancel`, and `Close` travel on that session pipe. This
isolates disconnect and ordering state per extension session and prevents the
service factory/unary API from becoming a mixed global dispatcher.

Control messages remain bounded. Large binary attachments use Mojo shared
buffers. A capability that truly transports continuous raw bytes, such as PTY
or media, receives a dedicated data pipe; it does not encode arbitrary bytes as
thousands of events. Agent JSONL and semantic deltas are message-oriented and
fit the ordered event channel when batched.

## 4. Event semantics and backpressure

Utility session events and renderer application events are different layers.

| Property | Utility session event | Application event |
| --- | --- | --- |
| Audience | trusted Rust host | authorized renderer documents |
| Delivery | ordered per session while connected | best-effort broadcast |
| Backpressure | explicit credits and bounded queue | none |
| Acknowledgement | committed sequence + new credits | none |
| Replay | host resumes from durable cursor or restarts session | renderer fetches snapshot/delta |
| Authority | input to Rust validation/persistence | projection notification only |

The Utility starts with a small event-credit window. Each event consumes one
credit and has a bounded payload. Rust validates and, where required, persists a
batch before acknowledging `committed_sequence` and granting more credits. The
Utility must stop producing when credits reach zero; a bounded coalescing buffer
may merge adjacent text deltas but cannot drop lifecycle, approval, tool, final,
or error events.

After persistence, Rust may broadcast a compact application event containing a
revision/cursor and an optional UI-friendly delta. A renderer may miss it and
must reconcile through a snapshot/delta command. Application events therefore
improve latency but never provide Utility backpressure or durable correctness.

## 5. Rust backend protocol

The Rust side acts like the backend of an extension API protocol. A session receives
an unforgeable principal containing application ID, extension ID and content hash,
Driver ID, workspace ID, Attempt ID, and granted capabilities. Host operations
are typed requests, not ambient JavaScript globals:

```text
process.probe       resolve one declared executable and version
process.spawn       start a validated argv/cwd/env plan
process.write       write bounded stdin or PTY input
process.resize      resize an owned PTY
process.cancel      cancel one command or Worker
process.kill        terminate the owned process tree
workspace.read      read an approved bounded file/region
artifact.put        persist a bounded attachment and return an ID
secret.bind         inject an approved secret into a child without revealing it
approval.request    request a user/policy decision
state.checkpoint    persist opaque bounded extension state for recovery
telemetry.emit      emit structured, redacted diagnostics
```

The initial AgentDriver API should prefer `process.spawn` returning an opaque
Worker handle. Subsequent stdout/stderr/exit/PTY frames are routed back to the JS
adapter as inputs; the adapter normalizes them into Agent events. The extension
never receives a raw OS handle, arbitrary environment, database connection,
network socket, or secret value.

Every host request is checked against both the session grant and the manifest.
The Rust backend owns idempotency, handle generations, timeouts, quotas, audit
records, process-tree cleanup, and secret redaction. A Utility crash invalidates
all of its handles. Running external Workers are either cancelled or reattached
only when the Driver contract explicitly proves safe recovery.

## 6. Lifetime policy

Long-lived does not mean unbounded:

- unary `invoke` retains its current 30-second default;
- sessions have no per-turn fixed deadline, but commands may have deadlines;
- a session can remain alive while it owns Workers, pending approvals, or
  uncommitted events;
- idle sessions use a configurable timeout and graceful drain;
- heartbeats detect a wedged isolate without treating model silence as failure;
- CPU, heap, event bytes, queue depth, process count, and restart frequency are
  capped independently;
- shutdown first revokes new host operations, then cancels/drains, persists the
  terminal state, closes Mojo, and finally terminates the Utility process.

Application attempt and inactivity deadlines belong to application policy, not
the generic Tuffite Utility session. Tuffite enforces transport and process health;
the application decides whether a quiet but healthy session may continue.

## 7. API evolution

Tuffite has no production compatibility contract, so this can be a direct ABI
evolution while versions remain at `0.1.0`. The implementation order is:

1. [done] Add versioned Utility session callbacks/handles to the Rust/C ABI and
   Mojo.
2. [done] Add browser `UtilitySessionHost` with credits, bounds, close, crash,
   shutdown handling, and release compilation.
3. [done] Expose safe Rust `UtilitySessionHost`, `UtilitySessionHandle`,
   `UtilitySession`, and emitter APIs without a renderer generic stream.
4. [partial] The release-built Playground v23 runs beyond 30 seconds, crosses
   its initial credit window through ACK, closes with an ordered summary, and
   proves two unary calls execute concurrently, and hard-cancels an infinite
   V8 extension command across the process boundary. Add renderer-reload
   reconciliation and crash/restart cases.
5. [done] Build the restricted JS adapter runtime as a framework-owned session
   service, reusing Chromium `gin::IsolateHolder` with the Utility isolate type
   and V8 module compilation rather than bundling Node or another JavaScript
   engine. Each realm has a 32 MiB heap limit and its own sequenced runner;
   dynamic imports, string code generation, and Wasm are denied.
6. [done] Provide the restricted JavaScript extension example in Playground.
   The two-module fixture validates static relative imports, supervised events,
   and denied access to `eval`, Node globals, and direct network operations.
   Application scheduling and workflow policy remain outside the Framework.

The existing unary API remains useful and should not be overloaded with flags
that silently change its delivery contract.
