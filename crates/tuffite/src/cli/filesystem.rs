//! Shared filesystem validation and portable paths for CLI workflows.
use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

pub(super) fn absolute_path(path: PathBuf) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path)
    } else {
        env::current_dir()
            .map(|current| current.join(path))
            .map_err(io_error("read current directory"))
    }
}

pub(super) fn canonical_file(path: &Path) -> Result<PathBuf, String> {
    reject_symlink(path)?;
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("resolve file {}: {error}", path.display()))?;
    if !canonical.is_file() {
        return Err(format!("expected a file: {}", canonical.display()));
    }
    Ok(canonical)
}

pub(super) fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("resolve directory {}: {error}", path.display()))?;
    if !canonical.is_dir() {
        return Err(format!("expected a directory: {}", canonical.display()));
    }
    Ok(canonical)
}

pub(super) fn safe_relative_path(value: &str, field: &str) -> Result<PathBuf, String> {
    if value.is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!("{field} must be relative: {value}"));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => normalized.push(part),
            _ => return Err(format!("{field} contains an unsafe path: {value}")),
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(format!("{field} must not be empty"));
    }
    Ok(normalized)
}

pub(super) fn portable_relative_path(path: &Path) -> Result<String, String> {
    path.components()
        .map(|component| match component {
            Component::Normal(part) => part
                .to_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| "asset paths must be valid UTF-8".to_owned()),
            _ => Err(format!("unsafe asset path: {}", path.display())),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))
}

pub(super) fn reject_symlink(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "symbolic links are not allowed: {}",
            path.display()
        ));
    }
    Ok(())
}

pub(super) fn validate_relative_components(path: &Path, field: &str) -> Result<(), String> {
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "{field} is not a safe relative path: {}",
            path.display()
        ));
    }
    Ok(())
}

pub(super) fn portable_path(path: &Path) -> Result<String, String> {
    validate_relative_components(path, "package path")?;
    path.components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("package path is not valid UTF-8: {}", path.display())),
            _ => Err(format!("unsafe package path: {}", path.display())),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|parts| parts.join("/"))
}

pub(super) fn io_error(context: &'static str) -> impl FnOnce(io::Error) -> String {
    move |error| format!("{context}: {error}")
}
