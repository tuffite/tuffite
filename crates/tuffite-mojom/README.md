# tuffite-mojom

Standalone Rust frontend for the handle-free Mojom v0 data profile. It loads
confined imports, lexes and parses contracts, validates types, and computes wire
layouts. It returns a resolved schema rather than SDK-specific generated code.

```rust
use std::path::{Path, PathBuf};

let sources = tuffite_mojom::read_sources(
    Path::new("schemas"), &[PathBuf::from("shell_api.mojom")],
)?;
let schema = tuffite_mojom::compile(&sources)?;
// Inspect schema.module, schema.methods and schema.structures.
# Ok::<(), String>(())
```

The crate has no dependency on the Framework, C ABI, Chromium, CLI, codec runtime,
or Rust code-generation libraries. It invokes no external tools. Lexer, parser,
source loading, schema/layout and compilation each live in separate modules.
Data-only schemas and dotted module names are supported by the frontend; target
language naming restrictions belong to generators.

Tuffite-specific generators live under `tuffite::build::mojom` (an internal
module), shared by `tuff` and the public `tuffite::build::shell_api::Generator`.
That layer owns endpoint names, command IDs, schema fingerprints, and generated
Rust/JavaScript/TypeScript bindings. Rust
generation uses `quote!`, `syn` and `prettyplease`. Official C++ codecs continue
to come from Chromium's Mojom generator.

The v0 profile includes integer enums, nested arrays, nullable references and
nullable scalar fields. Scalar optionals use Chromium's presence-bit/value
packing. Handles, endpoint types, recursive types, defaults, attributes,
versioning and arrays of nullable scalars remain unsupported. Method request
and response data structures are included in the resolved schema. Framework
native adapters feed this same wire IR to Chromium's official C++ generator.

Errors implement `std::error::Error` and `miette::Diagnostic`. Lexer/parser and type-reference errors
retain UTF-8 byte spans and named source text; import failures label the importing
file. Tools can render or inspect diagnostics themselves. The frontend does not
enable miette's terminal/backtrace features. SDK generation renders snippets at
its String-error boundary so Cargo build scripts and `tuff` show the same context.
