# Tuffite `<explorer>`

`<explorer>` embeds a normal, non-Guest Chromium `WebContents` in trusted app
UI. It preserves normal compositor layout, clipping, focus, input, accessibility
and DevTools behavior while giving the embedded document its own browsing
context, renderer isolation domain, lifecycle, and browser-side policy.

This is the architecture used for extension-heavy conversations and browser
previews: a slow or failed Explorer renderer does not share the app renderer,
and renderer-originated navigation/open behavior can be routed to application
workbench tabs.

## Configuration

The configuration separates resource routing, runtime settings and authorization:

| Field | Responsibility |
| --- | --- |
| `app.assets[]` | Map request origin + path to a packaged file or directory. |
| `build.shellApi` | Business namespace, IDL definitions and facade generation. |
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
and missing files return 404. The framework owns `/tuffite-runtime.js` and
`/tuffite-shell-api.js`; applications cannot replace those resources. URL and
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

## Element and parent API

```html
<explorer id="thread"
  navigation="emit" dialogs="emit"></explorer>
```

The `partition` attribute is intentionally omitted above. Omission shares the
parent's exact current `StoragePartitionConfig`, whether that parent partition
is persistent or in-memory. `partition="persist:name"` and
`partition="memory:name"` select independent, non-inheriting named
partitions. `partition="shared"` is not valid element syntax; `shared` is the
manifest capability kind that authorizes omission.

`partition="inherit+persist:name"` and
`partition="inherit+memory:name"` create independent named partitions and,
before first navigation, seed an empty destination with a snapshot of the
immediate parent's cookies and localStorage. This is not live sharing. The
destination's cookies and localStorage are checked as one unit: if either
already contains data, neither is overwritten. IndexedDB, Cache Storage,
service workers, and sessionStorage are not copied.

The parent API is intentionally close to the useful portion of Chrome's
`webviewTag` API:

```js
explorer.load(url);
await explorer.back();
await explorer.forward();
await explorer.go(-2);
await explorer.reload({bypassCache: true});
await explorer.stop();
const state = await explorer.getNavigationState();
const value = await explorer.executeJavaScript(source, {
  world: 'isolated',
  expectedDocumentId: state.documentId,
});
explorer.postMessage(data, { targetOrigin: location.origin });
```

Creation-time `partition`, `navigation`, and `dialogs` values are
immutable for a WebContents. Recreate the element to change them or to switch a
conversation session. `contextMenuEnabled` remains mutable.

## Events and dialogs

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

```js
explorer.addEventListener('dialog', async ({detail: dialog}) => {
  if (dialog.type === 'prompt') {
    await dialog.accept('value');
  } else {
    await dialog.dismiss();
  }
});
```

The request identity remains browser-internal. This prevents stale or
concurrent custom UI from responding to the wrong `alert`, `confirm`, `prompt`,
or `beforeunload`; a second response resolves to `false`.

## Nested communication

Explorer documents expose their real embedding windows through `window.parent`
and `window.top`. Ordinary iframes inside an Explorer follow the same composed
ancestry. Detached documents have no embedding parent; attaching an existing
page updates its ancestry without reloading it.

```js
window.parent.postMessage(data, { targetOrigin: location.origin });
window.top.postMessage(data, location.origin, [port]);
window.addEventListener('message', event => {
  if (event.origin !== location.origin) return;
  // event.source is the sender's native WindowProxy.
  event.source?.postMessage({ type: 'reply' }, event.origin);
});
```

`explorer.contentWindow` exposes the guest WindowProxy.
`explorer.postMessage(...)` delegates to that window, while
`window.tuffite.postMessage(...)` delegates to `window.top.postMessage(...)`.
Both accept the standard positional `(message, targetOrigin, transfer)` and
`(message, { targetOrigin, transfer })` forms, return `undefined`, and use native
structured clone. Omitted targetOrigin defaults to `/` (the sender's origin).
Invalid target origins and uncloneable data throw synchronously. A target origin
is a delivery filter, not an address.

Messages arrive on the receiving `window`, without DOM bubbling. The
`message` and `messageerror` listeners on `window.tuffite` delegate to that
window. To identify a direct guest, compare `event.source` with
`explorer.contentWindow`; reply through `event.source`. Explorer request events
such as `dialog` and `newwindow` retain their separate bubbling/capture model.

ShellAPI/dispatch events share the root EventTarget but use the fixed event
type `channel`, so business channel names do not share the Explorer tree
`message` event space:

```js
window.tuffite.addEventListener('channel', ({detail}) => {
  if (detail.channel !== 'app:changed') return;
  // detail.data is a Uint8Array.
});
```

This keeps the Web-standard `message` event for `postMessage` while allowing a
business dispatch channel literally named `message` without collision. There is no public `tuffite.channel` object; application facades such as
`window.myapp.channel` remain available to generated business APIs.

The generated typed ShellAPI remains on the application namespace such as
`window.myapp`. Its low-level binary transport is
`window.tuffite.invoke(route, Uint8Array)`, whereas Explorer communication is
`window.tuffite.postMessage(data, {targetOrigin})`; the distinct verbs and payload types
avoid a Proxy collision. `shellApi: true` can enable ShellAPI for an Explorer
only when normal origin authorization also allows it.

## Architecture and security

See [the Chromium tab audit](explorer-tab-parity.md) for native find, zoom, audio,
favicon and crash-state controls, their Playground lab and the remaining browser
capabilities that require complete services.

Blink creates an `about:blank` child-frame placeholder. A document-scoped Mojo
request reaches the browser, which validates the direct-child frame and policy,
creates an ordinary WebContents in the selected partition, and attaches it with
`PrepareForInnerWebContentsAttach()` / `AttachInnerWebContents()`. Rendering
then follows `WebContentsViewChildFrame`; GuestView is not used.

Each Explorer receives a Chromium `FrameworkIsolationInfo::kUniqueInstance`
domain. Sharing storage therefore never requires sharing the app renderer.
The composed frame ancestry uses Chromium RemoteFrame/WindowProxy objects.
Page main-frame and widget ownership remain scoped to the inner WebContents so
moving an existing page does not destroy its document or rendering surface.
Ancestor origins and inherited Permissions Policy update when the page moves.

Application asset schemes are standard, secure, CORS-enabled schemes, not local
file schemes. They retain tuple security origins for standard messaging and
Workers without granting file-origin access.

Normal origin security still applies, and an Explorer does not receive ShellAPI
unless its committed origin is granted ShellAPI commands in `app.security.capabilities`.

The initial SiteInstance carries both that stable isolation id and the final
StoragePartitionConfig. For named non-default partitions its BrowsingInstance
is fixed to that partition before the first navigation; a shared default
partition does not need the fixed flag. This keeps named persistent/in-memory
partitions intact across initial load, redirects, later `load()` calls, history
navigation, and nested Explorers.

Removing the element destroys its WebContents. Owner document replacement
invalidates late Mojo callbacks. The browser owns open interception, dialog
completion, message routing, and native context menus so nested or remote
renderers cannot bypass host policy.

The implementation and complete API reference live in
`src/tuffite/components/explorer/README.md`. The Playground Capability Lab is the
executable demonstration and smoke-test surface.

## Build isolation

With `tuffite_in_chromium=false`, the custom IDL, generated binding, browser
interfaces, and runtime feature are absent. Chromium-owned changes are guarded
by `tuffite_in_chromium` / `BUILDFLAG(TUFFITE_IN_CHROMIUM)` or narrow Tuffite
markers. Sources compiled into Chromium targets are listed through component-owned
`tuffite_in_chromium.gni` files imported directly by Chromium.

## Moving a live page

A same-document DOM move preserves the existing element and page:

```js
sidebar.moveBefore(explorer, null);
```

Use an exclusive native handle to move an existing page between elements:

```js
const handle = await source.detach();
try {
  await target.attach(handle);
} catch (error) {
  await source.attach(handle);
  throw error;
}
```

`ExplorerHandle` is a native EventTarget with readonly `state`, `statechange`,
and asynchronous `close()`. It exposes no token or serializable credential.
States are `pending`, `attaching`, `transferred`, and `closed`. An
attach reserves the handle synchronously, preventing concurrent reuse.
Success clears its internal credential and consumes ownership; repeated use
rejects with `InvalidStateError`. Closing a consumed or closed handle is harmless.
Handles are transferable, but not cloneable. A transfer list is required for
`structuredClone()`, `Window.postMessage()` and `MessagePort.postMessage()`.
Successful serialization synchronously invalidates the sender and gives the
receiver a new, exclusive native handle. Duplicate transfer-list entries,
unavailable handles, and ordinary cloning throw `DataCloneError`. Serialization
failure preserves handle ownership. BroadcastChannel does not accept handles.

`attach(handle)` accepts a connected target with or without `src` or an existing
page. A successful attach removes the `src` attribute and adopts the original
page without reloading it. A failed attach preserves the declaration and restores
the previous page. Setting `src` afterward starts a new load. There is no additional
`url` property: read the current address from `getNavigationState()`.
An element with no `src` starts empty and can receive a handle directly.

For a new application window, wait for its ready announcement, then send the
handle using the window reference's `postMessage()`:

```js
// Install the ready listener before creating the receiver window.
const readyChannel = new BroadcastChannel(`viewer:${session}`);
const ready = new Promise(resolve => {
  readyChannel.onmessage = event => {
    if (event.data?.type === 'ready') resolve();
  };
});
const win = await window.tuffite.window.create({ url: viewerUrl });
await ready;
readyChannel.close();
const handle = await source.detach();
win.postMessage({ type: 'attach-explorer', handle }, {
  targetOrigin: new URL(viewerUrl).origin,
  transfer: [handle],
});
```

```js
// viewer.html: the target has no src and starts empty.
window.addEventListener('message', async event => {
  if (event.origin !== expectedOrigin || event.data?.type !== 'attach-explorer') return;
  const handle = event.data.handle;
  try { await target.attach(handle); }
  catch (error) { await handle.close(); throw error; }
});
const readyChannel = new BroadcastChannel(`viewer:${session}`);
readyChannel.postMessage({ type: 'ready' });
readyChannel.close();
```

BroadcastChannel is used only for ready notification; it does not carry the
handle or attachment acknowledgements. For acknowledged delivery, include a
MessagePort in the transfer list and reply through `event.ports[0]` after attach,
as the Playground and Ovule implementations do. A ready timeout occurs before
detach, so the source remains attached.

`win.postMessage(message, options)` returns undefined and follows structured
serialization and `targetOrigin` semantics. The default target origin is the
sender's origin; `"*"` permits any origin. Invalid origin syntax throws before
transfer. The browser validates sender identity and application permissions,
then dispatches a trusted standard MessageEvent on the receiving `window`, with
`data`, `origin` and `ports`. These independent application windows have no
WindowProxy relationship, so `event.source` is null; use a transferred port for
replies. Origin mismatch, closed windows and absent listeners can drop messages;
successful serialization is not a delivery or attach acknowledgement. Dropped
messages release their unconsumed handle through native pipe disconnection.

Window references target the current document, like normal window messaging,
and do not retain an initial-document resource registry. WindowInfo itself is
not transferable. Its messaging method is bound to the original window identity;
editing the snapshot's id cannot retarget the method. `win.transfer()`, named
`explorer:` sources, and the resource-waiting `defer` attribute are removed.

Attach still requires compatible resolved policy and storage partition. History,
script state, form input, document ID, renderer isolation and storage remain
intact. Transferred handles survive sender navigation. An unreachable receiver
handle releases its page when its control pipe is garbage-collected; close
abandoned handles explicitly for deterministic cleanup. Detached pages are
hidden and cannot open windows or host dialogs.

The source emits `detached`, and the receiver emits `attached` after ownership
commits. Moves themselves do not reload or emit navigation events. Attach failure
uses `InvalidStateError` for unavailable handles, `SecurityError` for incompatible
hosts, and `AbortError` for interrupted preparation.

The Playground `transfer.html` lab demonstrates direct attachment and window
messaging. Its native smoke covers exclusive ownership, declaration removal,
replacement rollback, structuredClone and MessagePort transfer, clone rejection,
standard MessageEvent delivery, and sender navigation after handoff:

```sh
python3 playground/scripts/smoke.py --explorer-transfer
```

## WebAuthn and passkeys

Cross-origin Explorer pages inherit their embedding document's Permissions Policy,
like iframes. Delegate supported web features with the standard `allow` attribute:

```html
<explorer src="https://example.com/"
  allow="publickey-credentials-get *; publickey-credentials-create *; fullscreen *">
</explorer>
```

An `allow` declaration cannot override a parent or response header's denial.
It grants web feature delegation, independently of native ShellAPI capabilities.
Cookie SameSite rules also reflect the embedding ancestry: cross-site Strict
cookies are blocked; cookies intended for cross-site use need appropriate
SameSite and Secure attributes. Storage partition selection remains explicit.


Explorer uses Chromium's WebAuthn origin and relying-party validation. Requests
are handled by the Framework's application-scoped WebAuthenticationService, without
ShellAPI grants. The native chooser identifies the relying-party domain and
provides cancellation, account confirmation and security-key PIN entry. Request
delegates and dialogs retain their per-request lifetime. Downloads use a separate
BrowserContext-scoped DownloadService with managed observer teardown. PINs
stay in browser UI and are never returned to application JavaScript.

Windows delegates to the operating system's WebAuthn UI, including Windows Hello,
security keys and phone passkeys where the installed Windows version supports
them. Other desktop systems offer USB security keys and nearby phone QR sign-in
when Bluetooth is available. Conditional assertion requests use an inactive,
nonmodal chooser; there is no Chrome password-manager autofill or cloud passkey
store. Conditional passkey creation upgrades and security-key biometric
enrollment are not supported and cancel instead of remaining pending. Immediate
mediation reports no match because this embedder has no password-manager
credential store.

On macOS, iCloud passkeys require Apple's approved
[`com.apple.developer.web-browser.public-key-credential` entitlement](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.web-browser.public-key-credential)
and a properly provisioned application signature. Ad-hoc `tuff package` signatures
do not grant this entitlement. The Framework checks actual OS support and omits
the iCloud option when unavailable; it does not forge an entitlement or store a
second copy of the user's passkeys.

Phone sign-in on macOS requires a packaged `.app` with
`NSBluetoothAlwaysUsageDescription` and launch from Finder so the responsible
process has the required Bluetooth metadata. `tuff package --format app` emits
this description. Bare `tuff dev` executables continue to support USB keys but
omit phone sign-in to avoid macOS terminating a process without privacy metadata.
Bluetooth access still requires the user's system permission.

Run the Explorer WebAuthn smoke against a packaged Playground:

```sh
python3 playground/scripts/smoke.py --explorer-webauthn --package dist/playground.app
```

This uses a loopback relying party and DevTools virtual authenticator to verify
resident credential creation/assertion, user verification and RP isolation. It
also cancels repeated modal and conditional requests to verify UI teardown and
request reuse. It does
not prove physical USB, Bluetooth, Windows Hello or iCloud authentication; those
require platform devices and, on macOS, appropriate signing credentials.

### Login challenge diagnostics

Explorer supports Async Clipboard standard-format writes, including
`navigator.clipboard.writeText()`, from a user-activated, focused page in a
secure context. Clipboard reads and writes without fresh user activation are
denied. A Copy button can use the standard API without a ShellAPI capability;
no persistent clipboard grant is stored.

A security challenge after an identity-provider redirect is separate from a
WebAuthn request. Native authenticator support does not establish that a
third-party security challenge will complete. Keep the same Explorer and
partition throughout the login flow; recreating it can discard session state.

When a challenge repeats, enable **Preserve log** in the Explorer's DevTools
Network panel and inspect failed challenge requests and Console errors. Record
the hostname, HTTP status or browser error, and whether the same flow works in a
standard browser on the same network. Do not share callback URLs, authorization
codes, Cookie headers or an unredacted HAR. Cloudflare's
[challenge troubleshooting](https://developers.cloudflare.com/cloudflare-challenges/troubleshooting/challenge-solve-issues/)
describes storage, blocked requests and browser compatibility causes.
An HTTP 401 from a challenge-platform `/pat/` request is an expected unsuccessful
Private Access Token probe; Cloudflare falls back to a standard challenge. It
does not by itself identify the cause of a loop. See
[Private Access Tokens](https://developers.cloudflare.com/cloudflare-challenges/reference/private-access-tokens/).
DNS lookup failures on subdomains of `challenges.cloudflare.com` or `dnstest.dev`
can also be expected, non-blocking probes. Do not infer a broken resolver from
such a request alone or treat it as a fatal Explorer navigation error. This
does not apply to a failure loading the main `challenges.cloudflare.com` host.
Compare the official [compatibility checking tool](https://debug.challenges.cloudflare.com/)
in Explorer and a working standard browser to identify environment differences.

The default partition and `persist:` partitions store their HTTP cache and
Cookie database in separate directories below the application profile. Cookie
values use Chromium OSCrypt with macOS Keychain, Windows DPAPI, or the Linux
Secret portal when D-Bus support is built in. No plaintext or known-key fallback
is configured; without a provider the Cookie store remains in memory, and if a
provider cannot obtain its OS key, encrypted writes fail rather than storing
plaintext. Session cookies are not restored after restart. Temporary partitions
keep both cookies and HTTP cache in memory. This restart behavior alone does not
explain a challenge repeating within one unchanged live Explorer.

## Browser compatibility diagnostics

Explorer uses Chromium's real User-Agent and User-Agent Client Hints metadata.
Navigation headers include the default low entropy hints. Additional hints
require the destination's `Accept-CH` opt-in and remain scoped to that origin
and the current BrowserContext session. A website's verification service may
still reject an embedded browser; these interfaces do not guarantee acceptance.

Cloudflare's diagnostic `resultsStorageStatus` describes whether its server
saved the diagnostic report, not the availability of browser storage. A
[`600*` Turnstile error](https://developers.cloudflare.com/turnstile/troubleshooting/client-side-errors/error-codes/) is a generic challenge failure and does not identify a
missing browser API. Keep this distinct from expected Private Access Token
401 responses or deliberate DNS probes.

For a live failure, attach DevTools before navigation and follow newly created
iframe targets after a widget reset. Record response status/protocol, failed
request reasons, blocked-cookie reasons, and focus/visibility during a manual
verification. Exclude cookies, request bodies, OAuth parameters and challenge
tokens from shared logs. A cached Console warning alone does not establish
which request failed during the current attempt.

Compare an Explorer with an ordinary top-level WebContents using the same
Framework and separate temporary profiles. If both fail, investigate shared
browser/network behavior before changing Explorer isolation. The diagnostic
widget can place its iframe in a closed shadow root; an empty `window.frames`
list is not by itself evidence of a missing iframe. Successful local input,
storage or WebAuthn tests do not establish acceptance by a verification service.

### Embedder contract audit

The live macOS comparison on October 11, 2026 reproduced `600010` in both
Explorer and an ordinary top-level WebContents with temporary profiles. The
top-level verification frame received focus, visibility and transient user
activation. Captured verification GET/POST requests returned HTTP/2 200, with
no recorded blocked-cookie reasons. The diagnostic page's synthetic `HEAD`
connectivity probe returned 404; this was not a failed verification resource.

The successful Codex in-app browser supplied both `Chromium` and `Google Chrome`
brands in `Sec-CH-UA`. Tuffite supplied `Chromium` and the generated GREASE
brand, consistent with its unbranded Chromium build. Its JavaScript high entropy
version was `154.0.8019.0` and its reduced UA version was `154.0.0.0`. The brand
difference is established; it does not prove why the verification was rejected.
Do not add another browser's brand to disguise the build.

| Contract | Current behavior | Assessment |
| --- | --- | --- |
| UA and Client Hints | Chromium metadata and a per-context Client Hints delegate | Implemented; real HTTP and JavaScript version/platform agree. |
| Network languages | Shared `en-US,en` preference for network, documents and workers | The application currently uses an English locale; network headers include Chromium-generated quality weights. |
| Network compression | Brotli and zstd enabled | Aligned with Chromium content_shell; not a proven challenge cause. |
| DevTools product | `Tuffite/<Chromium version>` in `/json/version` | Truthful debugging metadata; does not change the browser's web-visible brand. |
| Cookie and HTTP cache persistence | NetworkContextService supplies per-partition paths and OS encryption | Persistent and temporary partitions retain separate storage semantics; no plaintext fallback. |
| DOM storage and worker APIs | Chromium storage/worker implementation; profile path supplied | A null special storage policy does not disable these APIs. Live interface presence alone does not verify every operation. |
| Permissions | Sanitized clipboard writes can prompt; other permissions denied | Explicit policy limitation. A verification failure does not justify broadly granting permissions. |
| JavaScript dialogs | Explorer retains its event-based policy; Window uses JavaScriptDialogService | Top-level alert, confirm, prompt and before-unload have native modal handlers with navigation/shutdown cancellation. |
| Media capture, notifications, push and background sync | Unsupported/default rejection or null service | Missing optional browser capabilities; no captured request establishes their involvement in this failure. |
| SSL host state and Guest manager | Default SSL exception strategy; ordinary non-Guest WebContents | Null delegates here are permitted contracts, not missing core page implementation. |
| WebGPU | Hardware adapter requests succeed; a forced fallback adapter request returns null | The verification frame's adapter warning alone does not establish missing WebGPU support; no evidence supports overriding Chromium's GPU restrictions. |

The remaining rejection reason is not exposed by the generic client error.
Cloudflare describes custom engines and embedded browsers as having
[limited support](https://developers.cloudflare.com/cloudflare-challenges/reference/supported-browsers/).
Treat this as a compatibility investigation rather than a guarantee that adding
optional embedder delegates will make third-party verification succeed.

The browser contract regression uses only local fixtures:

```sh
python3 playground/scripts/smoke.py --browser-contract --package dist/playground.app
```

It checks document/worker/network languages, the DevTools product, native
alert/confirm/prompt and before-unload handlers, Cookie and HTTP cache recovery
after normal process shutdown, and temporary/session Cookie isolation. The
runner also reads the resulting Cookie databases and rejects plaintext values.
The macOS release validation passed after the October 11 contract changes;
Windows and Linux runtime validation must run on their respective hosts. A
fresh top-level Cloudflare diagnostic still reported `600010` after the changes
and manual verification. Its real HTTPS requests advertised `gzip, deflate, br,
zstd` and `en-US,en;q=0.9`; captured verification responses used HTTP/2 with
status 200 and no Cookie rejection records. The diagnostic HEAD probe still
returned 404.
These implementation fixes are not a confirmed solution for that rejection.

A subsequent clean control used a copied Playground bundle, a new profile,
one ordinary top-level window and no remote debugging port, DevTools or CDP
connection. Its local application entry navigated normally to the diagnostic
site. Manual verification still returned `600010` after all four public
diagnostics passed. Debugger attachment is therefore not necessary to reproduce
the failure, and profile replacement alone did not resolve it.

The successful Codex in-app browser report and the failed Tuffite reports had
the same reduced UA, reported CPU count and memory, screen characteristics and
Intel UHD 630 ANGLE Metal WebGL renderer. This rules out attributing the failure
to a different reported WebGL renderer; it does not establish that WebGPU
adapter availability or other browser capabilities match.

A separate inspected process reported hardware GPU compositing, WebGL and
WebGPU enabled. Ordinary and high-performance WebGPU requests returned an AMD
RDNA 1 adapter, and a low-power request returned an Intel Gen 9 adapter.
`forceFallbackAdapter: true` returned null and produced the same "No available
adapters" warning seen during verification. This establishes that hardware
adapter enumeration works; the captured verification warning does not reveal
which options its caller used.

A `content_shell` reference from the same checkout and build graph requires
about 3,900 additional build tasks in the current macOS release output. This
holds the engine, downstream Chromium patches and build settings constant while
changing the embedder. It is not a pristine upstream comparison:
`content_shell` also lacks Chrome profile services and reports a test-specific
`content_shell` Client Hints brand with an `Unknown` platform. A negative
result alone cannot isolate Tuffite or prove a UA problem. Record the result
before adding optional delegates or changing browser identity.

The reference build was stopped after its plan revealed roughly 3,000 C++
compilations; no runtime result was obtained from it. Completed objects remain
reusable. Do not report this reference as a passed or failed verification test.

### Codex browser comparison

Browser Use confirmed that the existing Codex in-app diagnostic page displayed
successful verification. The same local loopback diagnostic fixture then ran
in that browser and a temporary Tuffite Playground process. It read standard
browser APIs and queried permission states without requesting permissions or
interacting with the verification widget.

| Observation | Codex in-app browser | Tuffite Playground |
| --- | --- | --- |
| Full Chromium version from Client Hints | `154.0.8037.98` | `154.0.8019.0` |
| Reduced UA version | `154.0.0.0` | `154.0.0.0` |
| Client Hints brands | Chromium, Google Chrome, GREASE | Chromium, GREASE |
| Architecture / bitness / platform version | x86 / 64 / 26.6.2 | x86 / 64 / 26.6.2 |
| `navigator.webdriver` | false | false |
| JavaScript languages | en-US | en-US, en |
| Request Accept-Language | en-US,en;q=0.9 | en-US,en;q=0.9 |
| Request Accept-Encoding | gzip, deflate, br, zstd | gzip, deflate, br, zstd |
| WebGL renderer | Intel UHD 630 ANGLE Metal | Same |
| Default / high-performance WebGPU adapter | AMD RDNA 1 | Same |
| Low-power WebGPU adapter | Intel Gen 9 | Same |
| Forced fallback WebGPU adapter | null | null |
| Clipboard read / write permission | prompt / granted | denied / prompt |
| Notification, geolocation, camera, microphone permission | prompt | denied |

Both browsers exposed service workers, IndexedDB, CacheStorage, SubtleCrypto,
SharedWorker, WebGPU, PublicKeyCredential and Notification interfaces. Interface
presence does not establish support for every operation. Permission results are
specific to the fresh local diagnostic origin, not the verification site's
stored permissions.

The matching reduced UA had hidden an actual engine-version difference. These
are not the same Chromium build, so the successful browser cannot yet serve as
a control for embedder behavior alone. GPU adapter enumeration and the tested
request language/compression headers matched. Neither the version/brand
difference nor the explicit permission policy is a demonstrated cause of the
challenge rejection. Keep policy restrictions intact and compare actual engine
revisions before claiming that another missing delegate explains the failure.


### Permission delegate trace

A fresh, ordinary top-level Playground window using the October 11 tab
integration build recorded document permission queries while the public
Cloudflare diagnostic page ran. The delegate observed VideoCapture,
AudioCapture, Geolocation, Notifications, BackgroundSync, CameraPanTiltZoom and
WebAppInstallation queries; all returned denied under the current capability
policy. The trace contains entry point, permission type and result only, without
origins, request URLs or authentication data. It does not identify the calling
script or distinguish the diagnostic page from its embedded challenge.

These observations establish that permission policy differences are exercised
on the real diagnostic page, rather than only on the local comparison fixture.
They do not establish that any one query caused verification failure. Native
notifications also have a separate denied fast path when no platform service is
installed. A supported capability must implement its backend, consent and
revocation semantics together before exposing a different permission state.

The October 11 tab integration build passed the native browser contract suite
(language, JavaScript dialogs, encrypted cookie restart, partition isolation
and HTTP cache restart), plus Explorer input, Client Hints, clipboard and
WebAuthn regressions. Native file chooser selection and cancellation still
require platform UI validation; successful compilation does not establish
those behaviors. Windows and Linux validation of that suite remained pending.

The manual retry on the permission-trace build returned `600010` after
interaction; its four basic diagnostics passed on both attempts. This did not
validate a permission-policy fix.

### Playground compatibility regression

The Explorer lab links to `compatibility.html`. Its local report executes
Canvas and WebGL pixel readback, SHA-256, IndexedDB write/read, localStorage,
CacheStorage, Client Hints, Worker metadata, native legacy navigation timing
and permission queries. Permission
queries do not request or grant access. Test storage uses unique fixture names
and is removed after each probe; the report does not read existing cookies or
authentication data.

Run the standard-origin comparison in an isolated packaged Playground:

```sh
python3 playground/scripts/smoke.py --browser-compatibility --package PATH
```

The runner serves a loopback HTTP fixture and compares an Explorer with a
separate top-level window using the same Framework. It verifies actual Worker
execution and HTTP navigation headers as well as the local API operations.
Application-scheme Workers also use the native asset loader and normal tuple
origins. The runner checks both the application and HTTP(S) contexts.
CacheStorage entries use synthetic HTTPS request keys without issuing network
requests, because CacheStorage rejects custom-scheme request keys.

For separate real-service acceptance, add `--turnstile-live`. The test requires
the diagnostic page's actual `Turnstile Status: success`; local checks never
substitute for it. The runner does not click the verification widget and fails
if the result is still loading or unsuccessful after 90 seconds. It keeps the
challenge visible when the troubleshooter automatically scrolls to a warning.
It reads only
the public status and issue title, excluding diagnostic session IDs, IPs and
challenge tokens.

Add `--turnstile-test-keys` to check the official always-pass and always-fail
test-key callbacks in an HTTP Explorer fixture. These are client-side
integration checks only, without Siteverify or a real production challenge.
See Cloudflare's [testing documentation](https://developers.cloudflare.com/turnstile/troubleshooting/testing/).

The Framework installs `chrome.loadTimes()` and `chrome.csi()` in document
contexts through `framework/renderer`. The implementation
adapts Chromium's renderer bindings and returns the current frame's actual Blink
navigation timings and negotiated transport information. The local regression
checks consistency with `performance.timeOrigin` and elapsed wall time.

On October 14, 2026, the Windows release candidate using Chromium
`154.0.8019.0` passed the standard-origin comparison, both official test-key
callbacks and the UI production build. A persistent Explorer reached real
Turnstile success without clicking the widget. A second isolated-profile run
also reached real success within the 90-second acceptance test while keeping
the widget visible. Both retained the Microsoft Basic Render Driver privacy
warning; the warning did not prevent successful verification.

The earlier build had returned `600010` after an authorized manual click.
The candidate adds the missing native timing APIs, but the repeat test also
corrects viewport handling: the first automated candidate run remained loading
after the troubleshooter scrolled to its warning. These observations validate
the candidate's actual Windows acceptance, without proving that timing API
presence alone explains every rejection. macOS acceptance of this candidate
has not been tested.

### Composed ancestry validation (October 15, 2026)

The Windows release candidate passed native parent/top, trusted MessageEvent
source/reply, nested iframe/Explorer messaging and detach/attach ancestry tests.
It also passed strong-exclusive handle transfer through structured clone,
MessagePort and independent-window postMessage, event capture/bubbling and
one-shot request handling, and Ovule's production independent-window promotion
without reloading the moved document.

The final packaged Playground passed the complete Explorer demo and resource
regressions, including fullscreen with explicit `allow` delegation, downloads,
video PiP and repeated document PiP. WebAuthn tests covered delegated and denied
Permissions Policy, dynamic allow changes, response-header restrictions and
native request cancellation. The compatibility comparison and both official
Turnstile test keys passed. The final package's real Cloudflare diagnostic page
returned `Turnstile Status: success` without a verification click; it retained
the graphics privacy warning. This result validates this Windows candidate;
macOS has not been revalidated with the composed ancestry implementation.
