# DOM isolated worlds

`tuffite.isolated` hosts Web Components in Blink isolated worlds in the current
main-frame document. Worlds share its underlying DOM, layout, renderer thread
and process. They have separate JavaScript globals, DOM wrappers and scoped
custom-element registries. This is not a restricted DOM or process sandbox.

## Dynamic native registration

Retain `context.isolated()` from `Browser::new`, then perform registrations in
`Browser::initialize` or later, after native services become available.
Applications install and validate third-party packages, then register immutable,
self-contained classic-script snapshots through `BrowserContext::isolated()`:

```rust
let isolated = context.isolated();
isolated.register(
    tuffite::isolated::WorldDefinition::new("io.example.weather", script_snapshot)
        .allow_origin("ovule://main"),
    |result| {
        let registration = result.expect("registration failed");
        // Retain registration for replace/unregister; notify the UI after success.
    },
)?;
```

`script_snapshot` is an owned UTF-8 String read from an application-validated
package (or embedded with `include_str!`). Registration performs no file or
network loading. Package installation, signatures, extension manifests and user
permissions belong to the application. Compile the UI entry into a self-contained
classic script; ESM imports and arbitrary resource loading are not provided.

After the native completion callback succeeds, the authorized page uses the
same `createWorld({name: 'io.example.weather'})` API. No application restart or
`tuffite.json` edit is needed. The registration control plane is native-only;
it uses the independently queried `TUFFITE_CAPABILITY_ISOLATED` ABI table,
not ShellAPI routing. Old native Frameworks report this capability as unavailable.
`isolated.is_available()` allows checking support before submission.
Operations run on the browser sequence, so completion callbacks must not block it.

`replace(&registration, definition, callback)` atomically replaces the exact
generation under the same name and returns a new Registration. Duplicate
registration, stale replace/unregister, and changes to bundled definitions are
rejected with `InvalidArgument`. `unregister(&registration, callback)` prevents
future loads. Generation numbers never repeat within a BrowserContext.
Dropping or cloning a Registration does not change registry ownership.

Existing worlds retain the source version evaluated when their load was accepted.
Replacement/unregistration does not terminate their JavaScript, cancel request
handlers or remove UI. Applications must dispose tracked world handles and revoke
application capabilities when disabling/uninstalling an extension. A concurrent
load accepted before unregistration may still finish; coordinate creation and
uninstallation in the application. Registrations are memory-only and BrowserContext
scoped; reload the installed package inventory on application startup.

Names are 1–128 ASCII characters, start with a letter, and otherwise contain
letters, digits, `_`, `-`, or `.`. Dynamic definitions require 1–32 exact origins
(no wildcards), at most 2048 bytes each. The browser enforces 32 total definitions,
8 MiB per source and 32 MiB aggregate source bytes, including bundled entries.
The authenticated development entry override inherits grants for the packaged
entry origin, including for registrations added after startup.

## Browser service lifecycle

`IsolatedServiceFactory` is a `BrowserContextKeyedServiceFactory`. Each profile
owns an independent registry; renderer DocumentService instances resolve the
service from their frame's BrowserContext and hold WeakPtrs. The native Browser
client resolves `IsolatedServiceFactory` from the main BrowserContext on each
UI-sequence operation. The generic ABI host stores only a BrowserContext WeakPtr;
it holds no isolated service reference.
No registry is stored in ApplicationContext. Service shutdown releases snapshots
and invalidates document-provider WeakPtrs. BrowserContext destruction invalidates
the native context WeakPtr; accepted native calls complete or are cancelled by
the bridge's common shutdown queue.
The renderer lookup path is:
`createWorld` → `IsolatedBindings::Load` → `mojo::Remote<mojom::IsolatedHost>::Load`
→ generated Mojo dispatch → `IsolatedHost::Load` → `IsolatedSourceProvider::SourceForFrame` → `IsolatedService::Source`.
`isolated_host.{h,cc}` lives in `components/isolated/browser/`. Following the
`DispatchHost`/`RpcHandler` and `ExplorerHost`/`ExplorerPolicyProvider` pattern,
it only depends on a component-level `IsolatedSourceProvider` interface;
Framework's `IsolatedService` implements it. The composition root injects a weak
provider resolved from the frame's BrowserContext. Component code has no
Framework dependencies.
It is referenced through the receiver registered by `ContentBrowserClient`, so
there is no direct browser C++ call to its `Load` override.

`IsolatedHost` owns document-scoped Mojo lifetime. The profile service checks the
actual frame's App/main-frame role and committed origin before returning source.

## Bundled registration shorthand

For application-owned components, the manifest remains optional startup sugar.
Its snapshots enter the same browser registry and reserve their names:

```json
{
  "app": {
    "isolatedWorlds": {
      "weather": {
        "entry": "src/isolated/card.js",
        "origins": ["playground://main"]
      }
    }
  }
}
```

Entries are relative to the packaged `assets/` directory; the application CLI
copies them from the same relative project path. Native startup rejects parent
traversal, symlinks, invalid origins, non-UTF-8 sources, entries above 256 KiB,
and aggregate sources above 1 MiB (at most 32 definitions). The browser snapshots
sources at startup. Only authorized App main-frame documents can load them;
Explorer documents cannot, even with the same origin. Development aliases follow
the packaged entry origin's grants. Renderer JavaScript cannot register a package or supply
an arbitrary entry URL/source through this API.

## Main world

```js
const world = await tuffite.isolated.createWorld({name: 'weather'});
const view = await world.mount(container, {
  tag: 'weather-card',
  properties: {data: {city: 'Shanghai'}, lang: 'zh-CN'},
  onRequest: async ({method, params, signal}) => {
    // Bind authorization to this mounted instance, not IDs in params.
    if (method !== 'refresh') throw new Error('Unsupported operation');
    return refreshWeather(params, signal);
  },
});
await view.update({data: {city: 'Beijing'}});
await view.dispose();
await world.dispose();
```

`createWorld` resolves after synchronous entry evaluation and registration.
Each call creates a fresh world; the application can cache the handle. At most
32 worlds may be active in one document. A world can mount multiple instances.
Types are exported from `tuffite/isolated`: `IsolatedHost`,
`IsolatedWorldHandle`, `IsolatedViewHandle`, `IsolatedWorld`, and
`IsolatedViewContext`. The optional module export `isolated` refers to the same
native global and has a host/world union type.

`mount` appends its own node and closed ShadowRoot without replacing existing
container children. The component and its registry belong to the target world.
Initial properties are assigned before connection. `update` assigns a top-level
patch in that world: omitted keys are unchanged, null assigns null, no deep merge.
Only JSON values cross the boundary; functions, DOM objects and custom instances
are rejected. Patches reject prototype keys, event properties and raw HTML sinks.
Payloads are bounded to 256 Ki UTF-16 code units. Completion means property
assignment completed, not paint or completion of asynchronous component work.
Setter failures are not rolled back. Host methods return promises; JavaScript
cannot recover a hung renderer by timing them out.

`onRequest` stays in the main world. The callback receives cloned parameters and
an AbortSignal. Its return is cloned back; undefined returns null. Errors become
message-only Errors, without cross-world Error objects. Requests are routed by
the native caller world and the registered view, not sender-supplied principals.
The limit is 128 pending requests per world. Host request handlers execute in a
microtask after leaving the component's native callback stack.

## Inside the world

```js
const world = tuffite.isolated;
class WeatherCard extends HTMLElement {
  constructor() {
    super();
    this.root = this.attachShadow({
      mode: 'closed', customElementRegistry: world.customElements,
    });
  }
  connectedCallback() {
    this.context = world.context(this);
    // this.context.request('refresh', {city: 'Shanghai'})
    // this.context.signal fires when the mounted instance is disposed.
  }
  set data(value) { /* render using ordinary DOM APIs */ }
}
world.customElements.define('weather-card', WeatherCard);
```

`customElements` is this world's default scoped registry, not the document's
global registry. `window.customElements` retains Chromium's isolated-world
behavior (null). Use explicit `customElementRegistry` for element creation and
shadow roots, including nested components. Direct `new Component()` has no
implicit scoped registry lookup; use `document.createElement(tag, options)`.
Autonomous custom elements are the supported initial contract.

`context(element)` works in connected callbacks and finds the containing mount,
including across nested shadow roots. Unowned elements are rejected. Instances
of the same world share trust; this is not per-message isolation inside a world.
`context.request(method, params, {signal})` supports cancellation. Context signals
abort on view disposal; the world signal aborts on world disposal. `dispose` is
idempotent and revokes communication before extension cleanup. Updating disposed
handles rejects with InvalidStateError. Timers and global listeners created by
components require cooperative cleanup; DOM removal does not stop arbitrary JS.
Old document handles cannot act on a replacement document. Component runtime
errors dispatch a message-only ErrorEvent on the world handle.

Registry access checks remain intact: host/other worlds cannot retrieve its
constructors through document/element/shadow-root registry getters. Custom
properties and methods are world-local; host code must use `view.update()` rather
than assigning custom properties through its own DOM wrapper.

## Security boundary

Only registered Tuffite worlds gain custom-element construction support. Other
isolated worlds and DevTools contexts retain upstream behavior. The world has
no ambient ShellAPI binding. Its own execution CSP disables dynamic scripts,
eval and direct network requests; inline styling and data/blob images are
allowed. This policy does not restrict all side effects through shared DOM.

A world can read/change reachable host DOM, observe inputs, and create misleading
UI. DOM-triggered actions use document policies and may execute page scripts
where the host CSP permits them. Treat granting this facility as granting
Content-Script-style document access, not as a capability-only sandbox. Do not
place secrets in the shared document. JS engine exploits, infinite loops and
memory pressure require stronger isolation. Revoking native channels is not a
hard termination guarantee. Use Explorer or remote rendering for narrower trust.

## Playground proof

Playground registers its two script snapshots from Rust during Browser initialization,
without `app.isolatedWorlds`. Startup replaces an obsolete throwing script with
the working component, checks stale-generation rejection, and unregisters a third
entry. The UI smoke verifies that the removed entry cannot be created. Open the Isolated Web Components lab. Its two worlds register
the same tags, receive isolated property updates and request results, and can
be disposed interactively. After building local Framework and Playground:

```sh
mise exec -- python3 playground/scripts/smoke.py \
  --package dist/playground --isolated-worlds --keep-files
```

The CDP smoke inspects closed shadow roots without adding a production DOM
backdoor. It checks registration, nested upgrade, separate globals/registries,
setter updates, callback world, request/reply and idempotent disposal/revocation.
