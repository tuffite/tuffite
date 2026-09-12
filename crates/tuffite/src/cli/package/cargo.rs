//! Cargo operations for application packaging.
use super::super::filesystem::{canonical_file, io_error};
use serde::Deserialize;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    manifest_path: PathBuf,
    targets: Vec<CargoTarget>,
}

#[derive(Debug, Deserialize)]
struct CargoTarget {
    name: String,
    crate_types: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CargoMessage {
    reason: String,
    target: Option<CargoTarget>,
    filenames: Option<Vec<PathBuf>>,
    message: Option<CargoDiagnostic>,
}

#[derive(Debug, Deserialize)]
struct CargoDiagnostic {
    rendered: Option<String>,
}

pub(super) struct ApplicationRuntime {
    pub(super) package_name: String,
    pub(super) version: String,
    pub(super) library: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PackageTarget<'a> {
    pub(super) triple: &'a str,
    pub(super) platform: &'static str,
    pub(super) architecture: &'static str,
}

impl ApplicationRuntime {
    pub(super) fn build(
        project_dir: &Path,
        framework: &crate::build::framework::Distribution,
        release: bool,
    ) -> Result<Self, String> {
        let target_triple = &framework.manifest().target;
        let manifest = canonical_file(&project_dir.join("Cargo.toml"))?;
        let mut metadata_command = cargo_command();
        let metadata_output = metadata_command
            .args([
                "metadata",
                "--format-version",
                "1",
                "--no-deps",
                "--manifest-path",
            ])
            .arg(&manifest)
            .output()
            .map_err(|error| format!("run cargo metadata: {error}"))?;
        if !metadata_output.status.success() {
            return Err(format!(
                "cargo metadata failed for {}:\n{}",
                manifest.display(),
                String::from_utf8_lossy(&metadata_output.stderr)
            ));
        }
        let metadata: CargoMetadata = serde_json::from_slice(&metadata_output.stdout)
            .map_err(|error| format!("parse cargo metadata for {}: {error}", manifest.display()))?;
        let package = metadata
            .packages
            .iter()
            .find(|package| {
                fs::canonicalize(&package.manifest_path)
                    .is_ok_and(|candidate| candidate == manifest)
            })
            .ok_or_else(|| format!("Cargo package not found for {}", manifest.display()))?;
        let mut cdylibs = package
            .targets
            .iter()
            .filter(|target| target.crate_types.iter().any(|kind| kind == "cdylib"));
        let target = cdylibs
            .next()
            .ok_or_else(|| format!("Cargo package {} has no cdylib target", package.name))?;
        if cdylibs.next().is_some() {
            return Err(format!(
                "Cargo package {} has multiple cdylib targets",
                package.name
            ));
        }

        tracing::info!(
            package = %package.name,
            target = %target.name,
            "building application cdylib"
        );
        let mut build_command =
            application_build_command(&manifest, target_triple, framework.root(), release);
        let mut child = build_command
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|error| format!("run cargo build: {error}"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "capture cargo build output failed".to_owned())?;
        let mut library = None;
        for line in BufReader::new(stdout).lines() {
            let line = line.map_err(io_error("read cargo build output"))?;
            let message: CargoMessage = serde_json::from_str(&line)
                .map_err(|error| format!("parse cargo build output: {error}"))?;
            if message.reason == "compiler-message"
                && let Some(rendered) = message.message.and_then(|message| message.rendered)
            {
                eprint!("{rendered}");
            }
            if message.reason != "compiler-artifact"
                || message.target.as_ref().map(|item| item.name.as_str())
                    != Some(target.name.as_str())
            {
                continue;
            }
            library = message
                .filenames
                .unwrap_or_default()
                .into_iter()
                .find(|path| is_dynamic_library(path));
        }
        let status = child
            .wait()
            .map_err(|error| format!("wait for cargo build: {error}"))?;
        if !status.success() {
            return Err(format!("cargo build failed for {}", manifest.display()));
        }
        let library = library
            .ok_or_else(|| format!("cargo did not report a cdylib artifact for {}", target.name))?;
        Ok(ApplicationRuntime {
            package_name: package.name.clone(),
            version: package.version.clone(),
            library: canonical_file(&library)?,
        })
    }
}

pub(super) fn application_build_command(
    manifest: &Path,
    target_triple: &str,
    framework_root: &Path,
    release: bool,
) -> Command {
    let mut command = cargo_command();
    command
        .env("TUFFITE_FRAMEWORK_DIST", framework_root)
        .arg("build");
    if release {
        command.arg("--release");
    }
    command
        .args([
            "--lib",
            "--target",
            target_triple,
            "--message-format=json-render-diagnostics",
            "--manifest-path",
        ])
        .arg(manifest);
    command
}

pub(in crate::cli) fn cargo_command() -> Command {
    let mut command = Command::new("cargo");
    if let Some(root) = env::var_os("TUFFITE_FRAMEWORK_DIR") {
        let path = PathBuf::from(root).join("crates/tuffite");
        let portable = path.to_string_lossy().replace('\\', "/");
        command.args([
            "--config",
            &format!("patch.crates-io.tuffite.path='{portable}'"),
        ]);
    }
    command
}

pub(super) fn is_dynamic_library(path: &Path) -> bool {
    let expected = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    path.extension().and_then(OsStr::to_str) == Some(expected)
}

pub(super) fn package_target(triple: &str) -> Result<PackageTarget<'_>, String> {
    let architecture = if triple.starts_with("x86_64-") {
        "x64"
    } else if triple.starts_with("aarch64-") {
        "arm64"
    } else {
        return Err(format!(
            "unsupported Tuffite Framework target architecture: {triple}"
        ));
    };
    let platform = if triple.ends_with("-apple-darwin") {
        "macos"
    } else if triple.contains("-windows-") {
        "windows"
    } else if triple.contains("-linux-") {
        "linux"
    } else {
        return Err(format!(
            "unsupported Tuffite Framework target platform: {triple}"
        ));
    };
    if platform != env::consts::OS {
        return Err(format!(
            "Tuffite packaging supports cross-architecture builds on {host}, but Framework target {triple} is for {platform}",
            host = env::consts::OS
        ));
    }
    Ok(PackageTarget {
        triple,
        platform,
        architecture,
    })
}
