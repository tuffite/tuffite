# Tuffite production readiness

## Implemented baseline

The current Windows vertical slice is suitable for framework development and
integration testing. It now includes:

- a versioned `tuffite.json` loaded and validated before browser-process
  application startup;
- an application-named native executable copied from `tuffite_launcher` that enters a
  version-matched Framework library and supplies a versioned callback table;
- manifest-authorized, application-registered Rust utility services launched
  through Chromium `ServiceProcessHost` and a generic binary Mojo interface;
- a generic, bounded manifest-scheme asset loader with traversal rejection, MIME
  detection, no-sniff, and a restrictive default CSP;
- manifest allowlisted privileged HTTP(S) through Chromium Network Service,
  with credentials omitted, redirects not auto-followed, a 1 MiB default body
  limit, and a 30 second default timeout;
- startup-validated wildcard policies for ShellAPI, Explorer, and Window
  origins; Explorer and Network URLs; and logical filesystem paths. Matching
  reuses Chromium `base::MatchPattern`, while opaque origins, traversal,
  resource writes, and implicit private-network wildcard access remain denied;
- a TypeScript-first ShellAPI contract as the source for generated Rust
  types/dispatch, strict binary codecs, identity-stable JavaScript Proxy
  bindings, `.d.ts`, and internal deterministic command IDs;
- browser-owned multi-window management with opaque IDs, manifest-gated
  create/list/cross-window control, shared BrowserContext storage, and native
  Views/NSWindow adapters;
- an explicit top-level `WebContents` visibility lifecycle tied to the native
  Widget, including first `WasShown()`, hide/minimize `WasHidden()`, restore,
  symmetric teardown, and a compositor-surface smoke marker. This prevents a
  packaged window from having a live DOM but a permanently black client area;
- supervised reusable utility processes with bounded queues, 30-second call
  timeout and idle shutdown, exponential restart backoff, circuit breaking, and
  no implicit replay of ambiguous in-flight work;
- a `python3 x.py` workflow with outer mise tool provisioning for `generate`, `check`, `dev`, `build`,
  `package`, and `smoke`;
- binary protocol errors that distinguish malformed requests, unknown commands,
  and application handler failures;
- a smoke test proving Runtime dispatch, sandboxed utility invocation, an
  allowed Network Service request, denial of an undeclared network host, denial
  of ShellAPI inside a same-origin Explorer, a complete child-window lifecycle,
  and Explorer pixels in the final outer composition.
- a packaged native-window smoke that verifies window creation, SQLite
  initialization, accessibility content, desktop pixels, and non-empty main
  compositor pixels without enabling remote debugging.

## Required before a public production release

These items deliberately remain open and must not be inferred from the working
demo:

1. Complete the existing request-ID based, off-UI-thread dispatch scheduler.
   It already has a bounded queue and soft queue deadlines; concurrent handler
   execution, running-handler cancellation, hard deadlines, and document-generation
   revocation remain open.
2. Extend the dedicated ShellAPI Mojo transport with explicit request IDs,
   cancellation/deadline propagation, bounded concurrent in-flight requests,
   attachment/data-pipe streaming, and document-generation revocation.
3. Add DNS resolution-aware SSRF enforcement for localhost, private, link-local,
   rebinding, redirects, proxies, and each resolved endpoint. The current host
   allowlist is necessary but not sufficient against DNS rebinding.
4. Add streaming uploads/downloads with backpressure and explicit credential,
   header, method, redirect, certificate, proxy, and cookie-session policy.
5. Hash assets in the signed application manifest and verify before serving;
   define cache/ETag behavior and make CSP configurable within safe constraints.
6. The dedicated production Framework no longer derives from Content Shell.
   Complete the remaining window product surface: add close veto with a bounded
   deadline, window events,
   parent/owner relationships, persistence, menus, tray, dialogs, accessibility,
   and complete platform lifecycle integration.
7. Harden Explorer crash recovery, popup/download/fullscreen/external-protocol
   policy, focus/input/accessibility, navigation APIs, and storage tests on all
   supported platforms.
8. Add fuzzing and golden vectors for manifest/FFI/Shell Protocol, ASan/UBSan,
   enabled/disabled build graph CI, and clean-machine packaging tests.
9. Ship platform signing/notarization, installers, atomic signed updates with
   rollback, crash reporting and symbols, SBOM/notices, provenance, and a
   Chromium security-roll process.
10. Extend the cross-platform Rust packaging CLI with `new` and `build`; refine
    the implemented authenticated `dev`/Vite HMR path with structured diagnostics
    and add asset watching, renderer reload, templates, and
    signed Framework Distribution publication and remote download. Local
    distribution discovery, target/ABI validation, full hash verification, and
    Chromium-output fallback are implemented.
11. Complete production hardening around the implemented Utility session and
    restricted V8 plugin contracts: configurable per-service sandbox policy,
    CPU watchdogs, structured telemetry, explicit cross-process cancellation
    tests, and crash/restart recovery. Session pipes, bounded command queues,
    event credits/acknowledgement, request IDs, cancellation, and V8 heap limits
    are implemented. High-volume PTY/binary streams still require dedicated
    DataPipes rather than renderer application events.

The first four items are release blockers because they define the privilege,
concurrency, and network boundaries. The remaining items are required for a supported
general-purpose desktop framework, even though they do not block this demo.

Windows Framework Distributions include `tuffite_launcher.exe` because Chromium
requires sandbox broker initialization in the process executable. Packaging
installs it under the business executable name; the shared Framework loads the
Rust runtime through the stable C ABI. Both release and
CI distributions exclude development-only runtime data and symbols by default.
The production Framework no longer derives from Content Shell or links its
test-only targets.

The default release profile uses a non-component `tuffite.dll` plus an
application-named sandbox executable. Explicit component mode remains available
for faster native iteration,
matching the bootstrap direction taken by modern CEF. The verified Playground
package is approximately 268 MiB.
