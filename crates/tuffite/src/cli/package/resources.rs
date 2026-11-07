//! Resources operations for application packaging.
use super::super::filesystem::{
    canonical_directory, canonical_file, io_error, reject_symlink, safe_relative_path,
};
use super::{FRAMEWORK_SDK_FILE, MANIFEST_FILE};
use serde::Deserialize;
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PackageApp {
    pub(super) identifier: String,
}

/// Target directory aliases mapped to project-relative sources and exclusions.
pub(super) type BundleResources = std::collections::BTreeMap<String, Vec<String>>;
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Resource {
    pub source: std::path::PathBuf,
    pub target: std::path::PathBuf,
}

pub(super) fn configure_resources(
    project_dir: &Path,
    bundle_resources: &BundleResources,
) -> Result<Vec<Resource>, String> {
    let mut files = std::collections::BTreeMap::new();
    for (alias, entries) in bundle_resources {
        let mount = if alias == "." {
            std::path::PathBuf::new()
        } else {
            safe_relative_path(alias, "bundle.resources alias")?
        };
        let excludes = entries
            .iter()
            .filter_map(|s| s.strip_prefix('!'))
            .map(|pattern| {
                if pattern.is_empty() {
                    return Err("empty resource exclusion".to_owned());
                }
                glob::Pattern::new(pattern).map_err(|e| format!("invalid resource exclusion: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for entry in entries.iter().filter(|s| !s.starts_with('!')) {
            let relative = safe_relative_path(entry, "bundle.resources entry")?;
            if resource_excluded(&excludes, &relative, Path::new("")) {
                continue;
            }
            reject_path_symlinks(project_dir, &relative)?;
            let source = project_dir.join(&relative);
            let candidates = if source.is_dir() {
                canonical_directory(&source)?;
                collect_resource_files(&source, &relative, &excludes)?
            } else {
                canonical_file(&source)?;
                vec![std::path::PathBuf::new()]
            };
            for child in candidates {
                let origin = relative.join(&child);
                let target = if child.as_os_str().is_empty() {
                    mount.join(relative.file_name().ok_or("resource has no file name")?)
                } else {
                    mount.join(&child)
                };
                if resource_excluded(&excludes, &origin, &child) {
                    continue;
                }
                let top = target
                    .components()
                    .next()
                    .ok_or("resource target is empty")?
                    .as_os_str();
                if ["app.icns", "app.ico", FRAMEWORK_SDK_FILE]
                    .iter()
                    .any(|reserved| top.to_string_lossy().eq_ignore_ascii_case(reserved))
                {
                    return Err(format!(
                        "bundle.resources target is reserved by Tuffite: {}",
                        target.display()
                    ));
                }
                if let Some(previous) = files.insert(target.clone(), origin.clone())
                    && previous != origin
                {
                    return Err(format!(
                        "bundle.resources target collision: {}",
                        target.display()
                    ));
                }
            }
        }
    }
    validate_resource_targets(&files)?;
    Ok(files
        .into_iter()
        .map(|(target, source)| Resource { source, target })
        .collect())
}

fn resource_excluded(excludes: &[glob::Pattern], origin: &Path, child: &Path) -> bool {
    let options = glob::MatchOptions {
        require_literal_separator: true,
        ..Default::default()
    };
    excludes.iter().any(|pattern| {
        origin
            .ancestors()
            .chain(child.ancestors())
            .filter(|path| !path.as_os_str().is_empty())
            .any(|path| pattern.matches_path_with(path, options))
    })
}
fn collect_resource_files(
    root: &Path,
    source: &Path,
    excludes: &[glob::Pattern],
) -> Result<Vec<std::path::PathBuf>, String> {
    let mut directories = vec![std::path::PathBuf::new()];
    let mut files = Vec::new();
    while let Some(directory) = directories.pop() {
        for entry in
            fs::read_dir(root.join(&directory)).map_err(io_error("read resource directory"))?
        {
            let entry = entry.map_err(io_error("read resource entry"))?;
            let child = directory.join(entry.file_name());
            if resource_excluded(excludes, &source.join(&child), &child) {
                continue;
            }
            let kind = entry
                .file_type()
                .map_err(io_error("inspect resource entry"))?;
            if kind.is_symlink() {
                return Err(format!(
                    "symbolic links are not allowed: {}",
                    entry.path().display()
                ));
            }
            if kind.is_dir() {
                directories.push(child);
            } else if kind.is_file() {
                files.push(child);
            } else {
                return Err(format!(
                    "resource is not a regular file or directory: {}",
                    entry.path().display()
                ));
            }
        }
    }
    files.sort();
    Ok(files)
}

fn validate_resource_targets(
    files: &std::collections::BTreeMap<std::path::PathBuf, std::path::PathBuf>,
) -> Result<(), String> {
    let mut portable_targets = std::collections::BTreeMap::new();
    for target in files.keys() {
        let key = target
            .to_str()
            .ok_or("resource target must be valid UTF-8")?
            .to_lowercase();
        if let Some(previous) = portable_targets.insert(key.clone(), target)
            && previous != target
        {
            return Err(format!(
                "bundle.resources case-insensitive target collision: {}",
                target.display()
            ));
        }
    }
    for (key, target) in &portable_targets {
        for (index, _) in key.match_indices('/') {
            if portable_targets.contains_key(&key[..index]) {
                return Err(format!(
                    "bundle.resources file/directory collision: {}",
                    target.display()
                ));
            }
        }
    }
    for target in files.keys() {
        for ancestor in target.ancestors().skip(1) {
            if files.contains_key(ancestor) {
                return Err(format!(
                    "bundle.resources file/directory collision: {}",
                    ancestor.display()
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn copy_resources(
    project_dir: &Path,
    asset_root: &Path,
    resources: &[Resource],
    package_dir: &Path,
) -> Result<(), String> {
    let project_dir =
        fs::canonicalize(project_dir).map_err(io_error("resolve project directory"))?;
    for resource in resources {
        reject_path_symlinks(&project_dir, &resource.source)?;
        let source = fs::canonicalize(project_dir.join(&resource.source))
            .map_err(io_error("resolve resource"))?;
        if !source.starts_with(&project_dir) {
            return Err("resource escaped project directory".to_owned());
        }
        copy_regular_file(
            &source,
            &package_dir.join(asset_root).join(&resource.target),
        )?;
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
