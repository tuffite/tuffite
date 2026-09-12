# Tuffite Framework

Tuffite Framework is a cross-platform desktop application framework built from a custom
Chromium `content` framework and a Rust application runtime. It targets the same
problem space as Electron and Tauri while making Rust the control plane and
keeping renderer integration native to Chromium.

These documents describe the target architecture and the first runnable
vertical slice. Chromium customizations, including `<explorer>`, live under
`src/tuffite/`; Playground is the application fixture built on the framework.

## Documents

- [Framework development](development.md): python3 x.py / cargo spo, gclient rolls, recovery, local builds, packaging, and CI.

- [Framework deep review (2026-09-29)](framework-review-2026-09-29.md): maintainability, typed contracts, API standardization, extensibility, GN build costs, and CLI development workflow.

- [Implementation review (2026-09-05)](review-2026-09-05.md): implemented improvements, remaining risks and verification limits.

- [Architecture](architecture.md): terminology, component boundaries, process
  model, ownership, FFI, repository layout, and application model.
- [ShellAPI](shell_api.md): renderer-facing unary commands, application-event
  invalidation, binary wire protocol, permissions, specialized data transports,
  and compatibility.
- [Networking](networking.md): one Chromium Network Service path for Runtime and
  privileged renderer requests, session behavior, streaming, and SSRF policy.
- [Utility services](utility-services.md): application-defined Rust services in
  managed Chromium utility processes, lifecycle, policy, and isolation.
- [Long-lived utility sessions](utility-sessions.md): supervised bidirectional
  sessions, ordered events, credit-based backpressure, and Rust host APIs.
- [Restricted JavaScript extension runtime](javascript-extension-runtime.md):
  in-memory ESM graphs in sandboxed Utility processes, resource limits,
  cancellation, and the Rust authority boundary.
- [Window management](window-management.md): typed multi-window APIs, opaque
  IDs, manifest permissions, platform adapters, and lifecycle semantics.
- [`<explorer>`](explorer.md): ordinary inner `WebContents` embedding, storage
  behavior, security boundaries, and Chromium integration.
- [Build and distribution](build-and-distribution.md): developer workflow,
  artifacts, versioning, packaging, updates, and verification.
- [Implementation plan](implementation-plan.md): staged delivery, acceptance
  criteria, Framework integration status, and review questions.

## Terminology decisions

The following terms are normative across the design:

| Term | Meaning |
| --- | --- |
| **Tuffite Framework** | The complete product: Rust APIs, tooling, Framework, Chromium customization, and distribution system. |
| **Tuffite Framework Distribution** | The versioned prebuilt native bundle consumed by applications: sandbox host, Framework libraries, Chromium runtime, resources, and identity manifest. |
| **Tuffite Runtime** | The Rust control plane. It owns application lifecycle, commands, permissions, windows, ShellAPI Host, extensions, and async work. |
| **Tuffite Framework** | The narrow C++ Chromium `content` framework. It owns Chromium objects and platform integration. |
| **Tuffite FFI** | Only the stable C ABI between the Rust Runtime and C++ Framework. It is not a general name for the C++ layer. |
| **ShellAPI** | The renderer-facing capability API provided by Tuffite Runtime. |
| **Shell Protocol** | The versioned binary messages and attachments used by ShellAPI. It is packaged independently, but belongs architecturally to Runtime. |
| **Chromium** | The browser kernel. It is not called the Tuffite engine. |
| **Playground** | The repository's application fixture, not the framework name. |

`engine` and `native` are intentionally not component names because they hide
ownership boundaries. Use `Runtime`, `Framework`, or `FFI` as appropriate.

## Decisions requested in review

The implementation can proceed with the recommended defaults below. Reviewers
should focus on these decisions first:

1. Packaging installs the sandbox-capable native `tuffite_launcher` under the business
   executable name. Chromium and all child processes therefore use the product
   executable identity directly.
2. The business crate emits a Rust `cdylib` exporting the versioned application
   callback table. Applications reuse the prebuilt bootstrap and Framework and do
   not rebuild Chromium.
3. ShellAPI belongs to Runtime. Its deterministic control codec and TypeScript
   generator are implementation modules of the existing public/runtime crates,
   not additional public packages.
4. Shell Protocol uses generated typed binary control messages, Mojo shared
   buffers for medium attachments, and Mojo data pipes for large or streaming
   data.
5. Command IDs are deterministic internal `u64` wire values derived from stable
   canonical names, with build-time collision checking. Generated bindings,
   rather than an additional lock file, make changes reviewable.
6. `<explorer>` uses a normal non-Guest `WebContents`, reuses the host
   `StoragePartition`, and receives ShellAPI only when its committed origin is
   authorized. The entry origin is implicit; all additional origins are
   explicit.
7. Chromium customizations remain feature-scoped under `src/tuffite/` and every
   upstream hook is isolated by a Tuffite buildflag.
8. Runtime and privileged renderer HTTP requests use Chromium Network Service
   through one Runtime-owned Network Broker; Tuffite does not ship a second Rust
   HTTP stack as its framework-level client.
9. Application-defined utility services are Rust types registered by name,
   allowlisted in the manifest, and hosted in Chromium sandboxed utility
   processes behind a generic Mojo service.

See [Implementation plan](implementation-plan.md#10-review-checklist) for the full
review checklist and unresolved policy choices.
