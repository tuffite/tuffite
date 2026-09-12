//! Resources operations for application packaging.
use super::super::filesystem::{
    canonical_directory, canonical_file, io_error, portable_relative_path, reject_symlink,
    safe_relative_path,
};
use super::archive::collect_files;
use super::{FRAMEWORK_SHELL_API_FILE, MANIFEST_FILE, RENDERER_RUNTIME_FILE};
use serde::Deserialize;
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PackageApp {
    pub(super) identifier: String,
    #[serde(default)]
    assets: Vec<AssetRoute>,
}

#[derive(Debug, Deserialize)]
struct AssetRoute {
    file: Option<String>,
    directory: Option<String>,
}

pub(super) fn configure_assets(
    project_dir: &Path,
    app: &PackageApp,
    bundle_resources: &[String],
    _packaged_manifest: &mut serde_json::Value,
) -> Result<Vec<String>, String> {
    let mut resources = bundle_resources.to_vec();
    for route in &app.assets {
        let (source, directory) = match (&route.file, &route.directory) {
            (Some(file), None) => (file, false),
            (None, Some(directory)) => (directory, true),
            _ => return Err("app.assets: specify exactly one of file/directory".to_owned()),
        };
        let relative = safe_relative_path(source, "app.assets source")?;
        let path = project_dir.join(&relative);
        if directory {
            canonical_directory(&path)?;
        } else {
            canonical_file(&path)?;
        }
        let source = portable_relative_path(&relative)?;
        if !resources.iter().any(|entry| entry == &source) {
            resources.push(source);
        }
    }
    // A directory already includes its children. Avoid duplicate copies.
    let all = resources.clone();
    resources.retain(|entry| {
        !all.iter()
            .any(|parent| parent != entry && entry.starts_with(&format!("{parent}/")))
    });
    Ok(resources)
}

pub(super) fn copy_resources(
    project_dir: &Path,
    asset_root: &Path,
    resources: &[String],
    package_dir: &Path,
) -> Result<(), String> {
    let project_dir =
        fs::canonicalize(project_dir).map_err(io_error("resolve project directory"))?;
    for resource in resources {
        let relative = safe_relative_path(resource, "bundle.resources entry")?;
        if relative == Path::new("app.icns")
            || relative == Path::new("app.ico")
            || relative == Path::new(RENDERER_RUNTIME_FILE)
            || relative == Path::new(FRAMEWORK_SHELL_API_FILE)
        {
            return Err(format!(
                "bundle.resources entry is reserved by Tuffite: {resource}"
            ));
        }
        let unresolved = project_dir.join(&relative);
        reject_path_symlinks(&project_dir, &relative)?;
        let source = fs::canonicalize(&unresolved)
            .map_err(|error| format!("resolve resource {}: {error}", unresolved.display()))?;
        if !source.starts_with(&project_dir) {
            return Err(format!("resource escaped project directory: {resource}"));
        }
        let target = package_dir.join(asset_root).join(&relative);
        if source.is_file() {
            copy_regular_file(&source, &target)?;
        } else if source.is_dir() {
            copy_directory(&source, &target)?;
        } else {
            return Err(format!(
                "bundle resource is neither a regular file nor a directory: {resource}"
            ));
        }
    }
    Ok(())
}

pub(super) fn reject_path_symlinks(root: &Path, relative: &Path) -> Result<(), String> {
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(part) = component else {
            return Err(format!("unsafe relative path: {}", relative.display()));
        };
        current.push(part);
        reject_symlink(&current)?;
    }
    Ok(())
}

pub(super) fn copy_regular_file(source: &Path, target: &Path) -> Result<(), String> {
    reject_symlink(source)?;
    if !source.is_file() {
        return Err(format!("expected a regular file: {}", source.display()));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "create package subdirectory {} for {}: {error}",
                parent.display(),
                target.display()
            )
        })?;
    }
    fs::copy(source, target)
        .map(|_| ())
        .map_err(|error| format!("copy {} to {}: {error}", source.display(), target.display()))
}

pub(super) fn copy_directory(source: &Path, target: &Path) -> Result<(), String> {
    let relative_files = collect_files(source)?;
    for relative in relative_files {
        copy_regular_file(&source.join(&relative), &target.join(relative))?;
    }
    Ok(())
}

pub(super) fn validate_internal_asset_root(
    asset_root: &Path,
    executable_name: &str,
    runtime_library: &str,
) -> Result<(), String> {
    let root = asset_root
        .components()
        .next()
        .and_then(|component| match component {
            Component::Normal(part) => part.to_str(),
            _ => None,
        })
        .ok_or_else(|| {
            "internal asset root must start with a valid UTF-8 directory name".to_owned()
        })?;
    let launcher = if cfg!(target_os = "windows") {
        "run-tuffite.cmd"
    } else {
        "run-tuffite.sh"
    };
    for reserved in [
        executable_name,
        runtime_library,
        "tuffite.json",
        MANIFEST_FILE,
        "README.txt",
        launcher,
    ] {
        if root == reserved {
            return Err(format!(
                "internal asset root top-level directory `{root}` conflicts with package-root file `{reserved}`"
            ));
        }
    }
    Ok(())
}
