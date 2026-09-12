# Tuffite ShellAPI

## 1. Position in the architecture

ShellAPI is the capability API that Tuffite Runtime provides to trusted
renderer documents. It belongs to Runtime, alongside the command router and
capability manager. It is not part of Tuffite FFI and it is not a peer runtime.

The independently packaged Shell Protocol contains only wire definitions,
stable identifiers, codecs, compatibility tests, and code-generation support.
This separation prevents the Chromium binding and developer tooling from
depending on the whole Runtime.

```text
JavaScript / TypeScript
        │ generated Proxy facade and Uint8Array codec
        ▼
Shell Binding in the renderer
        │ window.tuffite.invoke(route, payload) over Mojo
        ▼
Shell broker in Tuffite Framework
        │ command route uses Tuffite FFI; built-ins remain in browser
        ▼
ShellAPI Host in Tuffite Runtime
        ├── primary-document and origin validation
        ├── capability authorization
        ├── command router
        └── application command
```

No JSON is required on this path. JSON may still be an application payload if
an application explicitly chooses it.

## 2. Public JavaScript shape

The renderer receives a narrow, generated API instead of a universal native
object. A conceptual low-level API is:

```ts
interface TuffiteShell extends EventTarget {
  invoke(route: string, payload: Uint8Array): Promise<Uint8Array>;
  on(
    name: string,
    listener: EventListener,
    options?: AddEventListenerOptions,
  ): () => void;
}
```

Generated command wrappers hide numeric IDs and binary codec details:

```ts
const project = await window.demo.project.open(path);
```

Built-in facades are generated from `src/tuffite/framework/api/*.d.ts` with the same
`tuffite::build::shell_api::Generator` pipeline and exposed under the immutable
`window.tuffite` object. An
application can declare a separate generated global facade, for example:

```ts
const response = await window.tuffite.net.request({
  url: "https://api.example.com/projects",
  method: "GET",
});

const message = await window.demo.commands.doSomething("hello");
```

Both facades are manifest-backed, read-only Proxy trees. A Proxy path resolves
only to a generated command entry containing internal wire metadata and codecs; an
unknown property chain never becomes a string RPC. `then`, `constructor`,
`prototype`, `__proto__`, symbols, conflicting leaf/namespace paths, and unsafe
global names are rejected. Proxy objects and method functions are identity-
stable and cannot be overwritten.

`build.shellApi.definitions` is a non-empty array of `.d.ts`
declaration files. `namespace` is both the application JavaScript global and
the route prefix; the application URL scheme uses the same name.
Multiple files may contribute to the configured root namespace; the generator
merges them, rejects duplicate paths and declarations, and emits one Rust
dispatcher plus one JavaScript Proxy facade. Declaration files keep this input
surface type-only and avoid implying that executable TypeScript is bundled.

The single public npm package is `tuffite`. It contains the low-level renderer
runtime as the `tuffite/runtime` export, the built-in TypeScript contract, and
the generated built-in facade. The Rust packager embeds and injects the exact
matching runtime and facade, so applications do not copy either file or list
them in `bundle.resources`.

Application commands use their fully qualified API path, such as
`demo.commands.doSomething`, and carry the generated BRQ1 envelope. Built-ins
use paths such as `tuffite.net.request`, `tuffite.utility.invoke`, and
`tuffite.window.create`; the low-level runtime contains no
API-specific dispatch. The
JavaScript path is presentation, not wire identity. Renaming an application
global from `window.demo` to another valid name does not change canonical
command names or wire IDs. Framework commands occupy `window.tuffite`; an
application cannot register or replace paths in that namespace.

Rust handlers use generated, statically checked dispatch. The `.d.ts` contract
remains authoritative; no runtime string registry or inventory is involved:

```rust
#[tuffite::command(shell_api::commands::DoSomething)]
fn do_something(
    app: &mut Playground,
    message: String,
) -> Result<String, tuffite::CommandError> {
    app.record(&message);
    Ok(format!("Rust received: {message}"))
}

#[tuffite::main(browser = PlaygroundBrowser)]
fn main() -> tuffite::app::Builder<PlaygroundBrowser> {
    tuffite::app::Builder::<PlaygroundBrowser>::new()
        .commands(tuffite::generate_handler![do_something])
}
```

The generated `COMMAND` descriptor exposes the complete channel and stable ID.
The `command` attribute binds it to the Rust function, while
`generate_handler!`
only lists the handlers registered with the application. Registration remains
explicit: changing a Rust function name never silently changes its endpoint.
Missing attributes and wrong browser/parameter/result types fail at compile
time; duplicate channels or IDs fail during generation or Builder validation.
Endpoints omitted from `generate_handler!` remain unregistered and return the
protocol's unknown-command error. Handlers are synchronous because
application dispatch already runs on a Rust worker rather than Chromium's UI
sequence; finite Chromium-scheduled work can use `BrowserContext::executor()`.

The immutable low-level binding is installed only in a primary main document
whose committed origin is authorized. No entry-origin grant is implicit. `app.security.capabilities` authorizes
both business and framework methods through each capability’s `shellApi` entries. This policy applies equally to outer windows and the ordinary inner
`WebContents` used by `<explorer>`, so an Explorer receives the bridge only when
its own committed origin is listed. Iframes, DevTools, opaque origins, and
provisional documents are denied. Renderer allowlisting controls exposure, and
the browser repeats the authorization on every invocation.

Manifest origin lists accept exact canonical origins, `*`, and a leading-host
wildcard such as `https://*.example.com/`. Tuffite normalizes and validates each
pattern at startup, then uses Chromium's `base::MatchPattern` in both renderer
exposure and browser-side authorization. `*` never makes opaque origins,
subframes, or non-App roles eligible.

Rust can broadcast a named binary application event. The bridge dispatches a
`CustomEvent` named `channel` through the shared `window.tuffite` EventTarget.
Its detail is `{channel, data}`, where `data` is a `Uint8Array`. Standard
`addEventListener/removeEventListener` semantics allow multiple listeners:

```js
const listener = ({detail}) => {
  if (detail.channel === 'app:workspace-changed') {
    refreshFromRevision(detail.data);
  }
};
window.tuffite.addEventListener('channel', listener);
// Later: window.tuffite.removeEventListener('channel', listener);
```

There is no `window.tuffite.channel` namespace. An application facade may use a
business namespace named `channel`; only native bridge members such as
`invoke`, `postMessage`, `addEventListener`, and `removeEventListener` are
reserved on the framework facade.
During an explicit `tuff dev` launch, the browser grants the exact loopback
origin from `--tuffite-dev-url` access to the same generated ShellAPI. This
runtime-only exception rejects HTTPS, credentials, and non-loopback hosts and
requires the CLI-issued nonce/digest handshake with the direct parent process; passing
the switch directly to a production executable fails closed. It is never
serialized into `tuffite.json` or a package. The `tuffite/vite` plugin serves
the framework modules from that development origin; normal remote HTTP origins
remain unprivileged.
The override code is available in the default release build for local HMR but
requires the direct Tuffite CLI parent and its one-time nonce. CI builds compile
it out; ordinary release launches never receive the token and reject the switch.
Application events are deliberately ephemeral notifications for live,
authorized documents. They have no acknowledgement, replay, durable ordering,
or backpressure contract. An event should normally carry a revision/cursor and
an invalidation or compact progress summary; the renderer then reads the
authoritative snapshot or delta through a generated command. Navigation,
renderer restart, or a slow consumer may miss an event, so correctness must
never depend on receiving every event.

Ordered events produced inside a long-lived Utility session use the separate
credit-controlled Utility session protocol. Rust validates and persists those
events before optionally projecting a revision/delta here. Application events
must not be used as the Utility-to-host transport.

ShellAPI commands are unary: the declaration generator accepts `Promise<T>`
and does not turn `AsyncIterable<T>` into a generic stream registry. This keeps
document ownership, cancellation, buffering, and backpressure out of the
control plane. A capability that owns continuous bytes exposes its own typed
API backed by a Mojo data pipe or another purpose-built transport.

Window management follows this same path. Calls are generated from the
framework TypeScript contract and dispatched as ordinary `window.*` builtins;
the browser derives the current window from the calling document and exposes
only opaque `WindowId` values. See [Window management](window-management.md).

## 3. Transport classes

Different payload sizes use different Chromium IPC primitives:

| Class                     | Recommended transport                          | Purpose                                            |
| ------------------------- | ---------------------------------------------- | -------------------------------------------------- |
| Control and small payload | Generated binary codec in Mojo `BigBuffer`     | Unary commands, results, errors, metadata          |
| Low-rate notification     | Application event containing a revision/cursor | Invalidation, lifecycle, compact progress          |
| Medium binary attachment  | Mojo shared buffer                             | Images, compiled data, file chunks with known size |
| Large or unbounded data   | Capability-specific Mojo data pipe or PTY      | Files, media, terminal output, generated output    |

Thresholds are implementation tuning parameters, not protocol semantics. The
sender may select a shared buffer below the nominal threshold when zero-copy
behavior is more important, but both ends must enforce configured size limits.

Data pipes provide byte-level backpressure. Runtime must not read an unbounded
source into memory to turn it into one control response, and application events
must not be used as a chunk transport. Cancellation closes the owning
capability's pipe and propagates to its producer. The current
`window.tuffite.net.request()` remains a bounded unary response; a future
streaming body API belongs to the Network capability and will expose a
`ReadableStream<Uint8Array>` backed directly by its data pipe, not a generic
ShellAPI stream handle. See [Networking](networking.md).

## 4. Message envelope

Every control message carries a typed envelope with at least:

```text
magic
frame_kind
flags
request_id
command_id
schema_fingerprint
payload_length
```

Requirements:

- fixed byte order and field widths are part of the protocol;
- unknown flags and message kinds are rejected unless explicitly extensible;
- lengths are validated before allocation or typed payload decoding;
- request IDs are scoped to one authorized document connection;
- document identity comes from the browser, never from renderer-supplied data;
- no message is dispatched before frame, schema, and capability checks succeed.

The current control path has request, response, and framework-controlled
application-event envelopes. Unknown request flags are rejected. Cancellation
for a long-lived capability is part of that capability's typed API (for
example, `tuffite.utility.cancel`) rather than a universal stream message kind.

## 5. Command declaration and IDs

Application authors declare the public contract in TypeScript:

```ts
export interface Project {
  id: bigint;
  name: string;
}

export type ProjectId = string;

export enum OpenMode {
  Read = "read",
  Write = "write",
}

export namespace demo {
  export namespace project {
    export function open(id: ProjectId, mode: OpenMode): Promise<Project>;
  }
}
```

Commands are declared only as functions inside namespaces. Interfaces are data
records, not command containers. The code-generation phase integrated into
`tuff build` uses Oxc's Rust TypeScript
parser and derives Rust data types, namespace modules, typed request structs and
the decoder, the immutable JavaScript Proxy, global `.d.ts`, codec descriptors,
and global declarations. Business code matches the generated `Request` enum and
returns through each request type's `respond` helper; no generated handler trait
is required. Numeric IDs are never maintained manually. The canonical name is:

Application `build.rs` files configure paths with a bindgen-style
`tuffite::build::shell_api::Generator`. Each output independently selects `Write` or
`Verify`: generated Rust normally goes to Cargo's `OUT_DIR`, while checked-in
JavaScript and declarations are verified during builds. When assembled through
`tuffite::build::Builder`, these inputs and verified outputs are registered with
Cargo automatically for build-script reruns. Numeric command IDs are
generated into the Rust decoder and JavaScript transport table; they are not a
public API and application code never declares or references them.
The Rust emitter constructs `proc_macro2::TokenStream` values with `quote`,
parses the complete result as a `syn::File`, and formats it with `prettyplease`.
It does not interpolate Rust source strings, so identifiers, literals, nested
types, and raw-keyword names are validated as Rust syntax during generation.
The framework invokes the same Generator with `Facade::Framework` (or the
equivalent manifest setting), so built-in and application Proxy trees share the
same parser, descriptors, runtime validation, constants, and naming rules.

```text
<stable application namespace>:<stable command name>

com.example.demo:project.open
```

The recommended ID is:

```text
first_u64_le(
  BLAKE3("tuffite.command.v1\0" + canonical_name)
)
```

Normative rules:

- UTF-8 normalization and allowed characters are specified by the generator.
- Rust module paths, crate names, `DefaultHasher`, and build-local ordering are
  not inputs.
- the wire sends the `u64` ID and generated request/result schema fingerprint;
- the build checks all commands for ID and canonical-name collisions;
- generated JavaScript records the path, wire ID, and complete request/result
  codec descriptors, so a separate command lock file is unnecessary;
- the TypeScript API path is the stable command name and changing it is a wire
  compatibility change;
- compatible migrations may declare stable aliases;
- published canonical names and the `tuffite.command.v1` domain never change
  solely because of a future marketing rename.

The protocol domain should be finalized before the first stable release. If
brand independence is desired, replace it now with a permanent organization-
owned protocol domain and document that choice.

## 6. Serialization model

The TypeScript contract is the single source of truth. The static generator
accepts a deliberately bounded subset and does not execute TypeScript: nested
API namespaces, exported functions returning `Promise<T>`, named data
interfaces, non-generic `type` aliases, explicit homogeneous string or integer
`enum`s, primitive namespace `const`s, `boolean`, `number`, `bigint`, `string`,
`Uint8Array`, `Date`, arrays, and optional/nullable values. Constants are exposed
as immutable Proxy leaves and never receive command IDs or IPC calls. `any`,
computed or implicit enum values, unknown types, inline data objects, executable
statements, and recursive data graphs are rejected at generation time.

The default deterministic codec uses little-endian fields: boolean is one byte,
`number` is IEEE-754 `f64`, `bigint` is signed `i64`, `Date` is signed epoch
milliseconds, strings and `Uint8Array` have a `u32` byte length, arrays have a
`u32` element count, optional values use a strict 0/1 tag, and object fields are
encoded in declaration order. Both sides reject invalid UTF-8, tags, trailing
bytes, oversized collections, and JavaScript values that do not match the
declared type. `Uint8Array` remains the raw binary escape hatch.

The schema tool emits:

- Rust namespace modules, request/result types, codecs, and request decoder;
- JavaScript command Proxy and codec descriptors;
- TypeScript global declarations;
- internal command transport metadata embedded in the generated bindings;
- compatibility and golden-vector tests.

The generated decoder validates the complete payload before calling business
code. Large attachments should not be represented as a giant control-message
`Uint8Array`; they use shared buffers or data pipes so payload bytes do not need
to be copied through each layer.

## 7. Authorization

Authorization is evaluated for every invocation using browser-authenticated
context:

```text
live document-scoped Mojo connection
AND primary main frame
AND committed non-opaque origin is in the manifest policy
AND command capability granted
AND request constraints satisfied
```

Capabilities are deny-by-default and declared in application configuration. A
command can add constraints such as allowed path roots, dialog-only user
activation, maximum byte count, or allowed window IDs.

Example: `{ "identifier": "ui", "origins": ["myapp://main"],
"shellApi": ["myapp.commands.save", "tuffite.storage.get"] }`
is a rule inside `app.security.capabilities`. Generator settings live separately
in `build.shellApi`; business grants never imply framework grants.

Renderer-controlled origin strings, frame names, DOM attributes, or command IDs
cannot grant a capability.

## 8. Document lifecycle

ShellAPI authorization is document-scoped, not renderer-process-scoped. On
navigation, document replacement, frame detachment, role change, renderer crash,
or window destruction, Runtime must:

1. revoke the connection;
2. reject new messages;
3. cancel or detach in-flight work according to command policy;
4. close data pipes and shared-buffer leases;
5. discard late results using the document generation.

Commands default to cancellation when their initiating document disappears. A
command may explicitly become application-scoped when continued background work
is part of its contract.

## 9. Errors and observability

The stable protocol error categories include malformed message, incompatible
version, unauthorized, unknown command, invalid argument, cancelled, deadline
exceeded, resource exhausted, unavailable, and internal failure. Application
error details are typed payloads and must not expose arbitrary Rust debug text
or sensitive local paths by default.

Tracing records request ID, command ID, role, outcome, byte counts, queue time,
execution time, and cancellation reason. Payload contents are excluded unless a
command explicitly defines safe diagnostic fields.

## 10. Versioning before the stable release

Neither the author manifest nor the packaged application manifest uses
`schemaVersion`. FFI compatibility remains internal to
the Framework Distribution, and the wire codec owns its frame format; applications
do not repeat either value. Each request carries the generated
command schema fingerprint and is rejected before payload decoding when it
differs from the application handler's fingerprint. Any wire or schema change
is therefore a coordinated MVP change: regenerate bindings and rebuild all
layers together. Cross-version negotiation can be designed when the public API
approaches stability.

Application binding outputs are configured only in `build.rs` with
`Generator::rust` and `Generator::javascript`, both using `OutputMode::Write`.
Cargo owns generation and tracks the declarations, manifest and generated files;
missing or modified outputs rerun the build script. Build the Rust application
before building the frontend, then assemble or launch the package. The application
CLI performs its normal Cargo build without a separate generation pass or a
feature override. The Vite integration reports missing application bindings and
asks the developer to build the application first.

In CI, build the application and check `git diff --exit-code` afterwards to catch
uncommitted generated output changes. SDK maintenance can still use
`tuff framework generate-js <manifest> --output <directory> --check` with an
explicit destination; this command never executes an application's build script.
