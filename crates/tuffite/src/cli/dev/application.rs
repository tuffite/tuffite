//! Native package preparation, identity validation, and launch arguments.
use super::super::filesystem::reject_symlink;
use super::configuration::Configuration;
use super::server::{DEV_TOKEN_ENV, DevelopmentToken};
use processkit::{Command, StdioMode};
use serde::Deserialize;
use std::ffi::OsString;
use std::fs::File;
use std::path::PathBuf;
use url::Url;

pub(super) struct DevelopmentApplication {
    pub(super) package: PathBuf,
    pub(super) executable: PathBuf,
}

pub(super) struct Launch<'a> {
    pub(super) url: &'a Url,
    pub(super) token: &'a DevelopmentToken,
    pub(super) args: &'a [OsString],
    pub(super) devtools_port: Option<u16>,
}

#[derive(Deserialize)]
struct PackageManifest {
    schema_version: u32,
    framework: String,
    framework_version: String,
    framework_target: String,
    framework_profile: String,
    framework_variant: String,
    ffi_abi_major: u16,
    ffi_abi_minor: u16,
    ffi_abi_patch: u16,
    application: String,
    platform: String,
    architecture: String,
    executable: String,
    runtime_library: String,
    #[serde(default)]
    development_enabled: bool,
}

impl DevelopmentApplication {
    pub(super) fn prepare(
        framework: Option<&crate::build::framework::Distribution>,
        processes: &dyn super::super::processes::Processes,
        configuration: &Configuration,
        release: bool,
    ) -> Result<Self, String> {
        let requested_package = if let Some(manifest) = configuration.manifest.as_deref() {
            super::super::package::prepare_development(
                framework.ok_or("development preparation requires Framework")?,
                processes,
                manifest,
                configuration.package.as_deref(),
                release,
            )?
        } else {
            configuration
                .package
                .clone()
                .ok_or("development package is required without a manifest")?
        };
        let package = std::fs::canonicalize(&requested_package).map_err(|error| {
            format!(
                "resolve development package {} after automatic preparation: {error}",
                requested_package.display()
            )
        })?;
        if !package.is_dir() {
            return Err(format!(
                "development package is not a directory: {}",
                package.display()
            ));
        }
        let manifest_path = package.join("manifest.json");
        reject_symlink(&manifest_path)?;
        let manifest: PackageManifest = serde_json::from_reader(
            File::open(&manifest_path)
                .map_err(|error| format!("open {}: {error}", manifest_path.display()))?,
        )
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
        if !manifest.development_enabled {
            return Err(format!(
                "package {} was built with development URLs disabled; use a development-enabled Framework Distribution",
                package.display()
            ));
        }
        let executable = package.join(&manifest.executable);
        let runtime_library = package.join(&manifest.runtime_library);
        reject_symlink(&executable)?;
        reject_symlink(&runtime_library)?;
        if !executable.is_file() || executable.parent() != Some(package.as_path()) {
            return Err("package manifest executable must be a package-root file".to_owned());
        }
        if !runtime_library.is_file() || runtime_library.parent() != Some(package.as_path()) {
            return Err("package manifest runtime library must be a package-root file".to_owned());
        }
        tracing::info!(
            package = %package.display(),
            package_schema = manifest.schema_version,
            framework = %manifest.framework,
            framework_version = %manifest.framework_version,
            ffi_abi = %format_args!("{}.{}.{}", manifest.ffi_abi_major, manifest.ffi_abi_minor, manifest.ffi_abi_patch),
            framework_target = %manifest.framework_target,
            framework_profile = %manifest.framework_profile,
            framework_variant = %manifest.framework_variant,
            application = %manifest.application,
            platform = %manifest.platform,
            architecture = %manifest.architecture,
            executable = %executable.display(),
            runtime_library = %runtime_library.display(),
            "development package identity"
        );
        Ok(Self {
            package,
            executable,
        })
    }

    pub(super) fn command(&self, launch: &Launch<'_>) -> Command {
        let mut command = Command::new(&self.executable)
            .current_dir(&self.package)
            .env(DEV_TOKEN_ENV, launch.token.value())
            .arg(format!("--tuffite-dev-url={}", launch.url))
            .arg(format!("--tuffite-dev-token={}", launch.token.digest()))
            .args(launch.args);
        if let Some(port) = launch.devtools_port {
            command = command.arg(format!("--remote-debugging-port={port}"));
        }
        command
            .stdout(StdioMode::Inherit)
            .stderr(StdioMode::Inherit)
            .kill_on_parent_death()
    }
}
