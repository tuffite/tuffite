//! Publication operations for application packaging.
use super::super::filesystem::{io_error, reject_symlink};
use super::archive::{hash_file, package_totals};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

pub(super) fn publish_development(staging: &Path, package: &Path) -> Result<(), String> {
    let parent = package
        .parent()
        .ok_or_else(|| "development package has no parent directory".to_owned())?;
    let name = package
        .file_name()
        .and_then(OsStr::to_str)
        .ok_or_else(|| "development package name must be valid UTF-8".to_owned())?;
    let backup = parent.join(format!(".{name}.{}.backup", std::process::id()));
    if backup.exists() {
        reject_symlink(&backup)?;
        remove_dir_all_with_retry(&backup)?;
    }
    if package.exists() {
        reject_symlink(package)?;
        if !package.is_dir() {
            return Err(format!(
                "development package output is not a directory: {}",
                package.display()
            ));
        }
        fs::rename(package, &backup).map_err(|error| {
            format!(
                "replace development package {}: {error}; stop the running application and retry",
                package.display()
            )
        })?;
    }
    if let Err(error) = fs::rename(staging, package) {
        if backup.exists() {
            let _ = fs::rename(&backup, package);
        }
        return Err(format!("publish development package directory: {error}"));
    }
    if backup.exists() {
        remove_dir_all_with_retry(&backup)?;
    }
    Ok(())
}

pub(super) fn finish_package(
    package_dir: &Path,
    archive_path: Option<&Path>,
) -> Result<(), String> {
    let archive_hash = archive_path
        .map(hash_file)
        .transpose()?
        .map(|(_, hash)| hash);
    let (file_count, bytes) = package_totals(package_dir)?;
    tracing::info!(package_directory = %package_dir.display(), "package directory created");
    if let (Some(archive_path), Some(archive_hash)) = (archive_path, archive_hash) {
        tracing::info!(archive = %archive_path.display(), sha256 = %archive_hash, "package archive created");
    }
    tracing::info!(
        files = file_count,
        bytes,
        "Tuffite application package created"
    );
    Ok(())
}

pub(super) fn validate_output(
    package_dir: &Path,
    archive: Option<&Path>,
    force: bool,
) -> Result<(), String> {
    if !package_dir.exists() && archive.is_none_or(|path| !path.exists()) {
        return Ok(());
    }
    if !force {
        return Err(format!(
            "package output already exists; pass --force to replace: {}",
            package_dir.display()
        ));
    }
    if package_dir.exists() {
        reject_symlink(package_dir)?;
        if !package_dir.is_dir() {
            return Err(format!(
                "package output is not a directory: {}",
                package_dir.display()
            ));
        }
    }
    if let Some(archive) = archive
        && archive.exists()
    {
        reject_symlink(archive)?;
        if !archive.is_file() {
            return Err(format!(
                "archive output is not a file: {}",
                archive.display()
            ));
        }
    }
    Ok(())
}

pub(super) fn publish_release(
    staging_package: &Path,
    staging_archive: &Path,
    package: &Path,
    archive: &Path,
) -> Result<(), String> {
    let package_backup = package.with_extension(format!("backup.{}", std::process::id()));
    let archive_backup = archive.with_extension(format!("zip.backup.{}", std::process::id()));
    if package_backup.exists() {
        reject_symlink(&package_backup)?;
        remove_dir_all_with_retry(&package_backup)?;
    }
    if archive_backup.exists() {
        reject_symlink(&archive_backup)?;
        fs::remove_file(&archive_backup).map_err(io_error("remove stale archive backup"))?;
    }

    let had_package = package.exists();
    let had_archive = archive.exists();
    if had_package {
        fs::rename(package, &package_backup).map_err(io_error("stage old package backup"))?;
    }
    if had_archive && let Err(error) = fs::rename(archive, &archive_backup) {
        if had_package {
            let _ = fs::rename(&package_backup, package);
        }
        return Err(format!("stage old archive backup: {error}"));
    }
    if let Err(error) = fs::rename(staging_package, package) {
        if had_archive {
            let _ = fs::rename(&archive_backup, archive);
        }
        if had_package {
            let _ = fs::rename(&package_backup, package);
        }
        return Err(format!("publish package directory: {error}"));
    }
    if let Err(error) = fs::rename(staging_archive, archive) {
        let _ = fs::rename(package, staging_package);
        if had_archive {
            let _ = fs::rename(&archive_backup, archive);
        }
        if had_package {
            let _ = fs::rename(&package_backup, package);
        }
        return Err(format!("publish package archive: {error}"));
    }

    if had_package {
        remove_dir_all_with_retry(&package_backup)?;
    }
    if had_archive {
        fs::remove_file(&archive_backup).map_err(io_error("remove old archive backup"))?;
    }
    Ok(())
}

pub(super) fn remove_dir_all_with_retry(path: &Path) -> Result<(), String> {
    const ATTEMPTS: usize = 10;
    let mut last_error = None;
    for attempt in 0..ATTEMPTS {
        match fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(error) => {
                last_error = Some(error);
                if attempt + 1 < ATTEMPTS {
                    thread::sleep(Duration::from_millis(100 * (attempt as u64 + 1)));
                }
            }
        }
    }
    let error = last_error.expect("at least one removal attempt");
    Err(format!(
        "remove old package directory {}: {error}; ensure no packaged application or file indexer is using it",
        path.display()
    ))
}
