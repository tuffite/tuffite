# DOM isolated worlds

`tuffite.isolated` hosts Web Components in Blink isolated worlds in the current
main-frame document. Worlds share its underlying DOM, layout, renderer thread
and process. They have separate JavaScript globals, DOM wrappers and scoped
custom-element registries. This is not a restricted DOM or process sandbox.

## Dynamic native registration

Retain `context.isolated()` from `Browser::new`, then perform registrations in
`Browser::initialize` or later, after native services become available.
Applications install and validate third-party packages, then register
immutable ESM package snapshots through `BrowserContext::isolated()`:

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

`WorldDefinition::new(name, source)` creates a single-entry ESM package.
For a module graph, use `WorldDefinition::package(name, "entry.js")` with
`.resource(path, bytes)`, or `WorldDefinition::from_directory(name, directory,
"entry.js")` after application package validation. Files are snapshotted before
registration; the native control plane accepts binary-safe resource bytes,
not arbitrary paths or remote URLs. Installation, signatures and user permissions
remain application responsibilities.

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
→ generated Mojo dispatch → `IsolatedHost::Load` → `IsolatedSourceProvider::LoadForFrame` → `IsolatedService::LoadForFrame`.
`isolated_host.{h,cc}` lives in `components/isolated/browser/`. Following the
`DispatchHost`/`RpcHandler` and `ExplorerHost`/`ExplorerPolicyProvider` pattern,
it only depends on a component-level `IsolatedSourceProvider` interface;
Framework's `IsolatedService` implements it. The composition root injects a weak
provider resolved from the frame's BrowserContext. Component code has no
Framework dependencies.
It is referenced through the receiver registered by `ContentBrowserClient`, so
there is no direct browser C++ call to its `Load` override.

`IsolatedHost` owns document-scoped Mojo lifetime. The profile service checks the
actual primary main frame, browser-owned App or Explorer policy, and committed
origin before pinning an immutable package and returning its entry URL. The
normal asset loader resolves subsequent package requests for that document.
Disposal, failed startup and document teardown release the lease. Subframes and
unclassified surfaces are denied.

## Application-owned Web assets

Keep JS/TS, CSS and images in one package directory. The entry is a native ES
module: static and dynamic `import`, cyclic dependencies, JSON modules and
`import.meta.url` use Blink's standard loader and per-world module map.
`createWorld()` resolves only after the entry module graph finishes evaluation,
including top-level await. Evaluation failures reject with a message-only error
and release the partially created world and package lease.

```js
import sheet from './card.css' with { type: 'css' };
import data from './weather.json' with { type: 'json' };
const extra = await import('./details.js');
const icon = new URL('./icon.svg', import.meta.url);
// After attachShadow(...), using the world's registry:
this.root.adoptedStyleSheets = [sheet];
```

CSS module scripts produce a `CSSStyleSheet`; they do not inject styles into the
host document. Standard CSS module scripts do not support CSS `@import`. Import
multiple stylesheets from JS and adopt them together, or resolve CSS imports at
build time. CSS-relative image/font URLs resolve against the stylesheet URL.
Ordinary side-effect CSS imports and `?inline` are bundler conventions, not
native module syntax.

Playground's [`ui/public/isolated/`](../playground/ui/public/isolated/) uses plain
browser-standard modules and assets. Its existing public-resource staging copies
that tree into `ui/dist/playground-public/isolated/`; no isolated-specific Vite
build, IIFE, Rust `include_str!`, resource-inlining plugin or asset URL scheme is
required. TypeScript/npm projects can emit ESM and chunks with their existing
bundler. Preserve relative paths and import attributes in those outputs. Modules
must not import the host ShellAPI clients or its dev-server/HMR runtime.

Declare application-owned packages in Rust startup configuration:

```rust
configuration.app.isolated_worlds.insert(
    "weather".into(),
    tuffite::configuration::IsolatedWorld {
        directory: "ui/dist/playground-public/isolated".into(),
        entry: "card.js".into(),
        origins: vec!["playground://main".into()],
    },
);
```

`directory` is relative to packaged `assets/`; `entry` is relative to that
package. `bundle.resources` is the explicit packaging inventory. The
`beforePackageCommand` builds/stages frontend outputs after Cargo. Runtime
origin declarations stay out of source `tuffite.json`. Native startup snapshots
regular package files, rejecting unsafe paths and symlinks. Limits are 1024 files,
8 MiB per file, 32 MiB total registered resource bytes and 32 definitions.

Each accepted load gets an unguessable, document-bound resource URL under the
existing application scheme. The profile service pins the complete immutable
package version, so later dynamic imports and assets cannot mix replacement
versions. Its resource loader rechecks the owning document and profile; another
frame cannot reuse the URL. CSP restricts script/style/image/font and resource
fetches to that package prefix. Data/blob images and inline styling retain their
existing allowance. Ordinary document CSP also governs DOM resource effects.
No remote imports, document/iframe navigation or Worker entry loading is granted.

Disposal releases the package lease; the document-scoped host also releases all
leases on navigation/disconnection. Accepted in-flight resource requests retain
their copied bytes. Existing worlds remain pinned after registration replacement
or removal until disposed. Disposed code can no longer load additional resources.

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

`createWorld` resolves after entry-module evaluation and registration, including top-level await.
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
eval and requests outside its immutable package; inline styling and data/blob images are
allowed. This policy does not restrict all side effects through shared DOM.

A world can read/change reachable host DOM, observe inputs, and create misleading
UI. DOM-triggered actions use document policies and may execute page scripts
where the host CSP permits them. Treat granting this facility as granting
Content-Script-style document access, not as a capability-only sandbox. Do not
place secrets in the shared document. JS engine exploits, infinite loops and
memory pressure require stronger isolation. Revoking native channels is not a
hard termination guarantee. Use Explorer or remote rendering for narrower trust.

## Playground proof

Playground declares two packaged worlds using the same native ESM component.
Its separate dynamic-registration probe replaces an obsolete script, checks
stale-generation rejection, and unregisters a removed entry. The UI smoke
verifies that the removed entry cannot be created. The two UI worlds register
the same tags, receive isolated property updates and request results, and can
be disposed interactively. Open the Isolated Web Components lab after building
local Framework and Playground:

```sh
mise exec -- python3 playground/scripts/smoke.py \
  --package dist/playground --isolated-worlds --keep-files
```

The CDP smoke inspects closed shadow roots without adding a production DOM
backdoor. It checks registration, nested upgrade, separate globals/registries,
setter updates, callback world, request/reply and idempotent disposal/revocation.
It also checks static/dynamic imports, top-level await, JSON and CSS modules,
relative CSS/image URLs, actual image decoding and rejected out-of-package imports.
