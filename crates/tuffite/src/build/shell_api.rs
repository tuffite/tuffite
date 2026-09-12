//! ShellAPI source generation and output verification.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::Deserialize;

use super::{emit, emit_files, generate_bindings};

#[derive(Debug)]
pub struct GeneratedBindings {
    /// Public command names and native wire IDs, generated from IDL.
    pub command_ids: BTreeMap<String, String>,
    pub rust: String,
    pub javascript: String,
    pub declarations: String,
    /// Per-input Rust fragments keyed by `<stem>.rs`.
    pub rust_files: BTreeMap<String, String>,
    /// Per-input JavaScript metadata modules keyed by `<stem>.js`.
    pub javascript_files: BTreeMap<String, String>,
    /// Per-input portable declarations keyed by `<stem>.d.ts`.
    pub declaration_files: BTreeMap<String, String>,
    /// Generated C++ files keyed by filename (for example `dialog.h` and
    /// `dialog.cc`), with one pair per input `.d.ts`.
    pub cpp_files: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Write,
    Verify,
}

/// Selects the transport used by the generated JavaScript Proxy facade.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Facade {
    /// Application commands are encoded and sent through binary `ShellAPI` IPC.
    #[default]
    Application,
    /// Framework commands are dispatched to Tuffite's built-in transports.
    Framework,
}

#[derive(Debug)]
pub(super) struct Output {
    pub(super) path: PathBuf,
    pub(super) mode: OutputMode,
}

pub(super) struct BuildGeneration {
    pub(super) bindings: GeneratedBindings,
    pub(super) rerun_paths: BTreeSet<PathBuf>,
}

/// Configures `ShellAPI` generation and its output policy.
#[derive(Debug)]
pub struct Generator {
    manifest: PathBuf,
    rust: Option<Output>,
    rust_file: Option<Output>,
    javascript: Option<Output>,
    cpp: Option<Output>,
    cpp_include_root: String,
    facade: Option<Facade>,
}

impl Generator {
    /// Uses `tuffite.json` in `CARGO_MANIFEST_DIR`, beside the application's
    /// `build.rs`. Outside Cargo, defaults to the current directory.
    #[must_use]
    pub fn new() -> Self {
        Self {
            manifest: std::env::var_os("CARGO_MANIFEST_DIR")
                .map_or_else(|| PathBuf::from("."), PathBuf::from)
                .join("tuffite.json"),
            rust: None,
            rust_file: None,
            javascript: None,
            cpp: None,
            cpp_include_root: "tuffite/framework/shell_api".to_owned(),
            facade: None,
        }
    }

    /// Overrides the conventional manifest path.
    #[must_use]
    pub fn manifest(mut self, path: impl Into<PathBuf>) -> Self {
        self.manifest = path.into();
        self
    }

    /// Writes or verifies `bindings.rs` and per-input Rust fragments together.
    #[must_use]
    pub fn rust(mut self, directory: impl Into<PathBuf>, mode: OutputMode) -> Self {
        self.rust = Some(Output::new(directory, mode));
        self
    }

    /// Writes or verifies `index.js`, `index.d.ts` and per-input modules together.
    #[must_use]
    pub fn javascript(mut self, directory: impl Into<PathBuf>, mode: OutputMode) -> Self {
        self.javascript = Some(Output::new(directory, mode));
        self
    }

    /// SDK maintenance can export an aggregate Rust source without build fragments.
    #[cfg(feature = "cli")]
    pub(crate) fn rust_file(mut self, path: impl Into<PathBuf>, mode: OutputMode) -> Self {
        self.rust_file = Some(Output::new(path, mode));
        self
    }

    /// Writes or verifies per-input C++ headers/sources and shared protocol files.
    #[must_use]
    pub fn cpp(mut self, directory: impl Into<PathBuf>, mode: OutputMode) -> Self {
        self.cpp = Some(Output::new(directory, mode));
        self
    }

    /// Sets the include root used for cross-file generated C++ headers.
    #[must_use]
    pub fn cpp_include_root(mut self, root: impl Into<String>) -> Self {
        self.cpp_include_root = root.into();
        self
    }

    /// Overrides the facade transport declared by the input manifest.
    #[must_use]
    pub fn facade(mut self, facade: Facade) -> Self {
        self.facade = Some(facade);
        self
    }

    pub fn generate(self) -> Result<GeneratedBindings, String> {
        let generated = self.generate_for_build()?;
        for path in generated.rerun_paths {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        Ok(generated.bindings)
    }

    pub(super) fn generate_for_build(self) -> Result<BuildGeneration, String> {
        let (bindings, inputs) =
            generate_bindings(&self.manifest, self.facade, &self.cpp_include_root)?;
        let mut rerun_paths = BTreeSet::from([self.manifest.clone()]);
        rerun_paths.extend(inputs);
        if let Some(output) = self.rust_file {
            output.file(
                &bindings.rust,
                "Rust bindings",
                &self.manifest,
                &mut rerun_paths,
            )?;
        }
        if let Some(output) = self.rust {
            output.child("bindings.rs").file(
                &bindings.rust,
                "Rust bindings",
                &self.manifest,
                &mut rerun_paths,
            )?;
            output.files(
                &bindings.rust_files,
                "Rust ShellAPI fragment",
                ".tuffite-rust-outputs.json",
                &self.manifest,
                &mut rerun_paths,
            )?;
        }
        if let Some(output) = self.javascript {
            output.child("index.js").file(
                &bindings.javascript,
                "JavaScript bindings",
                &self.manifest,
                &mut rerun_paths,
            )?;
            output.child("index.d.ts").file(
                &bindings.declarations,
                "TypeScript declarations",
                &self.manifest,
                &mut rerun_paths,
            )?;
            output.files(
                &bindings.javascript_files,
                "JavaScript ShellAPI module",
                ".tuffite-javascript-outputs.json",
                &self.manifest,
                &mut rerun_paths,
            )?;
            output.files(
                &bindings.declaration_files,
                "TypeScript ShellAPI declaration",
                ".tuffite-declaration-outputs.json",
                &self.manifest,
                &mut rerun_paths,
            )?;
        }
        if let Some(output) = self.cpp {
            output.files(
                &bindings.cpp_files,
                "C++ ShellAPI file",
                ".tuffite-cpp-outputs.json",
                &self.manifest,
                &mut rerun_paths,
            )?;
        }
        Ok(BuildGeneration {
            bindings,
            rerun_paths,
        })
    }
}

impl Default for Generator {
    fn default() -> Self {
        Self::new()
    }
}

impl Output {
    fn new(path: impl Into<PathBuf>, mode: OutputMode) -> Self {
        Self {
            path: path.into(),
            mode,
        }
    }

    fn child(&self, name: &str) -> Self {
        Self::new(self.path.join(name), self.mode)
    }

    fn file(
        &self,
        contents: &str,
        description: &str,
        manifest: &std::path::Path,
        rerun_paths: &mut BTreeSet<PathBuf>,
    ) -> Result<(), String> {
        emit(Some(self), contents, description, manifest)?;
        // Missing or edited generated files must rerun the build script in
        // write mode too, so a clean checkout can regenerate them reliably.
        rerun_paths.insert(self.path.clone());
        Ok(())
    }

    fn files(
        &self,
        files: &BTreeMap<String, String>,
        description: &str,
        plan_name: &str,
        manifest: &std::path::Path,
        rerun_paths: &mut BTreeSet<PathBuf>,
    ) -> Result<(), String> {
        emit_files(Some(self), files, description, plan_name, manifest)?;
        rerun_paths.extend(files.keys().map(|name| self.path.join(name)));
        rerun_paths.insert(self.path.join(plan_name));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conventional_manifest_is_in_the_cargo_package() {
        let root = std::env::var_os("CARGO_MANIFEST_DIR")
            .map_or_else(|| PathBuf::from("."), PathBuf::from);
        assert_eq!(Generator::new().manifest, root.join("tuffite.json"));
        assert_eq!(
            Generator::new().manifest("custom.json").manifest,
            PathBuf::from("custom.json")
        );
    }
}
