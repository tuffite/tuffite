# Tuffite implementation plan

## 1. Principles

- Stabilize ownership and protocol boundaries before expanding convenience APIs.
- Preserve the working `<explorer>` prototype while moving it in one reviewable
  migration.
- Keep Chromium changes buildable with each Tuffite feature disabled.
- Ship a thin vertical slice before creating a large crate or npm package graph.
- Treat security policy, lifecycle cancellation, and diagnostics as part of each
  feature rather than a final hardening phase.

## 2. Phase 0: architecture baseline

Deliverables:

- approve terminology and component ownership in these documents;
- finalize repository structure and supported platform baseline;
- reserve/publish the first legitimate `tuffite` crate and npm package after
  legal and availability checks;
- choose the permanent command-hash and Shell Protocol domains;
- record architecture decisions as short ADRs when choices change.

Exit criteria:

- no use of `engine` or `native` as ambiguous component names;
- ShellAPI is owned by Runtime and Shell Protocol has a defined package boundary;
- Framework and FFI responsibilities are accepted;
- review checklist below is resolved or explicitly deferred.

## 3. Phase 1: host, Runtime, and FFI vertical slice

Deliverables:

- minimal `content`-based bootstrap template for Windows, then macOS and Linux;
- versioned C ABI function table and generation-aware handles;
- Rust Runtime bootstrap, shutdown, and async task supervision;
- one business Rust runtime library exported through `tuffite::app::Builder` and a
  prebuilt bootstrap installed under the business executable name;
- one `app://` window with packaged resources;
- structured startup and lifecycle tracing.
- explicit Rust panic and C++ failure containment at every FFI entry point.

Exit criteria:

- open and close one application window cleanly;
- stale handles and shutdown races fail safely;
- no Chromium C++ type crosses FFI;
- neither Rust unwinding nor C++ exceptions cross FFI;
- Chromium's UI sequence is never blocked waiting for a Rust future;
- an ABI mismatch fails before application entry;
- the sample application does not rebuild Chromium.

## 4. Phase 2: ShellAPI vertical slice

Deliverables:

- Shell Binding in the trusted App renderer;
- Shell Protocol envelope and generated deterministic binary codec;
- deterministic internal Command IDs emitted into generated bindings;
- Runtime ShellAPI Host, command router, and capability checks;
- request/response, cancellation, typed errors, and tracing;
- one shared-buffer command and one backpressured data-pipe command;
- TypeScript facade generated from the same command metadata.

Exit criteria:

- no JSON in the framework invocation path;
- ID collision and wire rename tests fail at build time;
- document navigation revokes outstanding authorization;
- only primary documents at manifest-authorized committed origins obtain ShellAPI;
- Rust/C++/TypeScript golden vectors agree;
- malformed input fuzzing produces no unsafe access or unbounded allocation.

## 5. Phase 3: Chromium Network Service vertical slice

Deliverables:

- Runtime-owned Network Broker and capability policy;
- Framework URLLoaderFactory/NetworkContext adapter using the application
  StoragePartition;
- `tuffite::net::Client` for Rust application code;
- manifest-backed `window.tuffite.net.request()` Proxy facade;
- shared-buffer and backpressured upload/download data-pipe paths;
- redirect, credentials, header, private-network, size, timeout, and concurrency
  enforcement;
- request cancellation and privacy-preserving tracing.

Exit criteria:

- Rust and trusted App requests use Chromium Network Service, proxy, cookies,
  cache, certificate verification, and sandbox;
- credentials are omitted by default and require a destination capability;
- initial URLs and every redirect are policy checked;
- public-to-private resolution and localhost access are denied unless separately
  declared;
- slow upload/download consumers retain bounded memory and propagate backpressure;
- document navigation or `AbortSignal` cancels the underlying URLLoader;
- `<explorer>` can invoke built-ins only when its committed origin is explicitly authorized.

## 6. Phase 4: migrate and harden `<explorer>`

The Explorer source and buildflag integration is complete. Reusable sources live
under `src/tuffite/components/explorer/`; the dedicated Content and Blink
integration targets use `tuffite_in_chromium` and `TUFFITE_IN_CHROMIUM`.
Generated include paths, GN targets, registrations, tests, and docs use these
names. Playground exercises the normal-WebContents integration. The remaining
work in this phase is production hardening:

Hardening deliverables:

- `WebContentsRole::Explorer`;
- Runtime navigation and permission policy integration;
- document lifecycle and rapid attribute-change race handling;
- crash, popup, download, fullscreen, external-protocol, and accessibility tests;
- explicit origin-policy tests proving default denial and deliberate Explorer grants;
- enabled and disabled build graph verification.

Exit criteria:

- normal, non-Guest WebContents behavior is preserved;
- same-layer composition works across supported platforms;
- host StoragePartition reuse matches documented semantics;
- all privileged API checks include primary-document identity and committed origin;
- `TUFFITE_IN_CHROMIUM=0` leaves no Tuffite implementation in the graph.

## 7. Phase 5: Framework Distribution and application tooling

Deliverables:

- public `tuffite` API and minimal supporting crates;
- `tuffite new`, `dev`, `build`, and `package`;
- application manifest, capability policy, and generated metadata;
- development asset workflow and production `app://` package;
- sample application using commands, streams, and Explorer;
- diagnostics for version mismatch, capability denial, and renderer crash.

Exit criteria:

- a new application reaches a running window with one documented workflow;
- application source contains no C++ or direct FFI calls;
- production packaging contains no development server dependency;
- generated files are reproducible and checked for drift.

## 8. Phase 6: platform packaging and release

Deliverables:

- signed Windows, macOS, and Linux artifacts;
- updater with signed manifests, atomic activation, and rollback;
- crash reporting and symbol pipeline with privacy controls;
- SBOM, third-party notices, provenance, and security response process;
- Chromium roll automation and compatibility dashboard.

Exit criteria:

- clean-machine install, launch, update, rollback, and uninstall pass in CI;
- sandboxing remains enabled in production;
- framework and application version mismatch cannot activate a broken update;
- a Chromium security release can be rolled without an application rebuild when
  the Tuffite ABI and protocol remain compatible.

## 9. Migration status

| Area | Current state | Tuffite target |
| --- | --- | --- |
| Framework name | Tuffite | Tuffite |
| Application fixture | `playground/` | Root-level Playground |
| Chromium custom root | Migrated to `src/tuffite/` | Keep all custom features under `src/tuffite/` |
| Chromium integration | `TUFFITE_IN_CHROMIUM` implemented | Keep enabled/disabled graph coverage |
| Explorer implementation | Runnable normal-WebContents vertical slice | Role-aware, policy-integrated production feature |
| Window management | Opaque-ID multi-window lifecycle with capability limits | Events, close veto, ownership, persistence, and full platform CI |
| Rust Runtime | Runnable binary-command vertical slice | Full `tuffite-runtime` control plane |
| C++ layer term | Previously discussed as engine/native | Tuffite Framework |
| ABI boundary | Narrow Tuffite FFI implemented for the demo | Versioned production Tuffite FFI |
| Renderer IPC | Binary ShellAPI request/response vertical slice | Runtime-owned ShellAPI with independent Shell Protocol |
| Unified networking | Demo request path uses Chromium Network Service | Full Runtime Network Broker backed by Chromium Network Service |
| Command IDs | Build-time BLAKE3 IDs with collision checking | Internal generated transport detail; no public macro or separate lock file |
| App manifest | Versioned `tuffite.json` loaded before browser startup | Schema evolution and signed package integration |
| Utility services | Registered Rust services in sandboxed Chromium utility processes | Managed instances, quotas, cancellation, pooling, restart policy, and per-service sandbox profiles |
| Developer workflow | Outer mise, `python3 x.py`, and application Rust CLI for generate/check/dev/build/package/smoke | Asset watching |

### Runnable vertical slice status

The Windows x64 Playground vertical slice is implemented under `playground/`
and verified from a fresh ZIP extraction. Its automated smoke marker confirms
all three paths together and includes final-compositor pixel evidence:

```json
{"status":"ok","shell_api":true,"network_service":true,"utility":true,"window":true,"explorer":true}
```

This milestone includes the Rust application runtime, versioned Framework
callback table, binary Shell Protocol command, application-defined utility service,
manifest-constrained JavaScript Proxy facade, browser-process Network Service
request, normal inner-WebContents `<explorer>`, per-file package manifest,
opaque-ID multi-window lifecycle, Windows AppContainer ACL preparation, and a
development ZIP. Cross-platform
installers, signing, updater, streaming transport, and the public CLI remain
later production phases.

The implemented baseline and explicitly remaining release blockers are tracked
in [production-readiness.md](production-readiness.md).

## 10. Review checklist

### Naming and boundaries

- [ ] Approve `Tuffite Framework`, `Runtime`, `Framework`, and `FFI` terminology.
- [ ] Approve ShellAPI as a Runtime service and Shell Protocol as an independent
      package.
- [x] Keep generation-aware handles in `tuffite-runtime`; do not publish a
      one-module `tuffite-core` crate.
- [ ] Decide whether the permanent protocol/hash domain should use `tuffite` or
      an organization-owned brand-independent string.

### Runtime and application loading

- [ ] Approve the in-browser-process Rust Runtime for the first release.
- [x] Install the native sandbox bootstrap under the business executable name
      and load a version-matched Rust application `cdylib` through the C ABI.
- [ ] Define whether an application crash is allowed to terminate the browser
      process initially or requires a future out-of-process Runtime.
- [ ] Set minimum Windows, macOS, Linux, CPU architecture, and libc targets.

### ShellAPI

- [x] Use the TypeScript-first generated binary control codec plus
      shared-buffer/data-pipe attachments.
- [ ] Approve deterministic `u64` BLAKE3 Command IDs as an internal wire detail.
- [ ] Choose initial attachment thresholds and maximum request sizes.
- [ ] Define which commands may outlive their initiating document.
- [x] Approve deny-by-default document, origin, capability, and constraint checks.

### Networking

- [ ] Approve Chromium Network Service as Tuffite's framework-level HTTP stack.
- [ ] Approve `window.tuffite.net.request()` as privileged policy-based I/O, not
      a replacement implementation of browser `fetch()`.
- [ ] Approve credential omission by default and explicit destination/session
      policy for cookie use.
- [ ] Define localhost, private-network, proxy, client-certificate, and HTTP-auth
      product policy.
- [ ] Decide whether ephemeral/named sessions are required for the first release.

### Explorer

- [ ] Approve shared StoragePartition as default and named isolation as a future
      explicit option.
- [x] Confirm Explorer receives ShellAPI only when its committed origin is authorized.
- [ ] Decide the minimum stable DOM API before adding navigation-history methods.
- [x] Keep Explorer implementation, symbols, and buildflags under the Tuffite namespace.

### Distribution

- [ ] Recheck crates.io/npm availability and complete legal/domain review.
- [ ] Choose initial package formats and update mechanism per platform.
- [ ] Define Chromium security-roll service level and supported release channels.
- [ ] Decide whether npm exposes only generated types or also owns the primary CLI.

## 11. Recommended first review outcome

Approve Phases 0-3 and the security invariants now. Keep these choices open until
the vertical slice produces measurements:

- exact shared-buffer/data-pipe zero-copy adapter ergonomics;
- shared-buffer threshold;
- renderer API naming details;
- out-of-process Runtime;
- dynamic plugin ABI;
- expanded Explorer history and navigation API.

This avoids freezing convenience APIs before the lifecycle, ABI, and binary IPC
boundaries are proven.
