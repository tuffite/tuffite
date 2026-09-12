//! CLI-owned, versioned NSIS toolchain; application users do not need Mise.
use super::super::filesystem::{canonical_directory, io_error, reject_symlink};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Cursor, Read};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

const VERSION: &str = "3.11";
const URL: &str = "https://downloads.sourceforge.net/project/nsis/NSIS%203/3.11/nsis-3.11.zip";
const SHA256: &str = "c7d27f780ddb6cffb4730138cd1591e841f4b7edb155856901cdf5f214394fa1";
const ARCHIVE_LIMIT: u64 = 8 * 1024 * 1024;
const EXTRACT_LIMIT: u64 = 64 * 1024 * 1024;
const REQUIRED: &[&str] = &[
    "makensis.exe",
    "Bin/makensis.exe",
    "Stubs/lzma_solid-x86-unicode",
    "Include/LogicLib.nsh",
    "Include/MUI2.nsh",
    "Plugins/x86-unicode/System.dll",
];

pub(super) struct Nsis {
    directory: PathBuf,
}

impl Nsis {
    pub(super) fn resolve() -> Result<Self, String> {
        if let Some(directory) = env::var_os("TUFFITE_NSIS_DIR") {
            return Self::open(Path::new(&directory)).map_err(|error| {
                format!("invalid TUFFITE_NSIS_DIR: {error}; provide the complete NSIS directory")
            });
        }
        let cache = if let Some(path) = env::var_os("TUFFITE_TOOLS_CACHE") {
            PathBuf::from(path)
        } else if cfg!(windows) {
            PathBuf::from(
                env::var_os("LOCALAPPDATA")
                    .ok_or("LOCALAPPDATA is unavailable; set TUFFITE_TOOLS_CACHE")?,
            )
            .join("Tuffite/tools")
        } else {
            return Err("automatic NSIS provisioning requires a Windows host".into());
        };
        let offline = match env::var("TUFFITE_TOOLS_OFFLINE") {
            Err(env::VarError::NotPresent) => false,
            Ok(value) if matches!(value.as_str(), "1" | "true") => true,
            Ok(value) if matches!(value.as_str(), "0" | "false") => false,
            _ => return Err("TUFFITE_TOOLS_OFFLINE must be true/false or 1/0".into()),
        };
        Self::cached(&cache, offline, Self::download)
    }

    fn open(directory: &Path) -> Result<Self, String> {
        let directory = canonical_directory(directory)?;
        for name in REQUIRED {
            let path = directory.join(name);
            reject_symlink(&path)?;
            if !path.is_file() {
                return Err(format!("NSIS toolchain is missing {name}"));
            }
        }
        Ok(Self { directory })
    }

    pub(super) fn command(&self) -> Command {
        let mut command = Command::new(self.directory.join("makensis.exe"));
        // Prevent an inherited installation path from mixing other stubs/plugins.
        command.env("NSISDIR", &self.directory);
        command
    }

    fn download() -> Result<Vec<u8>, String> {
        eprintln!("Downloading NSIS {VERSION} (first installer build)...");
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .timeout_global(Some(std::time::Duration::from_secs(120)))
            .build()
            .into();
        let mut response = agent.get(URL).call().map_err(|error| {
            format!("download NSIS: {error}; use TUFFITE_NSIS_DIR for an offline installation")
        })?;
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(ARCHIVE_LIMIT + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error("download NSIS archive"))?;
        if bytes.len() as u64 > ARCHIVE_LIMIT {
            return Err("NSIS archive exceeds the download limit".into());
        }
        Ok(bytes)
    }

    fn cached(
        cache: &Path,
        offline: bool,
        download: impl FnOnce() -> Result<Vec<u8>, String>,
    ) -> Result<Self, String> {
        fs::create_dir_all(cache).map_err(io_error("create tools cache"))?;
        let lock_path = cache.join(format!("nsis-{VERSION}.lock"));
        if lock_path.exists() {
            reject_symlink(&lock_path)?;
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .map_err(io_error("open NSIS cache lock"))?;
        lock.lock().map_err(io_error("lock NSIS cache"))?;
        let directory = cache.join(format!("nsis-{VERSION}"));
        if directory.exists() {
            reject_symlink(&directory)?;
        }
        if let Ok(tool) = Self::open(&directory) {
            return Ok(tool);
        }

        let archive_path = cache.join(format!("nsis-{VERSION}.zip"));
        let bytes = if archive_path.exists() {
            reject_symlink(&archive_path)?;
            let bytes = fs::read(&archive_path).map_err(io_error("read cached NSIS archive"))?;
            if Self::verify(&bytes).is_ok() {
                bytes
            } else if offline {
                return Err("cached NSIS archive failed SHA-256 verification; replace it or unset TUFFITE_TOOLS_OFFLINE".into());
            } else {
                download()?
            }
        } else if offline {
            return Err(format!(
                "NSIS {VERSION} is not cached; unset TUFFITE_TOOLS_OFFLINE for the first installer build, or set TUFFITE_NSIS_DIR"
            ));
        } else {
            download()?
        };
        Self::verify(&bytes)?;
        // The lock serializes cache repair. A interrupted archive write is rejected
        // on the next invocation, and the directory is published only when complete.
        fs::write(&archive_path, &bytes).map_err(io_error("cache NSIS archive"))?;
        let staging = cache.join(format!("nsis-{VERSION}.staging"));
        if staging.exists() {
            reject_symlink(&staging)?;
            fs::remove_dir_all(&staging).map_err(io_error("remove interrupted NSIS extraction"))?;
        }
        let result = (|| {
            Self::extract(&bytes, &staging)?;
            Self::open(&staging)?;
            if directory.exists() {
                fs::remove_dir_all(&directory).map_err(io_error("remove incomplete NSIS cache"))?;
            }
            fs::rename(&staging, &directory).map_err(io_error("publish NSIS toolchain"))?;
            Self::open(&directory)
        })();
        if result.is_err() {
            let _ = fs::remove_dir_all(staging);
        }
        result
    }

    fn verify(bytes: &[u8]) -> Result<(), String> {
        if format!("{:x}", Sha256::digest(bytes)) != SHA256 {
            return Err("NSIS archive failed SHA-256 verification; no tools were installed".into());
        }
        Ok(())
    }

    fn extract(bytes: &[u8], destination: &Path) -> Result<(), String> {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| format!("read NSIS ZIP: {error}"))?;
        fs::create_dir(destination).map_err(io_error("create NSIS staging"))?;
        let mut total = 0_u64;
        for index in 0..archive.len() {
            let mut file = archive
                .by_index(index)
                .map_err(|error| format!("read NSIS ZIP entry: {error}"))?;
            let name = file.name();
            if name.contains('\\') || name.contains(':') {
                return Err("unsafe NSIS archive path".into());
            }
            let path = file.enclosed_name().ok_or("unsafe NSIS archive path")?;
            let mut components = path.components();
            if components.next() != Some(Component::Normal(std::ffi::OsStr::new("nsis-3.11"))) {
                return Err("unexpected NSIS archive root".into());
            }
            let relative = components.as_path();
            if relative
                .components()
                .any(|component| !matches!(component, Component::Normal(_)))
                || file
                    .unix_mode()
                    .is_some_and(|mode| mode & 0o170_000 == 0o120_000)
            {
                return Err("unsafe NSIS archive entry".into());
            }
            let output = destination.join(relative);
            if file.is_dir() {
                fs::create_dir_all(output).map_err(io_error("create NSIS directory"))?;
                continue;
            }
            total = total
                .checked_add(file.size())
                .filter(|total| *total <= EXTRACT_LIMIT)
                .ok_or("NSIS archive exceeds extraction limit")?;
            fs::create_dir_all(output.parent().ok_or("NSIS entry has no parent")?)
                .map_err(io_error("create NSIS directory"))?;
            let mut writer = File::create(output).map_err(io_error("extract NSIS file"))?;
            let written = io::copy(&mut (&mut file).take(EXTRACT_LIMIT + 1), &mut writer)
                .map_err(io_error("extract NSIS file"))?;
            if written != file.size() {
                return Err("invalid NSIS archive entry size".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn archive(name: &str) -> Vec<u8> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"fixture").unwrap();
        zip.finish().unwrap().into_inner()
    }

    #[test]
    fn archives_reject_wrong_hash_and_unsafe_paths() {
        let temporary = env::temp_dir().join(format!("tuffite-nsis-paths-{}", std::process::id()));
        for name in [
            "nsis-3.11/../../outside",
            "nsis-3.11/C:/outside",
            "nsis-3.11\\outside",
            "other/makensis.exe",
        ] {
            let _ = fs::remove_dir_all(&temporary);
            assert!(Nsis::extract(&archive(name), &temporary).is_err());
        }
        let _ = fs::remove_dir_all(temporary);
        assert!(Nsis::verify(b"unexpected archive").is_err());
    }

    #[test]
    #[ignore = "requires TUFFITE_TEST_NSIS_ARCHIVE containing the pinned official ZIP"]
    fn verified_archive_installs_and_repairs_cache_offline() {
        let bytes =
            fs::read(env::var_os("TUFFITE_TEST_NSIS_ARCHIVE").expect("archive fixture")).unwrap();
        Nsis::verify(&bytes).unwrap();
        let cache = env::temp_dir().join(format!("tuffite-nsis-real-{}", std::process::id()));
        let _ = fs::remove_dir_all(&cache);
        let first = Nsis::cached(&cache, false, || Ok(bytes)).unwrap();
        assert!(
            first
                .directory
                .join("makensis.exe")
                .metadata()
                .unwrap()
                .len()
                > 0
        );
        fs::remove_file(first.directory.join(REQUIRED[2])).unwrap();
        let repaired = Nsis::cached(&cache, true, || {
            panic!("repair must reuse the verified ZIP")
        })
        .unwrap();
        assert!(repaired.directory.join(REQUIRED[2]).is_file());
        fs::remove_dir_all(cache).unwrap();
    }

    #[test]
    fn complete_cache_is_reused_offline_and_partial_cache_fails_without_download() {
        let cache = env::temp_dir().join(format!("tuffite-nsis-cache-{}", std::process::id()));
        let _ = fs::remove_dir_all(&cache);
        let directory = cache.join(format!("nsis-{VERSION}"));
        for name in REQUIRED {
            let path = directory.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, "fixture").unwrap();
        }
        let tool = Nsis::cached(&cache, true, || panic!("cache reuse must not download")).unwrap();
        assert_eq!(tool.directory, fs::canonicalize(&directory).unwrap());
        fs::remove_file(directory.join(REQUIRED[0])).unwrap();
        let error = Nsis::cached(&cache, true, || panic!("offline must not download"))
            .err()
            .unwrap();
        assert!(error.contains("not cached"));
        fs::remove_dir_all(cache).unwrap();
    }
}
