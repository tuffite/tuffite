mod archive;
pub(super) mod cargo;
mod formats;
mod icons;
mod nsis;
mod paths;
mod publication;
mod resources;
mod runtime;
mod symbols;

#[cfg(test)]
mod tests;

#[cfg(test)]
use super::filesystem::safe_relative_path;
use super::filesystem::{absolute_path, canonical_directory, io_error, reject_symlink};
use super::processes::Processes;
use crate::{
    build::framework::Distribution,
    ioc::{FactoryContext, Service},
};
#[cfg(test)]
use archive::PackageManifest;
use archive::{
    write_launcher, write_package_manifest, write_packaged_manifest, write_readme, write_zip,
};
#[cfg(test)]
use cargo::application_build_command;
use cargo::{ApplicationRuntime, package_target};
use formats::Format;
use paths::{default_package_name, validate_application_id, validate_package_name};
use publication::{
    finish_package, publish_development, publish_release, remove_dir_all_with_retry,
    validate_output,
};
use resources::PackageApp;
use resources::{
    configure_assets, copy_regular_file, copy_resources, validate_internal_asset_root,
};
use runtime::{
    copy_framework_runtime, executable_name, validate_executable_name, validate_executable_stem,
    validate_runtime_library_name,
};

use anyhow::Context as _;
use serde::Deserialize;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const MANIFEST_FILE: &str = "manifest.json";
const FRAMEWORK_SDK_FILE: &str = "tuffite.js";
const ASSET_ROOT: &str = "assets";

#[derive(clap::Args, Debug)]
pub struct Options {
    /// Application configuration; otherwise search the current directory and its parents.
    #[arg(long)]
    pub(super) manifest: Option<PathBuf>,
    #[arg(long)]
    destination: Option<PathBuf>,
    #[arg(long)]
    name: Option<String>,
    #[arg(long)]
    include_symbols: bool,
    #[arg(long, value_enum, default_value_t = Format::Zip)]
    format: Format,
    /// Keep debug/local symbols in staged binaries.
    #[arg(long)]
    no_strip: bool,
    /// Export private, matching application symbols separately.
    #[arg(long)]
    symbols_dir: Option<PathBuf>,
    #[arg(long)]
    force: bool,
}

pub(super) struct PackageService {
    options: Arc<Options>,
    framework: Arc<Distribution>,
    processes: Arc<dyn Processes>,
}
impl PackageService {
    pub(super) fn new(build: &mut FactoryContext<'_, Options>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self {
            options: build.depends_on::<Service<Options>>()?,
            framework: build.depends_on::<Service<Distribution>>()?,
            processes: build.depends_on::<Service<dyn Processes>>()?,
        }))
    }

    pub(super) fn run(&self) -> anyhow::Result<()> {
        Packager::new(
            self.framework.as_ref(),
            self.processes.as_ref(),
            &self.options,
            Mode::Distribution,
        )
        .run()
        .map(|_| ())
        .map_err(anyhow::Error::msg)
        .context("package application")
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppManifest {
    product_name: Option<String>,
    app: PackageApp,
    #[serde(default)]
    bundle: Bundle,
}

#[derive(Debug, Default, Deserialize)]
struct Bundle {
    executable: Option<String>,
    #[serde(default)]
    resources: Vec<String>,
    #[serde(default)]
    icons: icons::Icons,
}

impl Options {
    pub(super) fn validate(&self) -> Result<(), String> {
        self.format.validate()?;
        let manifest = super::config::resolve_manifest(self.manifest.as_deref())?;
        super::config::check(&manifest).map(|_| ())
    }
}

pub(super) fn prepare_development(
    framework: &crate::build::framework::Distribution,
    processes: &dyn super::processes::Processes,
    manifest: &Path,
    package_dir: Option<&Path>,
    release: bool,
) -> Result<PathBuf, String> {
    let destination = package_dir
        .map(|path| {
            path.parent()
                .ok_or("development package has no parent directory")
        })
        .transpose()?;
    let name = package_dir
        .map(|path| {
            path.file_name()
                .and_then(OsStr::to_str)
                .ok_or("development package name must be valid UTF-8")
        })
        .transpose()?;
    Packager::new(
        framework,
        processes,
        &Options {
            manifest: Some(manifest.to_path_buf()),
            destination: destination.map(Path::to_path_buf),
            name: name.map(str::to_owned),
            include_symbols: false,
            format: Format::Directory,
            no_strip: true,
            symbols_dir: None,
            force: true,
        },
        Mode::Development { release },
    )
    .run()
}

#[derive(Clone, Copy)]
enum Mode {
    Distribution,
    Development { release: bool },
}

struct Packager<'a> {
    framework: &'a crate::build::framework::Distribution,
    processes: &'a dyn super::processes::Processes,
    options: &'a Options,
    mode: Mode,
}

impl<'a> Packager<'a> {
    fn new(
        framework: &'a crate::build::framework::Distribution,
        processes: &'a dyn super::processes::Processes,
        options: &'a Options,
        mode: Mode,
    ) -> Self {
        Self {
            framework,
            processes,
            options,
            mode,
        }
    }

    #[allow(clippy::too_many_lines)]
    fn run(self) -> Result<PathBuf, String> {
        let Self {
            framework,
            processes,
            options,
            mode,
        } = self;
        let (create_archive, release) = match mode {
            Mode::Distribution => (options.format.archive_extension().is_some(), true),
            Mode::Development { release } => (false, release),
        };
        options.format.validate()?;
        let manifest_path = super::config::resolve_manifest(options.manifest.as_deref())?;
        tracing::info!(
            manifest = %manifest_path.display(),
            "packaging Tuffite application"
        );
        super::config::check(&manifest_path)?;

        let project_dir = manifest_path
            .parent()
            .ok_or_else(|| "manifest has no parent directory".to_owned())?;
        let mut packaged_manifest: serde_json::Value = serde_json::from_reader(BufReader::new(
            File::open(&manifest_path).map_err(io_error("open application manifest"))?,
        ))
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
        let command_ids = crate::build::security_command_ids(&manifest_path)?;
        if let Some(rules) = packaged_manifest["app"]["security"]["capabilities"].as_array_mut() {
            for rule in rules {
                if let Some(entries) = rule
                    .get_mut("shellApi")
                    .and_then(serde_json::Value::as_array_mut)
                {
                    let mut compiled = Vec::new();
                    for entry in entries.iter() {
                        compiled.extend(crate::build::compile_security_entry(entry, &command_ids)?);
                    }
                    *entries = compiled;
                }
            }
        }
        // Runtime metadata contains identities, never build-time declaration paths.
        packaged_manifest["app"]["shellApi"] = serde_json::json!({
            "commandIds": command_ids,
        });
        packaged_manifest
            .as_object_mut()
            .expect("validated manifest")
            .remove("build");
        let app_manifest: AppManifest = serde_json::from_value(packaged_manifest.clone())
            .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
        validate_application_id(&app_manifest.app.identifier)?;
        let framework_manifest = framework.manifest();
        tracing::info!(
            framework = %framework.root().display(),
            framework_version = %framework_manifest.framework_version,
            ffi_abi = %format_args!("{}.{}.{}", framework_manifest.ffi_abi_major, framework_manifest.ffi_abi_minor, framework_manifest.ffi_abi_patch),
            target = %framework_manifest.target,
            profile = %framework_manifest.profile,
            variant = %framework_manifest.variant,
            development_enabled = framework_manifest.development_enabled,
            "using Tuffite Framework Distribution"
        );
        let target = package_target(&framework.manifest().target)?;
        if matches!(mode, Mode::Distribution) {
            super::hooks::run(&manifest_path, "beforeBuildCommand", processes)?;
        }
        let application_runtime =
            ApplicationRuntime::build(project_dir, framework, release, processes)?;
        super::hooks::run(&manifest_path, "beforePackageCommand", processes)?;
        let product_name = app_manifest
            .product_name
            .as_deref()
            .unwrap_or(&application_runtime.package_name);
        super::config::validate_product_name(product_name)?;
        packaged_manifest["productName"] = serde_json::Value::String(product_name.to_owned());
        tracing::info!(
            path = %application_runtime.library.display(),
            target = target.triple,
            profile = if release { "release" } else { "debug" },
            "built application runtime"
        );
        let resources = configure_assets(
            project_dir,
            &app_manifest.app,
            &app_manifest.bundle.resources,
            &mut packaged_manifest,
        )
        .map_err(|error| {
            format!("{error}; build the application frontend after Cargo has run build.rs, then retry packaging")
        })?;
        reject_symlink(&application_runtime.library)?;
        if let Some(configured) = app_manifest.bundle.executable.as_deref() {
            validate_executable_stem(configured)?;
        }
        let executable_name = executable_name(
            app_manifest.bundle.executable.as_deref(),
            &application_runtime.package_name,
        );
        validate_executable_name(&executable_name)?;
        let runtime_library = application_runtime
            .library
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| "Cargo cdylib filename must be valid UTF-8".to_owned())?
            .to_owned();
        validate_runtime_library_name(&runtime_library)?;
        let asset_root = PathBuf::from(ASSET_ROOT);
        validate_internal_asset_root(&asset_root, &executable_name, &runtime_library)?;

        let package_name = options.name.clone().unwrap_or_else(|| {
            if matches!(mode, Mode::Development { .. }) {
                application_runtime.package_name.clone()
            } else if options.format.is_bundle() {
                product_name.to_owned()
            } else if app_manifest.product_name.is_some() {
                format!("{product_name}-{}-{}", target.platform, target.architecture)
            } else {
                default_package_name(&app_manifest.app.identifier, &target)
            }
        });
        validate_package_name(&package_name)?;
        if matches!(mode, Mode::Development { .. })
            && options.destination.is_none()
            && matches!(
                package_name.as_str(),
                "build" | "deps" | "examples" | "incremental"
            )
        {
            return Err("crate name conflicts with a Cargo output directory; pass --package to select a development directory".to_owned());
        }
        let destination = absolute_path(options.destination.clone().unwrap_or_else(|| {
            match mode {
                Mode::Development { .. } => application_runtime
                    .library
                    .parent()
                    .expect("Cargo artifact parent")
                    .to_path_buf(),
                Mode::Distribution => project_dir.join("dist"),
            }
        }))?;
        fs::create_dir_all(&destination).map_err(io_error("create package destination"))?;
        let destination = canonical_directory(&destination)?;
        let directory_name = if options.format.is_bundle() {
            format!("{package_name}.app")
        } else {
            package_name.clone()
        };
        let final_package_dir = destination.join(&directory_name);
        if let Some(symbols) = &options.symbols_dir {
            let symbols = absolute_path(symbols.clone())?;
            let parent = symbols.parent().ok_or("symbols directory has no parent")?;
            fs::create_dir_all(parent).map_err(io_error("create symbols parent"))?;
            let symbols = canonical_directory(parent)?
                .join(symbols.file_name().ok_or("invalid symbols directory")?);
            if final_package_dir.starts_with(&symbols) || symbols.starts_with(&final_package_dir) {
                return Err("symbols directory overlaps the final application package".into());
            }
        }
        let final_archive_path = destination.join(format!(
            "{package_name}.{}",
            options.format.archive_extension().unwrap_or("zip")
        ));
        let staging_suffix = format!("{}.{}", package_name, std::process::id());
        let staging_package_dir = destination.join(format!(
            ".{staging_suffix}.staging{}",
            if options.format.is_bundle() {
                ".app"
            } else {
                ""
            }
        ));
        let staging_archive_path = destination.join(format!(
            ".{staging_suffix}.staging.{}",
            options.format.archive_extension().unwrap_or("zip")
        ));
        let (package_dir, archive_path) = if create_archive {
            validate_output(
                &final_package_dir,
                Some(final_archive_path.as_path()),
                options.force,
            )?;
            if staging_package_dir.exists() {
                reject_symlink(&staging_package_dir)?;
                remove_dir_all_with_retry(&staging_package_dir)?;
            }
            if staging_archive_path.exists() {
                reject_symlink(&staging_archive_path)?;
                fs::remove_file(&staging_archive_path)
                    .map_err(io_error("remove stale staging archive"))?;
            }
            fs::create_dir(&staging_package_dir)
                .map_err(io_error("create staging package directory"))?;
            (staging_package_dir.clone(), staging_archive_path.clone())
        } else {
            if matches!(mode, Mode::Distribution) {
                validate_output(&final_package_dir, None, options.force)?;
            }
            if staging_package_dir.exists() {
                reject_symlink(&staging_package_dir)?;
                remove_dir_all_with_retry(&staging_package_dir)?;
            }
            fs::create_dir(&staging_package_dir)
                .map_err(io_error("create development staging package directory"))?;
            (staging_package_dir.clone(), final_archive_path.clone())
        };

        let result = (|| {
            let runtime = copy_framework_runtime(
                framework,
                &package_dir,
                &asset_root,
                OsStr::new(&executable_name),
                options.include_symbols,
                matches!(mode, Mode::Development { .. })
                    .then(|| {
                        application_runtime
                            .library
                            .parent()
                            .expect("Cargo artifact parent")
                            .join(".tuffite-framework")
                    })
                    .as_deref(),
            )?;
            copy_regular_file(
                &application_runtime.library,
                &package_dir.join(&runtime_library),
            )?;
            let private_symbols = if matches!(mode, Mode::Distribution) {
                if target.platform == "macos" {
                    formats::relocate_libraries(&package_dir, options.format.is_bundle())?;
                }
                symbols::SymbolProcessor {
                    root: &package_dir,
                    application_library: &application_runtime.library,
                    strip: !options.no_strip,
                    destination: options.symbols_dir.as_deref(),
                    force: options.force,
                    identifier: &app_manifest.app.identifier,
                    target: &target,
                }
                .prepare()?
            } else {
                None
            };
            app_manifest.bundle.icons.install(
                &package_dir,
                project_dir,
                target.platform,
                &executable_name,
            )?;
            write_packaged_manifest(&package_dir.join("tuffite.json"), &packaged_manifest)?;
            copy_resources(project_dir, &asset_root, &resources, &package_dir)?;
            if !options.format.is_bundle() {
                write_launcher(&package_dir, OsStr::new(&executable_name))?;
            }
            write_readme(
                &package_dir,
                OsStr::new(&executable_name),
                options.format.is_bundle(),
            )?;

            write_package_manifest(
                &package_dir,
                &app_manifest.app.identifier,
                executable_name.clone(),
                runtime_library,
                runtime.development_enabled,
                &target,
                framework.manifest(),
                options.format.is_bundle(),
            )?;
            let bundle = formats::ApplicationBundle {
                identifier: &app_manifest.app.identifier,
                product_name,
                executable: &executable_name,
                version: &application_runtime.version,
            };
            if options.format.is_bundle() {
                bundle.assemble(&package_dir)?;
            }
            if create_archive {
                if options.format == Format::Zip {
                    write_zip(&package_dir, &archive_path, &package_name)?;
                } else {
                    bundle.installer(
                        options.format,
                        &package_dir,
                        &archive_path,
                        &directory_name,
                    )?;
                }
                publish_release(
                    &package_dir,
                    &archive_path,
                    &final_package_dir,
                    &final_archive_path,
                )?;
                if let Some(symbols) = private_symbols {
                    symbols.publish()?;
                }
                finish_package(&final_package_dir, Some(&final_archive_path))?;
            } else {
                publish_development(&package_dir, &final_package_dir)?;
                if let Some(symbols) = private_symbols {
                    symbols.publish()?;
                }
                finish_package(&final_package_dir, None)?;
            }
            Ok(())
        })();

        if result.is_err() && package_dir == staging_package_dir {
            let _ = fs::remove_dir_all(&package_dir);
            if create_archive {
                let _ = fs::remove_file(&archive_path);
            }
        }
        result.map(|()| final_package_dir)
    }
}
