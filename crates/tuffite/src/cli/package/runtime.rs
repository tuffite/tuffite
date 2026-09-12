//! Runtime operations for application packaging.
use super::super::filesystem::portable_path;
use super::paths::validate_package_name;
use super::resources::copy_regular_file;
use super::{FRAMEWORK_SHELL_API_FILE, MANIFEST_FILE, RENDERER_RUNTIME_FILE};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

pub(super) struct RuntimeMetadata {
    pub(super) development_enabled: bool,
}

pub(super) fn copy_framework_runtime(
    framework: &crate::build::framework::Distribution,
    package_dir: &Path,
    asset_root: &Path,
    application_name: &OsStr,
    include_symbols: bool,
) -> Result<RuntimeMetadata, String> {
    let previous = previous_package_hashes(package_dir);
    let bootstrap = framework.bootstrap(false)?;
    // Install the sandbox-capable Tuffite launcher as the business executable.
    // Chromium subprocesses re-enter this same application-named image; the
    // Rust implementation remains isolated in the application cdylib.
    let application_path = package_dir.join(application_name);
    let application_key = application_name.to_string_lossy();
    copy_if_changed_from_manifest(
        &bootstrap,
        &application_path,
        &application_key,
        framework.manifest().bootstrap.size,
        &framework.manifest().bootstrap.sha256,
        &previous,
    )?;
    // `python3 x.py verify` owns full Framework artifact hashing. Packaging validates
    // manifest sizes and then hashes the final package once.
    let mut renderer_runtime = false;
    let mut framework_shell_api = false;
    for (source, relative) in framework.runtime_files(false)? {
        let destination = if relative == Path::new(RENDERER_RUNTIME_FILE) {
            renderer_runtime = true;
            package_dir.join(asset_root).join(&relative)
        } else if relative == Path::new(FRAMEWORK_SHELL_API_FILE) {
            framework_shell_api = true;
            package_dir.join(asset_root).join(&relative)
        } else {
            package_dir.join(&relative)
        };
        if include_symbols || !is_symbol_file(&relative) {
            let key = portable_path(
                destination
                    .strip_prefix(package_dir)
                    .map_err(|_| "framework runtime destination escaped package".to_owned())?,
            )?;
            let source_key = portable_path(&relative)?;
            let entry = framework
                .manifest()
                .runtime_files
                .iter()
                .find(|entry| entry.path == source_key)
                .ok_or_else(|| format!("missing Framework runtime metadata for {key}"))?;
            copy_if_changed_from_manifest(
                &source,
                &destination,
                &key,
                entry.size,
                &entry.sha256,
                &previous,
            )?;
        }
    }
    if !renderer_runtime || !framework_shell_api {
        return Err("Framework Distribution is missing its version-matched Web runtime".to_owned());
    }
    Ok(RuntimeMetadata {
        development_enabled: framework.manifest().development_enabled,
    })
}

pub(super) fn previous_package_hashes(package_dir: &Path) -> BTreeMap<String, String> {
    let Ok(bytes) = fs::read(package_dir.join(MANIFEST_FILE)) else {
        return BTreeMap::new();
    };
    let Ok(manifest) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return BTreeMap::new();
    };
    manifest
        .get("files")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some((
                entry.get("path")?.as_str()?.to_owned(),
                entry.get("sha256")?.as_str()?.to_owned(),
            ))
        })
        .collect()
}

pub(super) fn copy_if_changed_from_manifest(
    source: &Path,
    destination: &Path,
    key: &str,
    expected_size: u64,
    expected_hash: &str,
    previous: &BTreeMap<String, String>,
) -> Result<(), String> {
    if previous.get(key).is_some_and(|hash| hash == expected_hash)
        && destination
            .metadata()
            .is_ok_and(|metadata| metadata.is_file() && metadata.len() == expected_size)
    {
        return Ok(());
    }
    copy_regular_file(source, destination)
}

pub(super) fn is_symbol_file(path: &Path) -> bool {
    path.components()
        .any(|part| part.as_os_str().to_string_lossy().ends_with(".dSYM"))
        || path
            .extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| {
                matches!(
                    extension.to_ascii_lowercase().as_str(),
                    "pdb" | "lib" | "exp" | "ilk" | "debug" | "dwp"
                )
            })
}

pub(super) fn executable_name(configured: Option<&str>, package_name: &str) -> String {
    let stem = configured.unwrap_or(package_name);
    if cfg!(target_os = "windows") {
        format!("{stem}.exe")
    } else {
        stem.to_owned()
    }
}

pub(super) fn validate_runtime_library_name(value: &str) -> Result<(), String> {
    validate_package_name(value)?;
    let expected = if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    if Path::new(value).extension().and_then(OsStr::to_str) != Some(expected) {
        return Err(format!(
            "Cargo runtime library must use the .{expected} extension"
        ));
    }
    Ok(())
}

pub(super) fn validate_executable_name(value: &str) -> Result<(), String> {
    validate_package_name(value)?;
    if cfg!(target_os = "windows") && !value.to_ascii_lowercase().ends_with(".exe") {
        return Err("Windows application executable name must end with .exe".to_owned());
    }
    Ok(())
}

pub(super) fn validate_executable_stem(value: &str) -> Result<(), String> {
    validate_package_name(value)?;
    if Path::new(value).extension().is_some() {
        return Err("bundle.executable must not include a file extension".to_owned());
    }
    Ok(())
}
