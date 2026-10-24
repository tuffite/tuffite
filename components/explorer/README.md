# Tuffite `<explorer>`

`<explorer>` embeds an ordinary, non-Guest `content::WebContents` in a trusted
Tuffite document. It is a same-layer visual surface with its own top-level
browsing context, renderer isolation domain, lifecycle, and browser-enforced
capability policy. It is intended for conversations, extension UI, previews,
and remote pages that must not be able to stall or compromise the app renderer.

The implementation is compiled only with `tuffite_in_chromium = true`. The app
manifest controls document-origin grants and independent Explorer behavior.

## Implementation boundaries

`common/explorer.mojom` defines browser/renderer messages. `browser/` owns
Explorer hosts, per-BrowserContext routing, surface policy state, and storage
snapshot support. `renderer/` supplies document bindings. Tests and fixtures
live in `test/`. `browser/content/` contains the private Content isolation
integration, while `renderer/blink/` contains the Blink element and IDL sources.
These integration sources are still compiled into Chromium Content and Blink
Core respectively, not into the component's ordinary browser/renderer targets.
Each integration directory owns a `tuffite_in_chromium.gni` alongside its
implementation. These files define the source, dependency and IDL/V8 input
lists. Chromium build files import them directly; no mirrored forwarding
files are needed. Generated IDL/V8 paths are unchanged.

The component has no framework dependency. Browser hosts receive an
`ExplorerPolicyProvider` interface; the framework's ExplorerService
implements manifest and application-role rules. BrowserContext owns the router,
and document/WebContents destruction cleans up hosts and surface state.
The manifest example below describes Tuffite framework configuration, rather
than a manifest format parsed by this component.

## Manifest

The configuration separates resource routing, runtime settings and authorization:

| Field | Responsibility |
| --- | --- |
| `app.assets[]` | Map request origin + path to a packaged file or directory. |
| `build.mojomSources` | Mojom source paths; module names come from IDL. |
| `app.windows[]` | Startup state and per-window min/max size constraints. |
| `app.network` | Request timeout, maximum response size and HTTP/2 setting. |
| `app.security.capabilities[]` | Positive operation grants and resource scopes by committed document origin. |

```json
{
  "app": {
    "assets": [
      {
        "origins": [
          "myapp://main"
        ],
        "paths": [
          "/main",
          "/explorer"
        ],
        "file": "ui/dist/index.html"
      },
      {
        "origins": [
          "myapp://main",
          "myapp://*.artifact",
          "myapp://*.preview"
        ],
        "paths": [
          "/assets/*"
        ],
        "directory": "ui/dist/assets"
      }
    ],
    "windows": [
      {
        "label": "main",
        "url": "myapp://main/main",
        "width": 1100,
        "height": 800,
        "minWidth": 640,
        "minHeight": 480,
        "maxWidth": 1920,
        "maxHeight": 1200
      }
    ],
    "network": {
      "timeoutSeconds": 30,
      "maxResponseBytes": 1048576,
      "disableHttp2": false
    },
    "security": {
      "capabilities": [
        {
          "identifier": "app-shell",
          "origins": [
            "myapp://main"
          ],
          "explorer": {
            "create": true,
            "executeJavaScript": true
          },
          "shellApi": [
            "myapp.*",
            "tuffite.storage.*",
            "tuffite.window.current",
            "tuffite.window.list",
            "tuffite.window.create",
            "tuffite.window.getBounds",
            "tuffite.window.focus",
            "tuffite.window.minimize",
            "tuffite.window.maximize",
            "tuffite.window.restore",
            "tuffite.window.setFullscreen",
            "tuffite.window.close",
            "tuffite.window.setTitle",
            "tuffite.window.setBounds",
            {
              "method": "tuffite.fs.readFile",
              "allow": [
                {
                  "path": "resource/*"
                },
                {
                  "path": "data/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.exists",
              "allow": [
                {
                  "path": "resource/*"
                },
                {
                  "path": "data/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.writeFile",
              "allow": [
                {
                  "path": "data/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.createDir",
              "allow": [
                {
                  "path": "data/*"
                }
              ]
            },
            {
              "method": "tuffite.fs.removeFile",
              "allow": [
                {
                  "path": "data/*"
                }
              ]
            },
            "tuffite.path.directories",
            {
              "method": "tuffite.path.resolve",
              "allow": [
                {
                  "path": "resource/*"
                },
                {
                  "path": "data/*"
                }
              ]
            },
            {
              "method": "tuffite.net.request",
              "allow": [
                {
                  "url": "https://api.example.com/*"
                }
              ]
            }
          ]
        },
        {
          "identifier": "artifact-and-preview",
          "origins": [
            "myapp://*.artifact",
            "myapp://*.preview"
          ],
          "shellApi": [
            "tuffite.path.directories",
            "tuffite.storage.get"
          ]
        }
      ]
    }
  },
  "build": {
    "shellApi": {
      "namespace": "myapp",
      "definitions": [
        "shell_api.d.ts"
      ]
    }
  }
}
```

Assets and capabilities match independently. A public application origin can
load an explicitly mapped page without receiving any native API. Each asset
route has nonempty `origins` and `paths` arrays and exactly one `file` or
`directory`, relative to the project directory. Sources are bundled automatically.
Paths are exact or terminal `/*` prefixes; directory routes strip the matching
prefix and append the suffix to the configured directory. Exact routes win,
then the longest prefix. Equal-priority rules with overlapping origins are
configuration errors. There is no implicit root/SPA fallback: unmatched paths
and missing files return 404. The framework owns `/tuffite.js` and
`/tuffite.js`; applications cannot replace those resources. URL and
origin are preserved even when multiple origins map to the same bytes.

Capabilities are selected by the caller's committed document origin. Unmatched
origins receive no native APIs, including the startup document. Multiple matching
capabilities union grants; explicit resource denials take precedence across all
matching entries. `identifier` is a configuration label, not an element role.

- `shellApi` grants both framework and business methods. A string is shorthand
  for `{ "method": "qualified.method" }`; an object can add `allow` and `deny`
  arrays of JSON values. Only exact methods and trailing `.*` namespace patterns
  are supported. Patterns are expanded against the current IDL registry during
  packaging; zero matches are an error. Rebuilding after an API upgrade can
  expand a wildcard grant. Business channels must also match their binary IDs.
- Scope validation belongs to the API module, not the main manifest schema.
  `tuffite.fs.*` and `tuffite.path.resolve` accept `{ "path": "data/**" }`;
  `tuffite.net.request` accepts `{ "url": "https://api.example.com/**" }`.
  These methods default to no allowed resources. Resource writes are forbidden.
  Network redirects repeat the scope check; private endpoints require an exact
  allow declaration. This does not change web fetch/SOP/CSP behavior.
- Window target methods accept `{ "window": "current" | "others" | "*" }`.
  Their default scope is the current window. `create`, `list`, and `current` are
  independent methods without scopes. A created document does not inherit its
  caller's native permissions. Dimensions remain in `app.windows[]`.
- Scope objects must have the shape accepted by every matched method; unknown
  fields, unsupported scopes, and incorrect value types fail validation. An
  explicit `allow: []` allows no resources. Deny-only entries use the method's
  default allow scope and do not implicitly grant all resources. JSON values
  are an extensible carrier, not a generic parameter-matching language.
- `explorer.create` permits creating/attaching owned Explorers with any valid
  partition. Each guest needs its own origin grant to create descendants. No
  configurable depth limit applies. Navigation supports HTTP(S), file, and the
  application scheme without a destination-origin allowlist.
- `explorer.executeJavaScript` permits isolated-world execution in an owned
  Explorer, including cross-origin DOM access. Calls check the current owner
  and the captured element grant. Transfer checks the receiving owner's policy.
- Dialog forwarding defaults to `suppress` and is enabled with `dialogs="emit"`, and
  messages follow actual owner relations. Neither grants guest ShellAPI access.
  Business-wide event subscriptions require all registered business commands.

Origin rules use exact schemes and canonical hosts, optionally a leading `*.`
that matches one or more nonempty host labels and excludes the bare suffix.
HTTP(S) effective ports are significant. Chromium application schemes use
host-only origins, so use distinct hosts rather than ports for isolation.
Paths, credentials, queries and fragments are invalid in origin rules; opaque
origins never match, including `*`. A shared capability or asset mapping does
not merge origins, cookies or DOM access. Ordinary iframes remain ineligible for
the native ShellAPI transport.

Development aliases only capabilities matching the packaged entry origin to
the explicitly configured loopback dev origin. Asset routes continue to serve
the packaged application origins. No implicit grants are added.

`app.frontend`, top-level `app.capabilities`, `assetRoutes`, named Explorer
profiles and element-selected capabilities are not supported.

## Element

```html
<explorer id="browser"
  src="https://example.com/"
  navigation="emit"
  dialogs="emit">
</explorer>
```

| Property | Values and behavior | Mutable after attach |
| --- | --- | --- |
| `src` | Initial/current requested URL; assigning it uses the `load()` path | yes |
| `partition` | omitted, `persist:<name>`, `memory:<name>`, `inherit+persist:<name>`, or `inherit+memory:<name>` | no |
| `navigation` | `emit` intercepts renderer current-tab navigation; `allow` navigates inside | no |
| `dialogs` | `emit` forwards authorized dialogs; `suppress` rejects them | no |
| `contextMenuEnabled` | Enables the browser-owned native context menu | yes |

Creation attributes are frozen for that WebContents. Remove and recreate the
element to change its partition, navigation, or dialog policy.
`partition="shared"` is intentionally invalid: `shared` is an internal partition kind; omission is the public element syntax.

## Storage partitions

| Element value | Effective storage | Initial data |
| --- | --- | --- |
| omitted under a persistent parent | the parent's exact persistent `StoragePartitionConfig` | shared live backend |
| omitted under an in-memory parent | the parent's exact in-memory `StoragePartitionConfig` | shared live backend |
| `persist:name` | independent named disk-backed partition | empty, unless that same named partition already exists |
| `memory:name` | independent named in-memory partition | empty for the lifetime of that partition |
| `inherit+persist:name` | independent named disk-backed partition | creation-time parent cookies + localStorage snapshot when the destination snapshot surfaces are empty |
| `inherit+memory:name` | independent named in-memory partition | creation-time parent cookies + localStorage snapshot when the destination snapshot surfaces are empty |

Thus omission means **share**, not “use the default persistent partition.” An
Explorer nested inside `memory:preview` and omitting its own attribute shares
that in-memory partition. Cookies, localStorage, IndexedDB, cache, and service
workers use the selected backend; normal origin rules still apply, and
sessionStorage remains tied to each top-level browsing context.

`inherit+*` is a seed operation, not a live relationship. The browser checks
destination cookies and localStorage together before the Explorer's first
navigation. If either already has data, it preserves both destination surfaces
and does not copy either one. Otherwise it copies the immediate parent's
cookies and localStorage, then starts navigation. IndexedDB, Cache Storage,
service workers, and sessionStorage are not copied. A later parent write is not
visible in the child, and a later child write is not visible in the parent. For
a persistent name this also means reopening an already-seeded partition keeps
its own state; an in-memory name lasts for that BrowserContext lifetime.

Storage inheritance and renderer isolation are independent axes. Sharing the
parent partition never means sharing its renderer: every Explorer still gets a
stable `EmbedderIsolationInfo::kUniqueInstance` domain. Conversely, selecting
`persist:name` or `memory:name` changes the storage backend but does not weaken
that process boundary.

`getNavigationState().storagePartition` exposes both the requested mode and the
actual Chromium config:

```js
{
  domain, name, inMemory, isDefault,
  requested, // "" when omitted; otherwise the exact partition attribute
  mode,      // "shared" | "persist" | "memory" | "inherit+persist" | "inherit+memory"
  shared,    // true only when the exact parent config is reused
  inherited, // true when an inherit+* policy was requested
  createdDomain, createdName, createdInMemory, createdIsDefault
}
```

Independent partitions require separate renderer processes and are rejected
safely under Chromium's `--single-process` debug mode.

## Parent API

```js
const explorer = document.querySelector('#browser');

explorer.load('https://example.com/account');
await explorer.back();
await explorer.forward();
await explorer.go(-2);
await explorer.reload();
await explorer.reload({bypassCache: true});
await explorer.stop();

const state = await explorer.getNavigationState();
const result = await explorer.executeJavaScript(
  `({title: document.title, href: location.href})`,
  {world: 'isolated', expectedDocumentId: state.documentId},
);
const delivered = await explorer.postMessage({type: 'selection-changed'});
```

Navigation methods resolve to whether the browser accepted the operation.
Native tab controls include `findInPage(text, options)`, `stopFindInPage(action)`,
`setZoomFactor(factor)`, `getZoomFactor()` and `setAudioMuted(muted)`.
Find results arrive through `findresult` and include an accepted request ID,
match count, active ordinal, screen selection rect and final-update flag.
Zoom is native, per Explorer and bounded to 25–500%; zoom/mute persist across
navigation and transfer. `zoomchange`, `audiostatechange`, `faviconchange` and
`renderprocessgone` report the native page state. See the
[tab audit and lab](../../../docs/explorer-tab-parity.md) for semantics and tests.

`getNavigationState()` also exposes `zoomFactor`, `audioMuted`, `audible`,
`crashed` and primary-document `favicons`. Existing `reload()` recovers a crashed
renderer. Favicon entries are metadata only, without an additional fetch.

`getNavigationState()` returns URL/title/history/loading state, a document ID,
renderer diagnostics, and storage diagnostics. Every Explorer has a stable
Chromium `kUniqueInstance` isolation domain, so `rendererProcess.isolated` is
true after commit even when storage is shared. Numeric process IDs are only
diagnostics.

`executeJavaScript()` is capability-gated, limited to 1 MiB, runs in an
isolated world, and requires a JSON-serializable result. Supplying
`expectedDocumentId` prevents execution after a navigation race.
`explorer.postMessage(data)` sends to the direct child; the child receives the
same `window.tuffite` message event described below.

## Events

Structured payloads are available as `event.detail`.

| Event | Detail |
| --- | --- |
| `error` | creation/attachment was rejected |
| `loadstart` | navigation state |
| `loadstop` | navigation state |
| `loaderror` | navigation state plus `errorCode` and `errorDescription` after a rejected or failed load |
| `loadcommit` | navigation state after commit |
| `titlechange` | navigation state after title update |
| `newwindow` | pending original page with attach/discard and cancellation signal |
| `navigation` | intercepted current-page navigation |
| `dialog` | one-shot JavaScript dialog response and cancellation signal |

The request events are `navigation`, `newwindow`, and `dialog`. All are
`bubbles: true`, `composed: true`, `cancelable: true`. Each owner document uses
ordinary DOM capture, target, and bubble phases, including shadow-root retargeting.
After that dispatch, the browser offers an unclaimed request at the next outer
Explorer. The final fallback dispatches on the top app's `window.tuffite`.
This is a new dispatch in each document, not a DOM event path through guest DOMs.
Guest input events never automatically cross the document boundary.

Call `preventDefault()` synchronously to claim a request before asynchronous UI
work. `stopPropagation()` and `stopImmediatePropagation()` only affect that
document's DOM listeners. A response initiated during dispatch also consumes the
request, so it is not offered outward. Passive listeners cannot claim a request.
Synthetic events have no browser request behind them.

`navigation.detail` contains `{url, disposition, userGesture, from}`; it reports an
intercepted current-page navigation. `newwindow.detail` adds `signal`,
`attach(targetExplorer)`, and `discard()`. It covers window.open, popup features,
target=_blank and browser gestures; `disposition` retains the native classification.
Attach adopts the original WebContents, preserving opener, POST, sessionStorage
and its actual partition. It requires a connected target with matching
immutable policy and partition. It never replays the URL. Successful attach and
discard return true; duplicate or stale responses return false. Invalid attach
targets reject with a DOMException. The old `open` and `popup` event names are
replaced by `newwindow`.

`dialog.detail` contains `{type, message, defaultValue, isReload, from, signal,
accept(), dismiss()}`. A dialog event means the guest invoked alert, confirm,
prompt or beforeunload; native UI has not already been shown. The application may
show its own UI and return the JavaScript result using accept/dismiss.

Requests are browser-owned and one-shot. Source navigation/destruction cancels
pending requests and aborts their signals. Destruction of the current claimant
also releases its pending requests. Completion retires all previously offered
controllers. Unhandled top-level new windows are discarded and dialogs dismissed.
Normal completion does not abort the signal; cancellation does. Keeping a JS
wrapper alive does not keep a native pending page alive after its source exits.

State events bubble and are composed, but are not cancelable and are not forwarded
across documents. `loadcommit` describes a committed navigation; `titlechange`
reports title updates. `zoomchange` includes `oldFactor` and `newFactor`.
`fullscreenchange` has `{entered}`; `pictureinpicturechange` has
`{type: "video" | "document", entered}`. `renderprocessgone` includes a stable
`reason` plus the diagnostic native terminationStatus. Other state events are
loadstart/loadstop/loaderror, attached/detached/error, download, findresult,
audiostatechange and faviconchange. Both element and surface message events use
`{data, from}`; explicit message routing is independent of DOM propagation.

```js
window.tuffite.addEventListener('newwindow', event => {
  event.preventDefault();
  const target = document.createElement('explorer');
  target.navigation = 'allow'; // match the source's immutable policy
  document.querySelector('#tabs').append(target);
  void event.detail.attach(target);
});
explorer.addEventListener('dialog', event => {
  event.preventDefault();
  const request = event.detail;
  showConfirm(request.message, {signal: request.signal}).then(yes => {
    if (yes) void request.accept(); else void request.dismiss();
  });
});
```

Playground `events.html` and `smoke.py --explorer-events` exercise this contract.

## Dialogs

`alert`, `confirm`, `prompt`, and `beforeunload` synchronously wait in the
Explorer renderer while the parent renders asynchronous custom UI. The event's
`ExplorerDialog` detail is therefore a one-shot controller: `accept()` or
`dismiss()` completes the exact browser-owned request that produced the event.
Its opaque request ID stays internal, so application code cannot accidentally
answer a newer dialog with stale copied fields. A second response resolves to
`false`. Navigation, destruction, or a newer dialog cancels the old request.

```js
explorer.addEventListener('dialog', async event => {
  event.preventDefault();
  const dialog = event.detail;
  const response = await showAppDialog(dialog);
  if (response.accept) {
    await dialog.accept(response.text ?? '');
  } else {
    await dialog.dismiss();
  }
});
```

## Cross-layer messaging

App and Explorer main documents share one framework object. The two transports
have intentionally distinct verbs: `invoke` is the binary ShellAPI bridge,
while `postMessage` is the JSON Explorer-tree bridge. Generated business APIs
normally hide `invoke` behind typed methods such as
`window.playground.commands.doSomething("hello")`. Explorer messages and ShellAPI channel
events use distinct internal EventTargets on the same root object; their
initialization order is irrelevant.

```js
await window.tuffite.postMessage({type: 'ready'}, {to: 'parent'});

window.tuffite.addEventListener('message', async ({detail}) => {
  const {data, from} = detail;
  if (data.type === 'notification-request') {
    showNotification(data.notification);
    await window.tuffite.postMessage({
      type: 'notification-result',
      requestId: data.requestId,
      shown: true,
    }, {to: from});
  }
});
```

The Explorer message API is:

- `postMessage(data, {to = 'top'}) -> Promise<boolean>`
- `addEventListener('message', listener)` and `removeEventListener(...)`; the
  listener receives a `CustomEvent` whose detail is `{data, from}`

`to` accepts `parent`, `top`, or an opaque browser-issued ref received as
`event.detail.from`. The trusted top surface may address any descendant in its tree, and
a direct parent may address its child. Routes, tree membership, directions, and
ownership are validated in the browser. `postMessage()` resolves to whether the
browser accepted and queued/delivered the message; it does not imply a reply.
Applications that need request/response semantics use ordinary message types
and IDs, and own their timeout and authorization policy. Tuffite does not add an
Explorer RPC envelope.

The low-level ShellAPI transport is separately:

```js
await window.tuffite.invoke(route, binaryPayload); // Promise<Uint8Array>
const listener = ({detail}) => {
  if (detail.channel !== 'app:changed') return;
  const payload = detail.data; // Uint8Array
};
window.tuffite.addEventListener('channel', listener);
// Later:
window.tuffite.removeEventListener('channel', listener);
```

It accepts no `to` option and never routes through the Explorer tree. An
Explorer receives it only when its current committed origin has ShellAPI grants.

Runtime application events use `tuffite.addEventListener('channel', ...)` and
carry `{channel, data}` in `detail`. Explorer tree delivery uses
`tuffite.addEventListener('message', ...)`. Thus a business channel literally
named `message` still cannot collide with Explorer messaging. There is no
public `tuffite.channel` namespace; app facades such as `window.playground` may
use that namespace normally.

Nested Explorers use the same element and normal-WebContents path:

```html
<explorer
  navigation="emit" dialogs="suppress"></explorer>
```

Here the omitted partition shares the immediate parent's actual partition,
including when the parent uses `memory:name`. This enables deeply nested
content to message the top app surface without relying on intermediate renderer code.

## Context menu

Browser-owned menus are off by default and can be toggled at runtime:

```js
explorer.contextMenuEnabled = true;
```

The menu includes history/reload, applicable editing actions, opening links
through the normal `open` interception path, image save/copy/address actions,
video Picture in Picture, and Inspect Element. It is owned by
the top-level Views widget; disabling it does not suppress page DOM
`contextmenu` events.

## Architecture and lifecycle

The element creates an `about:blank` child-frame placeholder. The browser
validates that it is a direct child, creates a normal WebContents with the
selected `StoragePartitionConfig`, then uses
`PrepareForInnerWebContentsAttach()` and `AttachInnerWebContents()`. Composition
uses Chromium's `WebContentsViewChildFrame` path, never GuestView.

The initial `SiteInstance` is constructed with both the final partition and
the Explorer's stable isolation id before `WebContents::Create()`. For a named
non-default partition, its `BrowsingInstance` is also marked fixed-partition
from the first navigation. This ordering is required: adding the isolation
marker only after WebContents creation can make the first navigation replace
the BrowsingInstance, lose the fixed-partition invariant, and later compare a
named partition against the default partition. A shared default partition does
not need the fixed flag, but still receives its isolation id before creation.

Removing the element destroys its controller and WebContents. Reinsertion or
replacement creates a new browsing context and document generation. Owner
document replacement invalidates late Mojo callbacks. Recreating the element
is the supported session-switch and recovery operation.

Chromium-owned integration is guarded by `tuffite_in_chromium` /
`BUILDFLAG(TUFFITE_IN_CHROMIUM)`. Feature implementation is under
`src/tuffite/components/explorer/`; Content and Blink integration sources are
colocated with their `tuffite_in_chromium.gni` files in `browser/content/` and
`renderer/blink/`, imported directly by the corresponding Chromium targets.

## Playground verification

The Playground Capability Lab covers parent navigation APIs, isolated-world
execution, all four request event types, dialog response, nested top messaging,
multi-level request fallback and `preventDefault()`, renderer isolation,
omitted/shared storage, independent `persist` / `memory`, snapshot-seeded
`inherit+persist` / `inherit+memory`, lifecycle stress, and the native context
menu on Baidu.

```console
mise exec -- bun run --cwd playground/ui build
mise exec -- python3 playground/scripts/smoke.py --keep-files
```

Run the smoke command from the outer workspace after packaging the application;
see [native architecture and validation](../../README.md) for the build and
package sequence. `mise run playground_explorer_smoke` exercises
the interactive Capability Lab against an explicitly supplied package root.

Production smoke should additionally cover owner navigation, renderer failure,
origin authorization, dialog cancellation, nested ownership, and shutdown while a menu
or dialog is open.

## Resource downloads and previews

The Framework implements Chromium downloads for Explorer navigation, page links,
blob resources and image context menus using the originating storage partition.
It writes each item to `<profile>/Downloads/<guid>/<filename>` with a separate
`.crdownload` intermediate file. Equal filenames never overwrite one another.
No executable is opened automatically. Directory and write failures emit an
interrupted state rather than silently dropping the action.

```js
explorer.addEventListener('download', ({detail}) => {
  // id, url, filename, path, mimeType, state, receivedBytes, totalBytes,
  // error (Chromium interrupt reason), navigation (active PDF navigation)
  showDownload(detail);
});
await explorer.download(resourceUrl); // accepted by the browser, not completed
await explorer.cancelDownload(id);
const items = await explorer.getDownloads(); // including after attach()
const base64 = await explorer.readDownload(id);
const pngDataUrl = await explorer.capturePreview();
```

`readDownload()` accepts only a completed item owned by this exact WebContents,
including after transfer, and is bounded to 64 MiB. Other Explorers cannot read
or cancel it. `capturePreview()` returns a compositor PNG data URL bounded to
1280 × 960, or an empty string when a surface is unavailable.

This Content embedder does not ship Chrome's PDF extension. A PDF navigation
therefore emits a download with `mimeType: 'application/pdf'` and
`navigation: true`. Applications can render the downloaded bytes with their own
PDF viewer, preserving cookies and avoiding a second cross-origin request.
Leaving the PDF clears its active navigation marker; historical downloads are
still listed but cannot replace a later page's viewer.

Standard video PiP uses the native video overlay. Document PiP creates a related,
resizable floating WebContents, preserving the opener relationship while keeping
ShellAPI authorization origin-based. It closes when the source navigates or is
destroyed. Ordinary popup interception remains independent of Document PiP.

The native resources regression runs with:

```console
mise exec -- python3 playground/scripts/smoke.py --package dist/playground --explorer-resources
```

### Exclusive page handles

`detach()` returns `ExplorerHandle`; `attach(handle)` consumes it and removes
`src` on success. `win.transfer(name, handle)` hands it to the newly created
window's initial document. `<explorer src="explorer:name" defer>` waits empty
for that resource and adopts it once. Names are document-local, not bearer
credentials. See [the ownership contract](../../docs/explorer.md) for
replacement rollback, policy compatibility and lifetime rules.

### Host-controlled DevTools policy

The package can disable inspection by document origin, including top-level
application windows. Add a rule to `app.security.capabilities` in `tuffite.json`:

```json
{
  "identifier": "no-inspection",
  "origins": ["ovule://-", "https://private.example.com"],
  "devtools": false
}
```

An origin without a matching denial retains the default inspection behavior.
If multiple capabilities match, any `devtools: false` takes precedence over
`true` or an omitted field. The policy gates both Inspect element and new CDP attachments;
existing inspectors detach when the inspected top-level document navigates to
a denied origin. Development-server aliases inherit the entry origin's policy.
This setting does not disable the debugging server itself, so allowed origins
can still be inspected through an explicitly enabled server.


Rust hosts can add the same denials before any native services or windows exist:

```rust
#[tuffite::browser]
impl tuffite::Browser for App {
    fn setup(builder: tuffite::browser::Builder) -> tuffite::ApplicationResult<tuffite::browser::Builder> {
        Ok(builder
            .disable_devtools_for_origin("ovule://-")
            .disable_devtools_for_origin("https://*.private.example.com"))
    }

    fn new(_: tuffite::BrowserContext) -> tuffite::ApplicationResult<Self> {
        Ok(Self)
    }
}
```

`Browser::setup` runs once in the browser process before `Browser::new`.
Its origin patterns are validated by the native manifest loader; invalid rules
or setup errors abort startup. Rust and manifest denials are combined, and
renderer scripts cannot override either. There is no Explorer `devtools`
attribute or property.

Import native element and event types from the npm SDK instead of copying them:

```ts
import type { HTMLExplorerElement, ExplorerEventMap } from "tuffite/explorer";
const explorer: HTMLExplorerElement = document.createElement("explorer");
```
