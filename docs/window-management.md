# Window management

Tuffite exposes desktop windows through the generated `window.tuffite.window`
facade. Window calls use the same binary ShellAPI path as every other builtin:

```text
TypeScript contract
  -> generated binary codec and Proxy
  -> window.tuffite.invoke("tuffite.window.*", Uint8Array)
  -> document-scoped Mojo DispatchHost
  -> browser-process window::Manager
  -> Chromium Views Widget or macOS NSWindow adapter
```

There is no second IPC protocol and no native window handle is exposed to the
renderer. `WindowId` is an opaque signed 64-bit value represented as a
JavaScript `bigint`. IDs are process-local and must not be persisted across app
runs.

## API

The framework contract in `src/tuffite/framework/api/window.d.ts` generates these calls:

```ts
const current = await window.tuffite.window.current();
const all = await window.tuffite.window.list();

const child = await window.tuffite.window.create({
  url: 'playground://main/window.html',
  title: 'Project',
  bounds: {x: 120, y: 120, width: 900, height: 640},
  focused: true,
});

await window.tuffite.window.focus(child.id);
await window.tuffite.window.minimize(child.id);
await window.tuffite.window.maximize(child.id);
await window.tuffite.window.restore(child.id);
await window.tuffite.window.setFullscreen(true, child.id);
await window.tuffite.window.setTitle('Project A', child.id);
await window.tuffite.window.setBounds(
  {x: 160, y: 140, width: 1024, height: 720},
  child.id,
);
await window.tuffite.window.close(child.id);
```

An omitted ID always means the caller's outermost application window. This is
also true when the caller is an authorized `<explorer>` document: its current
window is the outer application window, not an inner WebContents surface.

`WindowInfo` contains the opaque ID, native title, last committed URL, screen
bounds, focus state, minimized state, maximized state, and fullscreen state.
State-changing calls return a fresh `WindowInfo`; `close` returns only after the
browser has accepted the close request. Native close completion may occur on a
later UI task.

## Manifest policy

Window methods are granted by `app.security.capabilities[].shellApi` for the
caller's committed origin. Target methods default to the current window; use
`allow: [{ "window": "others" }]` or `allow: [{ "window": "*" }]` to extend
that scope. Explicit deny scopes override matching allows. Creating a window
never grants its destination native APIs.
Size settings belong to each `app.windows[]` entry, at the same level as width
and height. Different startup windows can have different native constraints.
No configurable maximum window count is exposed.

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "url": "demo://main/index.html",
        "width": 1000,
        "height": 700,
        "minWidth": 640,
        "minHeight": 480,
        "maxWidth": 1920,
        "maxHeight": 1200
      }
    ],
    "security": {
      "capabilities": [
        {
          "identifier": "app-shell",
          "origins": [
            "demo://main"
          ],
          "shellApi": [
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
            "tuffite.window.setBounds"
          ]
        }
      ]
    }
  }
}
```

Each startup-window declaration also accepts `decorations`, which defaults
to `true`. Setting it to `false` asks Tuffite to keep the normal resizable
window and content lifecycle while letting the application render its own title
bar:

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "url": "demo://main/index.html",
        "width": 1200,
        "height": 800,
        "decorations": false
      }
    ]
  }
}
```

For undecorated windows, CSS `app-region: drag` marks draggable renderer
regions and `app-region: no-drag` carves controls back out. Tuffite enables
Chromium draggable-region reporting only for these windows and maps the reported
region to native non-client hit testing. Applications invoke the existing Window
capability for minimize, maximize/restore, and close; the renderer never receives
a native handle. On Windows and Linux Tuffite removes the standard frame. On
Windows it uses an opaque DWM top-level surface plus a narrow native edge strip
so the renderer child HWND cannot intercept native resize input; application
CSS still owns every visible color. Linux and macOS retain the translucent
composition path. On
macOS it retains a titled, closable, minimizable, resizable `NSWindow`, extends
content under a transparent title bar, and hides the native traffic-light
controls so the web title bar can place equivalent controls on the left.

- `tuffite.window.create` permits creating another top-level window.
- `tuffite.window.list` permits enumerating application windows.
- Target operations check their method grant and window scope independently.
- Each native window retains its size limits for resizing and `setBounds`.

The URL and ShellAPI-origin policies are intentionally independent: an app may
open an untrusted documentation window without giving that document privileged
APIs. Credentials in window URLs are rejected.

## Ownership and lifecycle

The manager resolves the caller from its committed `RenderFrameHost`, walks to
the outermost WebContents, and maps that WebContents to Tuffite's own `Window`.
This prevents a renderer from claiming another native window as its current
window. The registry reconciles itself with `Window::windows()` on each call,
so closed native windows cannot be addressed through stale IDs.

Windows use the same BrowserContext as the caller and therefore share the
application storage partition, cookies, cache, and service workers. Desktop
operations use Chromium's Views `Widget` abstraction on Windows and Linux and
an NSWindow adapter on macOS. Platform-specific code stays under
`src/tuffite/framework/window/`.

The top-level framework, rather than `views::WebView`, owns the Content
visibility lifecycle. Tuffite attaches the `WebContents` after native Widget
initialization, performs the first visible layout, and calls `WasShown()` only
after the Widget is visible. A `WidgetObserver` maps hide/minimize/restore back
to `WasHidden()` / `WasShown()`, and teardown always hides before detaching.
This is required even though a newly created `WebContents` initially reports
`Visibility::VISIBLE`: Chromium does not consider it visible for real, nor
guarantee a first compositor frame, until the host calls `WasShown()`.

The explicit test-only switch
`--tuffite-window-surface-test=<marker.json>` verifies that contract without
opening a DevTools endpoint. It waits for the main `RenderWidgetHostView`,
captures the final compositor output through `CopyFromSurface`, and writes
Widget/View visibility, bounds, viewport, bitmap size, opacity, non-black pixel,
and sampled-color evidence. Packaged native smoke combines this marker with the
actual desktop window capture, accessibility tree, and application database.

## Events and close requests

The existing `window.tuffite` EventTarget-compatible channel is the future
transport for `tuffite:window-created`, `tuffite:window-state-changed`, and
`tuffite:window-closed` notifications. These events are notifications, not
authorization decisions. A cancellable native close request requires a
browser-owned deadline, deterministic default action, and renderer-crash
handling; it must not be implemented as an unbounded asynchronous JS event.
That close-veto protocol remains a separate production-hardening item.
