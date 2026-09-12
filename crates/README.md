# Tuffite Rust workspace

This workspace contains the Rust side of the runnable Tuffite vertical slice:

- `tuffite-abi`: stable C ABI declarations shared with Framework;
- `tuffite-runtime`: Runtime lifecycle, generation-aware handles, task
  supervision, and reusable C ABI implementation;
- `tuffite-macros`: typed `#[tuffite::command(endpoint)]` handler bindings;
- `tuffite`: the safe application-facing Rust facade.

The canonical C header is `framework/public/abi.h`. Static assertions in the
Framework bridge check the public table layout and version; the packaged
Playground exercises the complete host-to-Rust-runtime boundary.

Application crates depend only on `tuffite`, declare `#[tuffite::browser]` and named `#[tuffite::utility(name = ...)]` implementations, and
construct `app::Builder::<Browser>`. The crate emits its single Cargo `[lib]` target as a
cdylib through `#[tuffite::main(browser = BrowserType, utilities(UtilityTypes...))]`. The
application-named executable loads the resulting library and passes
its versioned callback table to Framework. Applications neither write unsafe FFI
functions nor depend directly on `tuffite-abi`. The current
binary Shell Protocol, the manifest-defined application scheme, ShellAPI,
Network Service, and `<explorer>` paths are exercised by Playground.

Run the Rust checks from the repository root:

```console
cargo fmt --all -- --check
cargo clippy --workspace --exclude playground --all-targets -- -D warnings
cargo test --workspace --exclude playground
```

Runtime errors use `thiserror`, retaining the public error variants and adding
source chaining for nested handle failures. Runtime task spans and shutdown
telemetry use `tracing`; libraries do not install a global subscriber. Playground
installs `tracing-subscriber` and accepts `RUST_LOG` (for example,
`tuffite_runtime=debug`). Its Windows GUI executable may have no attached stderr;
run with redirected stderr to collect these diagnostics.
The CLI uses `anyhow` for command context and `tracing-subscriber::fmt` for
diagnostics on stderr. Stable `key=value` artifact results remain on stdout for
scripts. Set `RUST_LOG=info` or `RUST_LOG=debug` for progressively detailed CLI
events. Build/packaging helpers still return string diagnostics; preserving
their underlying I/O sources is follow-up work.

The public `tuffite` crate also owns the asynchronous remoting client under
`tuffite::remoting`. Enable its optional `remoting` feature; see
[Remoting](../docs/remoting.md). Applications own session shutdown and executors.
