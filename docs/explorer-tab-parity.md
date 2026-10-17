# Explorer and Chromium tab audit

This audit compares the pinned Chromium checkout's production tab contracts with
Tuffite's ordinary inner WebContents, its delegates and the Framework services.
It separates engine functionality from browser/profile integrations. A native
tab's toolbar, permission UI and extensions are not installed by WebContents.

## Suitable additions implemented

| Area | Previous gap | Implementation and validation |
| --- | --- | --- |
| Find in page | No host search API or native result bridge | `findInPage()` calls Chromium's frame-tree search; `findresult` supplies request IDs, match counts, active ordinal, selection rect and completion. Test includes a child iframe, case sensitivity, next/previous, missing text and stop. |
| Page zoom | No independently controlled Explorer zoom | `setZoomFactor()` / `getZoomFactor()` use native temporary HostZoomMap levels, from 25% to 500%. State belongs to the WebContents, survives cross-site navigation and transfer, and leaves siblings and the app shell unchanged. Ctrl+wheel follows the same path. |
| Audio controls | No host mute API or audio state events | `setAudioMuted()` changes native WebContents output, not media DOM properties. `audiostatechange` reports native audible/mute changes. State survives navigation and transfer. |
| Favicon metadata | No candidate notification | The primary frame's actual favicon candidates, sizes and types are exposed in state and `faviconchange`; no extra icon fetch, permission grant or disk history is introduced. |
| Renderer exit | Host lacked an actionable crash event | `renderprocessgone` exposes native termination status and the actual `crashed` state. Existing `reload()` recovers the renderer; sibling isolation and retained zoom/mute are tested. |
| Network errors | Renderer delegate supplied no error document | A local error document presents Chromium's actual network error code. It contains no failed URL, external resources or TLS exception action. Load events retain Chromium error-navigation semantics. |
| Window activation | Explorer and Window inherited a no-op delegate | Activation reaches the owning native Widget and focuses the requesting contents. Detached contents cannot activate a window. |
| Keyboard fallback | Unhandled Explorer keys did not reach native Views | Keys unhandled by the page reach the owner's `UnhandledKeyboardEventHandler` and FocusManager. Application-specific browser accelerators remain application UI decisions. |
| Native window bounds | Delegate had no bounds for Document PiP window queries | Explorer forwards to the owning Widget's OS window bounds, including nested owners. Web-visible screen geometry remains Chromium's own renderer-host implementation. |
| Unsupported keyboard lock | Content's default accepts top-level and detached requests without a Tuffite consent service | Window, Document PiP and detached transfer delegates now explicitly reject; embedded Explorers retain Content's inner-contents rejection. |
| Legacy navigation timing | Missing Chromium renderer extension | `chrome.loadTimes()` / `chrome.csi()` return actual frame timing/transport data. Covered by the separate compatibility lab added during the Turnstile investigation. |

## Existing contracts reviewed

| Area | Current behavior |
| --- | --- |
| Navigation, redirects and history | Native NavigationController with policy-controlled open events, main-document identity and stale-script rejection. Same-document navigation retains identity. |
| Isolation and storage | Normal, non-Guest WebContents with a unique renderer isolation domain and fixed named storage partition. Persistent cookies use OS encryption; memory partitions remain separate. |
| Worker/DOM storage/network APIs on HTTP(S) | Native Blink/Content implementations; operational fixtures exercise module Worker, IndexedDB, localStorage, CacheStorage, SHA-256 and navigation headers. |
| Input and composition | Native child-frame compositor/input route, not screenshot-driven rendering. Existing Explorer suite covers nested input and focus. |
| JavaScript dialogs | Immutable emit/suppress policy with exactly-once response and navigation/destruction cancellation. Top-level windows use native dialog UI. |
| Downloads, PDF resources and previews | Native download manager with ownership checks, progress, cancellation and bounded reads; PDFs download because a PDF viewer is not installed. Previews capture the native compositor surface. |
| Native file inputs | Shared file chooser service handles select/multiple/folder upload/filtering and cancellation. Platform UI selection needs platform testing. |
| Desktop color inputs | Blink's desktop color popup path is distinct from the Android/iOS `OpenColorChooser` delegate. The absence of that mobile delegate is not a desktop gap. Native EyeDropper selection is a separate unsupported delegate. |
| Fullscreen and PiP | Shared native Window delegates, video overlay and Document PiP. Native behavior must be tested alongside viewport/owner transitions. |
| Context menus and editing | Shared context menu session implements history, editing, image/link actions, PiP and inspect, subject to Explorer policy. |
| WebAuthn | Shared security checks and request lifecycle; platform support and allowed transports remain explicit. |
| UA, Client Hints, language and compression | Real unbranded Chromium metadata; document/worker/network consistency and compression coverage already have regressions. |
| Graphics and media playback | Native Blink, ANGLE and Chromium media pipelines. Software GPU reports, codec availability and hardware restrictions remain truthful. |

## Browser capabilities requiring a separate complete service

The following are identified gaps, but are not suitable as isolated delegate
stubs or automatic grants. They need the listed implementation to provide a
usable, consistent capability. None is advertised as supported by this audit.

| Capability | Required work |
| --- | --- |
| HTTP/proxy authentication and client certificates | Origin/proxy-labelled credential UI, request cancellation, concurrent prompt ownership and certificate store selection. |
| Site permission persistence and revocation | Capability policy, origin/embedding-aware decisions, consent UI, storage and permission-change notifications. |
| Camera/microphone/display capture and audio-output selection | Native chooser, OS permission handling, device selection, active-capture indicators and teardown. |
| Notifications, push and background sync/fetch | Notification service and OS integration, service-worker delivery, profile persistence and lifecycle management. |
| USB/HID/Serial/Bluetooth, geolocation and sensors | Device-specific services and chooser UI, origin grants and revocation. |
| Writable File System Access | Origin grants, sensitive-path checks, write consent and handle restoration. Picker-authorized reads and OPFS are distinct existing contracts. |
| Pointer/keyboard lock | Explicit user consent and Escape/fullscreen exit behavior. Pointer lock currently rejects; inner keyboard lock is disabled. |
| EyeDropper | Native screen sampling and chooser UI, with correct cancellation and origin/user-activation checks. |
| Print/print preview and PDF viewing | Printing renderer/browser integration, backend and dialog lifecycle; a real PDF renderer/viewer with accurate supported-MIME reporting. |
| Native popup adoption | A browser-surface owner preserving opener, browsing-instance, partition and session storage, with popup policy. Current open events remain app-controlled. |
| Back/forward cache and prerender | Embedder lifecycle opt-in plus document-service, transfer and owner teardown tests. Content defaults currently disable these for this delegate. |
| App custom-scheme Workers | Separate origin/loader/CSP design. The compatibility lab reports the custom-scheme module Worker limitation separately from working HTTP(S) Workers. |
| Password management, autofill, spellcheck, translation, extensions, installed web apps and origin trials | Chrome/profile integrations with their own settings and lifecycle; no fake interfaces, Chrome branding or test trial keys. |
| Chrome-style download/history/site settings UI | Application/profile products over the native services, including site cookie controls and transport-policy persistence. |

## API and regression

```js
explorer.addEventListener('findresult', ({detail}) => {
  if (detail.finalUpdate) console.log(detail.matches, detail.activeMatchOrdinal);
});
await explorer.findInPage('needle', {matchCase: false});
await explorer.findInPage('needle', {findNext: true, forward: false});
await explorer.stopFindInPage('clear');
await explorer.setZoomFactor(1.25);
await explorer.setAudioMuted(true);
const {zoomFactor, audioMuted, audible, crashed, favicons} = await explorer.getNavigationState();
```

Empty or oversized find text rejects. Unknown stop actions and factors outside
the zoom range return false; WebIDL rejects non-finite factors. Detached/unattached
hosts cannot accept these controls. Native find updates can arrive before the
caller processes its accepted-ID promise; install the event listener first.
Stop, navigation, detach and renderer exit invalidate outstanding find results.

The Playground entry is `tabs.html`, with two independent Explorer fixtures and
interactive find, zoom, mute, reload and state controls. Run its isolated smoke:

```sh
python3 playground/scripts/smoke.py --explorer-tabs --package PATH
```

This regression intentionally crashes only its own isolated fixture renderer.
It creates fresh test state; it does not modify the user's active application
profile.

On October 14, 2026, the Windows x64 release candidate compiled and its
Distribution verified all 23 runtime files. The new tab regression passed,
including an actual isolated renderer crash and recovery. The existing Explorer
suite, 25-transfer stress test and resources suite also passed: navigation,
dialogs, partitions, nested events/input, native context menus, download
ownership/progress/cancellation, PDF byte recovery, video and Document PiP.
The UI production build, strict public TypeScript declarations and native
format checks passed. Resource-path assertions normalize Windows separators.
The separate browser compatibility suite and official test-key callbacks also
passed on the final candidate. Its real Turnstile acceptance check reached
`success` without clicking the widget; the diagnostic GPU warning remained.

macOS and Linux runtime tests were not run for this candidate. Native file
chooser selection/cancellation, OS window activation and system keyboard
fallback were not independently exercised by platform UI automation; compilation
and the existing focused-input/context-menu tests do not establish every native
platform interaction. No permission-based service in the remaining-capabilities
table is claimed as complete.
