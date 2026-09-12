//! Archive operations for application packaging.
use super::super::filesystem::{io_error, portable_path};
use super::MANIFEST_FILE;
use super::cargo::PackageTarget;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Serialize)]
pub(super) struct PackageManifest {
    pub(super) schema_version: u32,
    pub(super) framework: &'static str,
    pub(super) framework_version: String,
    pub(super) framework_target: String,
    pub(super) framework_profile: String,
    pub(super) framework_variant: String,
    pub(super) ffi_abi_major: u16,
    pub(super) ffi_abi_minor: u16,
    pub(super) ffi_abi_patch: u16,
    pub(super) application: String,
    pub(super) platform: &'static str,
    pub(super) architecture: &'static str,
    pub(super) executable: String,
    pub(super) executable_integrity: &'static str,
    pub(super) runtime_library: String,
    pub(super) development_enabled: bool,
    pub(super) files: Vec<PackagedFile>,
}

#[derive(Debug, Serialize)]
pub(super) struct PackagedFile {
    path: String,
    size: u64,
    sha256: String,
}

pub(super) fn write_launcher(package_dir: &Path, executable: &OsStr) -> Result<(), String> {
    let executable = executable
        .to_str()
        .ok_or_else(|| "application executable name must be valid UTF-8".to_owned())?;
    if cfg!(target_os = "windows") {
        let contents = format!(
            r#"@echo off
setlocal
set "TUFFITE_PACKAGE_DIR=%~dp0."
if not exist "%TUFFITE_PACKAGE_DIR%\.appcontainer-acl-ready" (
  icacls "%TUFFITE_PACKAGE_DIR%" /grant "*S-1-15-2-1:(OI)(CI)(RX)" "*S-1-15-2-2:(OI)(CI)(RX)" /T /C /Q >nul
  if errorlevel 1 exit /b 1
  type nul > "%TUFFITE_PACKAGE_DIR%\.appcontainer-acl-ready"
)
pushd "%TUFFITE_PACKAGE_DIR%"
start "" "%TUFFITE_PACKAGE_DIR%\{executable}" %*
set "TUFFITE_LAUNCH_STATUS=%ERRORLEVEL%"
popd
exit /b %TUFFITE_LAUNCH_STATUS%
"#
        )
        .replace('\n', "\r\n");
        fs::write(package_dir.join("run-tuffite.cmd"), contents)
            .map_err(io_error("write Windows launcher"))?;
    } else {
        let escaped = shell_single_quote(executable);
        let contents = format!(
            "#!/bin/sh\nset -eu\npackage_dir=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd)\ncd \"$package_dir\"\nexec \"$package_dir\"/{escaped} \"$@\"\n"
        );
        let launcher = package_dir.join("run-tuffite.sh");
        fs::write(&launcher, contents).map_err(io_error("write Unix launcher"))?;
        #[cfg(unix)]
        set_executable(&launcher)?;
    }
    Ok(())
}

pub(super) fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(unix)]
pub(super) fn set_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = fs::metadata(path)
        .map_err(io_error("read launcher permissions"))?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).map_err(io_error("set launcher permissions"))
}

pub(super) fn write_readme(
    package_dir: &Path,
    executable: &OsStr,
    bundle: bool,
) -> Result<(), String> {
    let launcher = if bundle {
        "the .app bundle in Finder"
    } else if cfg!(target_os = "windows") {
        "run-tuffite.cmd"
    } else {
        "./run-tuffite.sh"
    };
    let contents = format!(
        "Tuffite application package\n===========================\n\nOpen {launcher} to start {}.\n\nThe package contains the application-named Chromium executable, Rust business runtime, Framework runtime, application manifest, and renderer assets.\n",
        executable.to_string_lossy()
    );
    fs::write(package_dir.join("README.txt"), contents).map_err(io_error("write package README"))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_package_manifest(
    package_dir: &Path,
    application: &str,
    executable: String,
    runtime_library: String,
    development_enabled: bool,
    package_target: &PackageTarget<'_>,
    framework: &crate::build::framework::Manifest,
    signed_bundle: bool,
) -> Result<(), String> {
    let mut paths = collect_files(package_dir)?;
    // Bundle signing changes the main image and seals manifest.json itself.
    // Avoid a circular digest: codesign owns integrity of that image.
    paths.retain(|path| {
        path != Path::new(MANIFEST_FILE) && !(signed_bundle && path == Path::new(&executable))
    });
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let source = package_dir.join(&path);
        let (size, sha256) = hash_file(&source)?;
        files.push(PackagedFile {
            path: portable_path(&path)?,
            size,
            sha256,
        });
    }
    let manifest = PackageManifest {
        schema_version: 2,
        framework: "tuffite",
        framework_version: framework.framework_version.clone(),
        framework_target: framework.target.clone(),
        framework_profile: framework.profile.clone(),
        framework_variant: framework.variant.clone(),
        ffi_abi_major: framework.ffi_abi_major,
        ffi_abi_minor: framework.ffi_abi_minor,
        ffi_abi_patch: framework.ffi_abi_patch,
        application: application.to_owned(),
        platform: package_target.platform,
        architecture: package_target.architecture,
        executable,
        executable_integrity: if signed_bundle { "codesign" } else { "sha256" },
        runtime_library,
        development_enabled,
        files,
    };
    let target = File::create(package_dir.join(MANIFEST_FILE))
        .map_err(io_error("create package manifest"))?;
    serde_json::to_writer_pretty(BufWriter::new(target), &manifest)
        .map_err(|error| format!("write package manifest: {error}"))
}

pub(super) fn collect_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        let entries = fs::read_dir(&directory)
            .map_err(|error| format!("read package directory {}: {error}", directory.display()))?;
        for entry in entries {
            let entry = entry.map_err(io_error("read package entry"))?;
            let file_type = entry
                .file_type()
                .map_err(io_error("read package entry type"))?;
            if file_type.is_symlink() {
                return Err(format!(
                    "package contains a symbolic link: {}",
                    entry.path().display()
                ));
            }
            if file_type.is_dir() {
                directories.push(entry.path());
            } else if file_type.is_file() {
                result.push(
                    entry
                        .path()
                        .strip_prefix(root)
                        .map_err(|_| "package file escaped package root".to_owned())?
                        .to_path_buf(),
                );
            }
        }
    }
    result.sort_by_key(|path| portable_path(path).unwrap_or_default());
    Ok(result)
}

pub(super) fn hash_file(path: &Path) -> Result<(u64, String), String> {
    let mut reader = BufReader::new(
        File::open(path)
            .map_err(|error| format!("open {} for hashing: {error}", path.display()))?,
    );
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 128 * 1024];
    let mut size = 0_u64;
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("hash {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        size += u64::try_from(read).map_err(|_| "file read size overflow".to_owned())?;
    }
    Ok((size, format!("{:x}", hasher.finalize())))
}

pub(super) fn write_zip(
    package_dir: &Path,
    archive: &Path,
    package_name: &str,
) -> Result<(), String> {
    let file = File::create(archive).map_err(io_error("create package archive"))?;
    let mut archive_writer = ZipWriter::new(BufWriter::new(file));
    let mut buffer = vec![0_u8; 128 * 1024];
    for relative in collect_files(package_dir)? {
        let portable = portable_path(&relative)?;
        let entry_name = format!("{package_name}/{portable}");
        let source_path = package_dir.join(&relative);
        let options = zip_options(&source_path)?;
        archive_writer
            .start_file(entry_name, options)
            .map_err(|error| format!("start ZIP entry {portable}: {error}"))?;
        let mut source = BufReader::new(
            File::open(source_path)
                .map_err(|error| format!("open ZIP input {portable}: {error}"))?,
        );
        loop {
            let read = source
                .read(&mut buffer)
                .map_err(|error| format!("read ZIP input {portable}: {error}"))?;
            if read == 0 {
                break;
            }
            archive_writer
                .write_all(&buffer[..read])
                .map_err(|error| format!("write ZIP entry {portable}: {error}"))?;
        }
    }
    archive_writer
        .finish()
        .map_err(|error| format!("finish package archive: {error}"))?;
    Ok(())
}

pub(super) fn zip_options(path: &Path) -> Result<SimpleFileOptions, String> {
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(1));
    let metadata = fs::metadata(path).map_err(io_error("read ZIP input metadata"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        Ok(options.unix_permissions(mode))
    }
    #[cfg(not(unix))]
    {
        let _ = metadata;
        Ok(options)
    }
}

pub(super) fn package_totals(package_dir: &Path) -> Result<(usize, u64), String> {
    let files = collect_files(package_dir)?;
    let mut bytes = 0_u64;
    for relative in &files {
        bytes = bytes
            .checked_add(
                fs::metadata(package_dir.join(relative))
                    .map_err(io_error("read packaged file metadata"))?
                    .len(),
            )
            .ok_or_else(|| "package size overflow".to_owned())?;
    }
    Ok((files.len(), bytes))
}

pub(super) fn write_packaged_manifest(
    path: &Path,
    manifest: &serde_json::Value,
) -> Result<(), String> {
    let mut bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|error| format!("serialize packaged manifest: {error}"))?;
    bytes.push(b'\n');
    fs::write(path, bytes).map_err(io_error("write packaged application manifest"))
}
