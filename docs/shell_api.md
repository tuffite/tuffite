# Typed Mojom Shell API

All Framework and application Shell API contracts are `.mojom` sources.
JavaScript clients, TypeScript declarations, Rust bindings and native adapters
are generated from those contracts. `.d.ts` is an output format only.

## Contract and clients

```mojom
module example;
struct Message { string text; array<uint8> bytes; };
interface Commands {
  Echo@0(Message value) => (Message result);
};
```

Configure the Mojom inputs in `tuffite.json`; the module is declared only in Mojom:

```json
{"build":{"mojomSources": ["shell_api.mojom"]}}
```

Build scripts compose `tuffite::build::framework::Resolver` and
`tuffite::build::shell_api::Generator`. Rust output includes `bindings.rs`;
JavaScript output is the fixed pair `index.js` and `index.d.ts`, with layouts
and clients in the same module. No JS/TS output inventory is generated. Imports
participate in rebuild/watch checks, and Verify mode detects generated drift.
The Framework SDK also includes its internal runtime in this same pair through
OXC AST composition; there is no separately published runtime module.

```js
import { commands } from './generated/index.js';
const result = await commands.echo({text:'hello',bytes:new Uint8Array([1,2])});

import { fs, BaseDirectory } from 'tuffite';
const file = await fs.readFile(BaseDirectory.Data,'settings.json');
```

Application and Framework clients are ES modules. There is no application global
or installation of Framework business methods on `globalThis.tuffite`. The
native global remains the transport and native DOM integration surface.
Framework clients export numeric enums directly. Nullable fields/results use
`null`; nullable arguments may be omitted when trailing, and nullable object
fields may be omitted. A present `false`, zero or empty string remains present.

The `createExampleApi(transport)` factory accepts an explicit `MojomTransport`.
A transport receives canonical channel, command ID, schema fingerprint and
encoded request bytes, and returns response bytes. Importing a client or
injecting a transport grants no native authority.

## Supported data profile

The source-distributed `tuffite-mojom` crate owns loading, lexer/parser, type
resolution and wire layouts. It has no Chromium or SDK dependencies. SDK-specific
Rust/JS/TS generation lives in `tuffite::build::mojom`.

Supported: version-zero structs, all integer widths, float/double, packed bools,
strings, byte arrays, arrays of values/structs/arrays, integer enums, nullable
references and nullable scalar fields, and unary methods with zero or one result.
Nullable scalar fields follow Chromium's presence-bit plus value packing.
Arrays of nullable scalars, defaults, attributes, unions, maps, recursive types,
handles, endpoints and schema versions are rejected. Enum values currently use
non-negative i32 literals. Modules in one compilation must match; imports are
root-confined, cycle checked, source limited and tracked for rebuilds.

The codec limits payloads to 16 MiB, validates UTF-8, scalar ranges, enums, headers,
array lengths, pointer bounds and forward non-overlapping allocations. Rust/JS
accept the generated version-zero layout and reject truncation and trailing data.
Schema fingerprints require matching schemas; mixed-version evolution is not
implemented. None of these codecs is a standalone Mojo message-pipe runtime.

## Process implementations and interfaces

Each interface generates a Rust trait and typed dispatcher. Process ownership is
separate from interface identity:

```rust
#[tuffite::browser(interfaces(shell_api::Commands, shell_api::Diagnostics))]
impl tuffite::Browser for App { /* new(context), optional shutdown() */ }
impl shell_api::Commands for App { /* business methods */ }
impl shell_api::Diagnostics for App { /* business methods */ }

#[tuffite::utility(interfaces(shell_api::Echo, shell_api::Aux))]
#[derive(Default)]
struct Worker { /* state shared by both interfaces */ }
impl shell_api::Echo for Worker { /* business methods */ }
impl shell_api::Aux for Worker { /* business methods */ }

#[tuffite::main(browser = App, utilities(Worker))]
fn main() -> tuffite::app::Builder<App> { tuffite::app::Builder::new() }
```

`main` selects process implementation types. Browser installs interface registries
from its attribute. All interfaces on a Worker share one supervised process and
instance. Registration rejects duplicate ownership and Browser/Utility conflicts.
Utility initialization uses `Default` or an asynchronous `factory = create_worker` beside
`interfaces(...)`; the factory accepts `UtilityContext` and returns
`ApplicationResult<Worker>`. Its internal owner name is `module_path!()` + `::` +
the struct name, also exposed by `UtilityContext::name()`. Optional startup JSON
in `app.utilities` uses that name; empty entries are unnecessary. Raw Rust/V8
utilities retain their explicit `name = "..."` declaration and session lifecycle.

## Permissions and routing

Permissions use exact, case-sensitive `module.Interface.Method` names:

```json
{"identifier":"business","origins":["example://business"],
 "shellApi":["example.Commands.Echo","tuffite.Fs.ReadFile"]}
```

Place grants in `app.security.capabilities`. Prefer exact methods; wildcards
expand against the generated endpoint catalogue at packaging. Unknown endpoints
and patterns fail validation. There is no per-method permission metadata or
scope registry. The loader validates scope syntax; each concrete API validates
its resource with the common helper, passing kind and read/write behavior.
APIs without resource scopes reject scoped grants. Browser/Utility business
services currently expose method authorization and reject scoped grants too.
A field named `path` does not automatically confer filesystem authority. Scope root labels, such as `data/**`,
are independent of numeric enum wire values.

After origin, channel and command-ID checks, native application routing resolves
an interface to its Utility owner in-process; otherwise it dispatches to Browser.
Lookup sends no IPC and launches no process. Generic renderer Utility invoke and
cancel cannot target typed interfaces or their internal owners, even with a raw
Utility grant. Fingerprints, payload validation, response identity, cancellation,
document teardown and native-only restrictions remain enforced at their owners.
No `utilityRoutes`, typed service aliases or per-grant `service` field is used.

Grouped typed Utility apps require the `UtilityInterfaces` host capability before
Browser construction. The route resolver is an optional, size-checked C ABI tail
callback. The independent Cargo SDK and native C ABI remain separate from Mojo.

## Native Framework implementation

`framework/shell_api/*.mojom` is authoritative. Chromium's parser/IR drives
thin native interface and dispatcher templates under `build/mojom_templates`.
Per-source `*-wire.mojom` projections expose method request/response structs to
the official C++ generator. Implementations directly use its structs, enums,
`StructPtr`s and arrays; there is no duplicate DTO representation or generated
`ToWire`/`FromWire` conversion. Application SDK generation emits Rust/JS/TS;
the SDK exposes no C++ output API or `tuff framework generate-cpp` command.

Native handlers inherit generated abstract interfaces such as `generated::Fs`.
`generated::Fs::Bind(std::make_shared<fs::FsImpl>())` produces one interface
service. The router selects the interface, then its generated dispatcher selects
a method. `BuiltinServices()` is only the composition list of implementations;
there is no reflection or per-method route registry.

Methods take `(const dispatch::Context&, RequestPtr, Callback)`. Context is an
explicit, borrowed invocation identity; the callback is an ordinary
`base::OnceCallback<void(rpc::Status, Results...)>` that only replies. The public
signature preserves all response parameters. Empty responses take only Status.
`rpc::Method` centralizes official-codec validation, response serialization and
pending implementation ownership. `rpc::Reject(callback, status)` sends an error
without a wire payload, regardless of the response parameter count.

Context uses weak document/application references. The dispatcher rejects an
expired caller with `kCancelled`. Async implementations must explicitly copy
needed identity/snapshots before returning, never retain the borrowed Context
reference, and recheck validity before later accessing the caller. Invocation
identity is never stored as mutable current-caller state on shared services.
Service bindings and pending completions retain the implementation object.

Native bindings have standard `.mojom.h` / `.mojom.cc` paths. GN invokes the
official parser and `--generate_extra_cpp_only` template projection without
emitting unused Mojo endpoint bindings at those same paths.
Application URL schemes come from the entry window and are independent of
Mojom modules; native routing uses the generated method table, never a module
prefix configured by hand. The source frontend currently compiles one module
per SDK generation, with dotted module names supported.

Framework dispatch already carries RPC identity/status and transports raw typed
struct bodies. Application dispatch retains the CERM request/response/cancel
envelope around Mojom structs to preserve the standalone C ABI. Framework native
Rust calls use generated `ShellApiRequest` encode/decode adapters. Window snapshots
retain their captured messaging identity independently of the data codec.

JS/TS source templates are parsed and filled using typed OXC AST holes, including
interface members, parameters and enum members. Generated identifiers and literal
metadata become AST nodes. OXC codegen emits and reparses outputs. Rust generation
uses quote tokens, syn validation and prettyplease formatting.

## Verification

Run `scripts/test-browser-runtime.mjs`, `scripts/test-mojom-codec.mjs` and
`scripts/test-framework-mojom-codec.mjs` with Node. Cargo tests cover the frontend,
source generation, codecs and independent generated SDK fixtures. Native
`dispatch_unittests` compares the official C++ codec against the same Rust/JS
fixtures for enums, nullable scalars/references and nested arrays.

Use `python3 x.py build` / `package` for Framework, then package Playground with
the `tuff` CLI. Packaged smoke checks cover normal startup, Framework methods,
Browser and Utility typed calls, shared Worker state, process separation, origin
and endpoint permissions, raw bypass attempts, forged IDs and schema mismatch.

Performance checks: `node scripts/bench-mojom-codec.mjs` measures local codec
round trips. Set `TUFFITE_MOJOM_BENCHMARK=1` when running the packaged `--mojom`
smoke check for C++/Rust Browser, Rust Utility, large payload and concurrent-call
latencies. Results are diagnostic, not timing assertions. Utility unary replies
currently have a 1 MiB total-frame budget (including the envelope).
