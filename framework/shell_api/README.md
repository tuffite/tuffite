# Framework Shell API contracts

`*.mojom` is the only input contract. TypeScript declarations are generated
outputs, never compiler inputs. All contracts use `module tuffite` and explicit
method ordinals. JS names use camelCase; native channels and permissions keep
the exact IDL spelling, for example `tuffite.Fs.ReadFile`.

```js
import { fs, BaseDirectory } from 'tuffite';
const bytes = await fs.readFile(BaseDirectory.Data, 'settings.json');
```

Clients are ES modules. Importing them never installs business methods on
`globalThis.tuffite`. The native global remains the transport and native DOM
integration surface. `BaseDirectory` is a generated numeric enum; strings such
as `"data"` are rejected by the codec. Filesystem permission scopes still use
root labels such as `data/**`.

## Generation

The source-distributed Rust frontend parses contracts, imports, enums, nullable
types, arrays and version-zero structs. The Tuffite backend generates JS/TS
using OXC AST templates and Rust using quote/syn/prettyplease.

GN uses Chromium's official parser and `extra_cpp_template_paths` to emit
Dispatch abstract interfaces and thin interface registration bindings.
The extension emits per-source `*-wire.mojom` data contracts, which the official
C++ generator compiles into types, validators and codecs. There is no SDK C++
backend or `generate-cpp` CLI command.

Business classes implement interfaces such as `generated::Fs` with `override`.
Bind an implementation once: `generated::Fs::Bind(implementation)` returns one
interface dispatcher. `BuiltinServices()` composes these objects, with no
per-method registry. Methods receive `(const dispatch::Context&, RequestPtr,
Callback)`: caller identity is explicit and the ordinary OnceCallback only
replies. Implementations directly use official C++ StructPtrs, enums and arrays.
The common `rpc::Method` runtime validates and serializes responses with all
result fields; empty responses need no placeholder argument. Async work must
copy needed Context data, retain no borrowed reference, and recheck validity
before accessing the caller.
Both service bindings and asynchronous replies retain the implementation.
Native outputs use the standard `.mojom.h` and `.mojom.cc` names.

Framework dispatch carries request identity, origin, native application context,
cancellation and RPC status. Its body is one typed Mojom request or response.
Application Browser/Utility calls retain their CERM envelope around the same
Mojom data codec. Neither path exports handles or Mojo endpoints to the Cargo SDK.

There is no scope registry or per-method permission metadata. Dispatch verifies
caller identity and method grants. Each concrete API checks its resource scope
with the shared helper, passing the resource kind and read/write behavior.
A scoped grant cannot authorize an API without compatible scope semantics.
`Remoting.InjectInput` remains native-application-only, enforced by routing policy.

## Validation

- `scripts/test-framework-mojom-codec.mjs`: enums, nullable values/references,
  nested arrays, ESM clients and malformed payloads.
- `crates/tuffite/tests/framework_mojom.rs`: the same byte fixtures through
  the independently usable Cargo SDK.
- `components/dispatch/test/mojom_codec_unittest.cc`: Chromium's official codec
  serializes and validates those exact fixtures.
- Packaged Playground smoke checks: Browser/Utility dispatch, Framework calls,
  capability allow/deny, forged requests and generic-Utility bypass protection.

Schemas and SDK/Framework binaries must be built together. There is no legacy
TypeScript-IDL protocol, string-enum adapter or old business-global facade.
