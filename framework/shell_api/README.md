# Shell API protocol

This directory owns the Framework's language-neutral Shell API definitions and
resource scope registry. The definitions generate C++, Rust, JavaScript, and
TypeScript bindings for web content and native applications.

Browser command implementations and route registration live in
`framework/browser/services/shell_api/commands`. `ShellApiService` owns
BrowserContext authorization, request cancellation, event subscriptions, and
shutdown. Both that service and the native ABI bridge reuse the commands target.
Commands depend on manifest, utility, window, and remoting services, not on
`ShellApiService`.

## Directory layout

```text
framework/shell_api/
  BUILD.gn                    :protocol code generation
  README.md
  <domain>.d.ts               Types and method definitions
  scope_registry.json         Resource permission metadata
framework/browser/services/shell_api/
  BUILD.gn                    :shell_api service interface/factory/implementation
  shell_api_service*.{h,cc}
  commands/
    BUILD.gn                  :commands shared native handlers
    registry.{h,cc}           Registry and BuiltinRoutes()
    manifest.h     Manifest lookup helper
    <domain>.{h,cc}           Handler declarations and implementations
```

`.d.ts` files currently in this directory:

- `common.d.ts` — cross-domain type aliases (`BaseDirectory`)
- `dialog.d.ts` — file / directory pickers
- `fs.d.ts` — read/write/exists/createDir/removeFile
- `log.d.ts` — trace/debug/info/warn/error
- `network.d.ts` — HTTP request
- `path.d.ts` — base directory + path resolution
- `storage.d.ts` — key/value get/set/remove/clear
- `remoting.d.ts` — CRD sessions, input, capture and signaling
- `utility.d.ts` — utility-process invoke/cancel
- `window.d.ts` — window CRUD + focus/minimize/maximize/etc.

Generated C++ output mirrors the `.d.ts` source directory under
`root_gen_dir`, with exactly one pair per input:

```
<stem>.{h,cc}                  Types + endpoints + reflection + route
                               trampolines + reflection const definitions
```

Stable `Method` and primitive codec support lives in
`components/dispatch/browser/shell_api_protocol.{h,cc}` rather than generated
output.

Language bindings use the same per-schema split. The generation phase of
`tuff build` writes the
checked-in JavaScript and TypeScript files under the package or application's
`generated/` directory; `build.rs` writes Rust into Cargo's `OUT_DIR`:

```
generated/index.{js,d.ts}      Aggregate entry points
generated/<stem>.js            Per-domain route/codec metadata
generated/<stem>.d.ts
                               Portable declarations copied from that schema
$OUT_DIR/shell_api.rs           Aggregate Rust bindings consumed by the crate
$OUT_DIR/generated_<stem>.rs    Per-schema Rust fragments for tooling/composition
```

The aggregate JavaScript remains self-contained so the browser can inject one
runtime resource. The aggregate TypeScript declaration intersects the local
`generated_<stem>.d.ts` modules and never imports files from the source tree.
Cross-file declaration dependencies are emitted as local `import type`
statements. Rust fragments share the aggregate module root, so a domain can
refer to a common type with the same paths used by `shell_api.rs`.

## Generated C++ surface

Every domain gets its own generated header/source and lives inside
`tuffite::shell_api::generated::<domain>`. For each function `<name>` declared
in the domain's `.d.ts`, the generator emits:

- `struct <Name>Request` — one field per parameter
- `using <Name>Result = ...` — the promise's resolution type
- `using <Name>Callback = base::OnceCallback<void(rpc::Status, <Name>Result)>`
- `using <Name>Method = Method<<Name>Request, <Name>Result>`
- `namespace reflection { extern const <Name>Method <name>; }`

The shared `generated::Method<Request, Result>` template exposes three
`Bind()` overloads so a handler can be registered as:

- A plain function pointer `void(Request&&, Callback&&)`
- A context function pointer
  `void(const dispatch::Context&, Request&&, Callback&&)` — Tuffite builds the
  context from the incoming frame or native Application identity
- A `base::RepeatingCallback<void(const dispatch::Context&, Request&&, Callback&&)>`

The generated route trampoline decodes the wire payload, invokes the handler,
and encodes the reply. Encode/Decode helpers for interfaces and enums live in
each domain's `.cc`; primitives and `optional<T>`/`vector<T>` live in the
shared base.

## Browser command implementations (`commands/`)

Each domain has a matching `commands/<domain>.h` header declaring the handler
signatures and a `commands/<domain>.cc` implementing them in
`tuffite::shell_api::<domain>`. Two flavors are supported:

Handler without caller context (e.g. `log`):

```cpp
void Trace(generated::log::TraceRequest&& request,
           generated::log::TraceCallback&& callback);
```

Handler with caller context (e.g. `fs`, `dialog`, `network`, `path`,
`storage`, `window`):

```cpp
void ReadFile(const ::tuffite::dispatch::Context& context,
              generated::fs::ReadFileRequest&& request,
              generated::fs::ReadFileCallback&& callback);
```

`dispatch::Context` has either a borrowed Renderer frame (`sender`) or a native
`ApplicationContext` (`native_application`), plus a document/application
owner token. Router dispatch and generated trampolines preserve this identity.
There is no route-level caller classification: handlers enforce their own actual
requirements. Frame-dependent handlers reject a missing frame; Remoting and
Utility resolve the service context for either caller. `injectInput` explicitly
rejects Renderer callers before contacting CRD.

Context pointers are valid only during synchronous handler invocation.
Asynchronous handlers must copy values or capture an appropriate weak identity;
they must not retain the Context or dereference its frame across an async boundary.
Renderer manifest/origin checks use `ManifestFor()` in `manifest.h`;
file operations continue through the manifest's path/symlink policy.

Every handler completes asynchronously by invoking its callback, e.g.:

```cpp
void GetDiskInfo(GetDiskInfoRequest&& request,
                 GetDiskInfoCallback&& callback) {
  base::ThreadPool::PostTaskAndReplyWithResult(
      FROM_HERE, {base::MayBlock()},
      base::BindOnce(&base::SysInfo::AmountOfFreeDiskSpace,
                     base::FilePath::FromUTF8Unsafe(request.path)),
      base::BindOnce(
          [](GetDiskInfoCallback callback,
             std::optional<int64_t> free_disk_space) {
            DiskInfo info;
            info.freeDiskSpace =
                static_cast<double>(free_disk_space.value_or(-1));
            std::move(callback).Run(kStatusOk, std::move(info));
          },
          std::move(callback)));
}
```

## Registry

`registry.{h,cc}` owns three things:

1. Reply status constants (`kStatusOk`, `kStatusInvalidArgument`, ...).
2. The `Registry` builder plus a free `Register()` overload that pairs a
   reflection variable with an implementation.
3. `BuiltinRoutes()` — the sole route-table entry point consumed by
   `ShellApiServiceImpl`.

New builtin routes are added by editing `registry.cc`:

```cpp
Register(registry, generated::system::reflection::getDiskInfo,
         &system::GetDiskInfo);
Register(registry, generated::system::reflection::writeClipboard,
         &system::WriteClipboard);
```

The free-function form and chained `registry.Register(...)` form are
interchangeable; pick whichever reads best in context.

## Build integration

The GN template `shell_api_protocol.gni` wraps everything into a single dep:

```gn
import("//tuffite/build/shell_api_protocol.gni")

shell_api_protocol("protocol") {
  manifest = "//tuffite/packages/tuffite/tuffite.json"
  definitions = [
    "//tuffite/framework/shell_api/common.d.ts",
    # ...remaining definitions in manifest order...
  ]
  visibility = [ "//tuffite/framework/browser/services/shell_api/commands" ]
}
```

Internals:

- The template derives stems and the mirrored output directory from its
  explicit `.d.ts` inputs. The native build backend only ensures the repository-relative host
  generator exists before GN generation.
- Declares an `action` that runs `//tuffite/build/shell_api_protocol.py` with
  the resolved repository-local generator executable.
- Emits a `source_set` that lists every generated file, adds
  `include_dirs = [ root_gen_dir ]` via a `public_config`, and pulls in the
  Dispatch implementation containing the typed protocol support.

Native commands depend on `//tuffite/framework/shell_api:protocol`.

## Adding a new domain

1. Add `<domain>.d.ts` next to the existing schemas.
2. Add `"../../framework/shell_api/<domain>.d.ts"` to
   `packages/tuffite/tuffite.json` under
   `build.shellApi.definitions`.
3. Create `framework/browser/services/shell_api/commands/<domain>.{h,cc}` with the handler declarations/definitions.
   Follow the context / no-context pattern above.
4. Add the schema to `framework/shell_api/BUILD.gn` and the implementation
   files to `framework/browser/services/shell_api/commands/BUILD.gn`.
5. Update the commands `registry.cc` to include the generated protocol header
   and manual command header and add `Register(registry, ...)` calls for
   each new endpoint.

The next `gn gen` derives the new output names from that definition list.

## Adding a new method to an existing domain

1. Add the function declaration to the domain's `.d.ts`.
2. Add a handler declaration in `commands/<domain>.h` and its implementation in
   `commands/<domain>.cc`.
3. Add one `Register(registry, generated::<domain>::reflection::<name>,
   &<domain>::<Name>);` line to `registry.cc`.

## Wire status vocabulary

Handlers respond with strongly typed `rpc::Status` constants. The generated
trampoline converts them to `int32_t` only at the Dispatch wire boundary.
Preferred usage in handlers:

- `kStatusOk` — success (the only status that carries an encoded result).
- `kStatusInvalidArgument` — the payload / request contents were malformed.
- `kStatusAccessDenied` — the manifest, origin, or role does not permit the
  call.
- `kStatusUnavailable` — the resource is temporarily unreachable or the
  operation failed (I/O errors, backend timeout, etc.).

## Native Rust requests

Generated endpoint request types implement `tuffite::ShellApiRequest`, including
the channel, argument encoding and response type. `context.shell_api().invoke`
uses the same binary codec as the JavaScript caller and C++ route trampoline.
Model serde support is available for application storage/transport; native calls
use binary encoding, not JSON.

The SDK exports checked-in builtin bindings under `tuffite::shell_api`. After
editing schemas or the generator, refresh them with:

```sh
cargo run -p tuffite --bin tuff -- framework generate-rust packages/tuffite/tuffite.json --output crates/tuffite/src/shell_api_generated.rs
```

Use `--check` in CI; `x.py test --suite framework` includes this check.
Native acceptance uses ABI Status; asynchronous replies use the RPC status
vocabulary below. Each accepted call owns exactly one completion.
