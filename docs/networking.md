# Tuffite networking

## 1. Decision

Tuffite provides one framework-level HTTP networking capability backed by
Chromium Network Service. Rust Runtime code and trusted App renderer documents
can use the same Network Broker, policy, proxy configuration, cookie store,
cache, certificate verifier, authentication handling, and Chromium network
sandbox. Tuffite does not make a separate Rust HTTP client the default framework
transport.

This does not replace the web platform:

- ordinary renderer `fetch()`, navigation, subresources, and `<explorer>` traffic
  already use Chromium Network Service and retain normal web security semantics;
- `window.tuffite.net.request()` is an explicit privileged ShellAPI capability;
- Rust application code uses `tuffite::net`, which reaches the same Broker;
- an application may still use another Rust HTTP library itself, but that traffic
  is outside Tuffite's unified session, policy, observability, and network stack.

The public name is `Network Broker`. The C++ implementation is the `Network
Service adapter`; neither is a second network engine.

## 2. Architecture

```text
Trusted App renderer                      Rust application command
window.tuffite.net.request()              tuffite::net::Client
          │ ShellAPI                               │ Runtime API
          └──────────────────┬─────────────────────┘
                             ▼
                  Network Broker in Runtime
                  ├── capability decision
                  ├── URL/method/header policy
                  ├── request ownership/cancel
                  └── metrics and audit context
                             │ Tuffite FFI control events
                             ▼
              Network Service adapter in Framework
              ├── StoragePartition/NetworkContext selection
              ├── URLLoaderFactory and ResourceRequest
              ├── redirect/auth/certificate mediation
              └── Mojo shared buffer/data pipe plumbing
                             │ Mojo
                             ▼
                  Chromium Network Service process
                  ├── proxy and PAC
                  ├── DNS and sockets
                  ├── cookies and HTTP cache
                  ├── TLS/certificate verification
                  └── network sandbox
```

Runtime owns authorization and logical request lifetime. Framework owns Chromium
network objects and sequences. Network Service owns network I/O. The request body
and response body do not need to pass through Runtime as complete buffers.

## 3. JavaScript API

The built-in API is generated into the manifest-backed `window.tuffite` Proxy:

```ts
interface TuffiteNet {
  request(request: NetRequest): Promise<NetResponse>;
}

interface NetRequest {
  url: string;
  method?: string;
  headers?: ReadonlyArray<readonly [string, string]>;
  body?: ArrayBuffer | ArrayBufferView | ReadableStream<Uint8Array>;
  credentials?: "omit" | "include";
  redirect?: "follow" | "manual" | "error";
  cache?: "default" | "no-store" | "reload" | "no-cache";
  timeoutMs?: number;
  signal?: AbortSignal;
}

interface NetResponse {
  readonly status: number;
  readonly statusText: string;
  readonly url: string;
  readonly redirected: boolean;
  readonly headers: ReadonlyArray<readonly [string, string]>;
  readonly body: ReadableStream<Uint8Array>;
}
```

Example:

```ts
const response = await window.tuffite.net.request({
  url: "https://api.example.com/v1/project",
  method: "POST",
  headers: [["content-type", "application/octet-stream"]],
  body: encodedProject,
  credentials: "include",
  signal: controller.signal,
});

if (response.status !== 200) {
  throw new Error(`request failed: ${response.status}`);
}

for await (const chunk of response.body) {
  consume(chunk);
}
```

It is named `request`, not `fetch`, because it is privileged application I/O and
does not claim browser `fetch()` semantics. It does not expose Chromium Mojo
types or raw socket handles. WebSocket, WebTransport, and raw TCP/UDP are not
part of this first API; each needs a separate protocol and capability design.

## 4. Rust API

Rust commands and background tasks use an asynchronous Runtime facade:

```rust
let response = app
    .net()
    .request(
        tuffite::net::Request::get("https://api.example.com/v1/project")
            .credentials(tuffite::net::Credentials::Include),
    )
    .await?;

let mut body = response.into_body();
while let Some(chunk) = body.next().await {
    process(chunk?)?;
}
```

The safe Rust facade creates a Network Broker request. FFI carries control
metadata, request IDs, completions, and bounded body chunks or stream operations;
it does not expose `network::mojom` handles to application code. Runtime applies
the application's network policy even when the caller is trusted Rust code, so
renderer and background traffic have one auditable policy baseline. Framework
internal update and telemetry clients use distinct principals and policies.

## 5. Session and storage semantics

The default `AppSession` selects the application's default BrowserContext and
StoragePartition. Requests therefore share Chromium-managed:

- proxy and PAC configuration;
- cookie store and cookie partitioning rules;
- HTTP cache, HSTS, certificate verification, and connection pools;
- client-certificate and HTTP authentication policy;
- network sandbox and platform network settings.

Sharing a StoragePartition does not mean inventing a renderer frame. Brokered
requests have explicit browser-created isolation metadata and a trusted request
principal. They do not impersonate arbitrary web origins. Network isolation and
anonymization keys must be stable for the declared application session and must
not use an empty or globally shared key as a shortcut.

Optional future sessions can be explicit:

```text
AppSession                 persistent application partition
EphemeralSession           in-memory cookies/cache, destroyed with the session
NamedSession(name)         separately declared persistent partition
```

A renderer cannot choose an arbitrary partition name. Creating or selecting a
session is a separately authorized Runtime operation.

Browser-process URL loading does not automatically run through a renderer's
service worker. Unified networking promises the same Chromium Network Service and
session facilities, not interception by page service workers or perfect identity
with renderer `fetch()`. Applications that require service-worker semantics use
standard `fetch()` in the relevant App document.

## 6. Credentials, CORS, and origins

`window.tuffite.net.request()` is not a CORS bypass exposed to arbitrary pages.
It is installed only for authorized `WebContentsRole::App` documents and every
request is checked against a network capability. The default capability denies
all destinations.

Two rules remain distinct:

1. Standard renderer `fetch()` follows Chromium CORS, Fetch Metadata, CSP, and
   other web-platform rules.
2. Brokered `net.request()` follows Tuffite's privileged network policy. CORS is
   not treated as its authorization mechanism.

Credentials default to `omit`. `include` is allowed only when the matching policy
permits credentials for the destination. Runtime and Framework derive initiator
and isolation metadata from authenticated application context; the caller cannot
set arbitrary `Origin`, cookie partition keys, or network isolation keys.

The response does not expose `Set-Cookie` by default. Chromium processes cookie
effects according to the selected session and cookie policy. Sensitive or hop-by-
hop request and response headers are filtered. At minimum, applications cannot
directly set `Host`, `Connection`, `Content-Length`, `Cookie`, `Proxy-*`, `Sec-*`,
or framework authentication headers. Authorization headers require explicit
destination capability and must be redacted from logs.

## 7. Capability policy and SSRF resistance

Network access is deny-by-default and separately constrained from command access.
A conceptual application policy is:

```toml
[network.api]
callers = ["playground://main", "rust:application"]
urls = ["https://api.example.com/v1/**"]
methods = ["GET", "POST", "PUT", "DELETE"]
credentials = "include"
redirects = "same-policy"
private_network = false
max_upload_bytes = 16777216
max_response_bytes = 268435456
timeout_ms = 30000
concurrent_requests = 16
```

Policy checks cover:

- normalized scheme, host, effective port, and path before dispatch;
- allowed HTTP methods and request headers;
- credential mode and selected network session;
- upload, response, concurrency, timeout, and redirect limits;
- every redirect target before it is followed;
- private, loopback, link-local, multicast, and otherwise restricted networks;
- DNS rebinding and resolution changes according to Chromium's resolved endpoint
  information and network policy hooks;
- external protocols, file URLs, data URLs, and unsupported schemes.

URL string allowlists alone are insufficient. A policy that allows public hosts
but denies private networks must remain enforced after DNS resolution and on
redirect/retry. Access to localhost or private networks is a distinct capability
and cannot be enabled by a wildcard internet rule. Proxy behavior must not turn a
denied direct destination into an unreviewed tunnel.

The current `tuffite.json` surface accepts legacy exact hosts, `*`, and
normalized HTTP(S) URL patterns such as `https://*.example.com/api/*` in
`network.allow`. Matching reuses Chromium's `base::MatchPattern`. A global `*`
never authorizes a literal loopback, private, or link-local host; that requires
an exact host or exact-host URL pattern. Credentials in URLs and non-HTTP(S)
schemes remain invalid.

Framework update, development server, and application network policies are
separate principals. Production applications cannot inherit permissive developer
localhost access.

## 8. Streaming, copies, and backpressure

Request and response metadata use Shell Protocol control messages. Payloads use:

- inline/`BigBuffer` for small fixed bodies;
- shared buffers for medium bodies with known size;
- Mojo data pipes for streaming uploads and responses.

For a JavaScript caller, Framework can connect the Network Service response pipe
to the authorized renderer stream endpoint after Runtime has approved the request.
Runtime retains logical ownership and receives lifecycle/metrics events, but body
bytes need not make a round trip through Rust. Closing the `ReadableStream`,
aborting its signal, navigating the document, revoking its capability, or
destroying the window cancels the URLLoader and closes the pipe.

For a Rust caller, Framework adapts the Mojo pipe to bounded asynchronous FFI read
operations. Only a configured amount of data can be outstanding across FFI. Rust
consumer backpressure propagates to the data pipe and Network Service. Convenience
methods such as `bytes(limit)` may collect a response only with an explicit limit.

## 9. Redirects, authentication, and certificates

- Redirects pause at the Framework policy boundary and are checked before follow.
- Cross-origin redirects can strip credentials and sensitive headers even when
  both destinations are allowed.
- HTTP authentication challenges use an application policy/delegate; credentials
  are not requested by arbitrary renderer UI.
- Client-certificate selection is mediated by Runtime policy and trusted UI.
- TLS verification uses Chromium defaults. Certificate-error bypass is disabled
  in production and cannot be requested per call by application JavaScript.
- Proxy authentication and PAC evaluation remain Chromium responsibilities, with
  platform UI/delegation supplied by Framework where required.

## 10. Lifecycle and errors

Each request belongs to a principal and lifetime:

```text
Document request     cancelled when the authorizing document generation dies
Application request  may survive windows; cancelled during Runtime shutdown
Framework request    owned by a distinct internal service and policy
```

The stable error model distinguishes policy denial, invalid URL/header, timeout,
cancelled, redirect denied, authentication required, certificate failure, DNS,
connection, protocol, response too large, and internal adapter failure. Chromium
`net::Error` values may be retained as diagnostic metadata but are not the sole
stable public error contract.

Retries are not automatic by default for non-idempotent methods. A future retry
policy must account for replayable bodies, idempotency, deadlines, and redirect
history.

## 11. Observability and privacy

Network tracing records request ID, caller principal, policy name, normalized
destination, method, status, byte counts, timings, cache/proxy summary, redirect
count, and terminal error. It excludes bodies, cookies, authorization values,
query secrets, and sensitive headers by default. URL logging supports policy-
defined redaction.

DevTools observation of a brokered browser-process request is not guaranteed to
match a page `fetch()` entry. Tuffite provides Runtime diagnostics and may add a
dedicated DevTools panel. The design must not fake renderer attribution merely to
make a request appear in the standard Network panel.

## 12. Source layout

```text
crates/tuffite-runtime/src/network/    Broker, policy, ownership, Rust API
crates/tuffite/src/net/                public safe Rust facade
src/tuffite/network/browser/           Chromium Network Service adapter
src/tuffite/network/common/mojom/      control and stream contracts
src/tuffite/framework/api/network.*  window.tuffite.net route adapter
```

The Network Service adapter can be compiled with `TUFFITE_RUNTIME` initially. If
independent frameworks later consume it, introduce `TUFFITE_NETWORK` only when that
separate build boundary has a real consumer; avoid speculative buildflags.

## 13. Required tests

- proxy, PAC, cookies, cache, HSTS, and certificate behavior through Chromium;
- persistent, ephemeral, and named session isolation when implemented;
- credential omission by default and policy-gated inclusion;
- forbidden request/response headers and log redaction;
- initial URL, redirect, private-network, DNS-rebinding, and scheme denial;
- upload and download streaming with slow consumers and bounded memory;
- cancellation by `AbortSignal`, stream close, navigation, crash, and shutdown;
- response size, timeout, redirect, concurrency, and upload limits;
- same policy for `window.tuffite.net` and `tuffite::net`;
- `<explorer>` cannot access the privileged API;
- malformed Shell Protocol and Mojo messages;
- Network Service crash/restart behavior;
- Windows, macOS, and Linux proxy/certificate integration.
