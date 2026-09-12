//! Strip distribution copies while retaining private matching application symbols.
use super::super::filesystem::{absolute_path, io_error, reject_symlink};
use super::archive::{collect_files, hash_file};
use super::cargo::PackageTarget;
use super::formats::run;
use super::publication::{publish_development, validate_output};
use super::resources::copy_regular_file;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) struct PreparedSymbols {
    staging: PathBuf,
    destination: PathBuf,
}
impl PreparedSymbols {
    pub(super) fn publish(self) -> Result<(), String> {
        if self.destination.exists() {
            reject_symlink(&self.destination)?;
        }
        publish_development(&self.staging, &self.destination)?;
        tracing::info!(symbols = %self.destination.display(), "application symbols exported");
        Ok(())
    }
}
impl Drop for PreparedSymbols {
    fn drop(&mut self) {
        if self.staging.exists() {
            let _ = fs::remove_dir_all(&self.staging);
        }
    }
}

pub(super) fn native_image(path: &Path) -> Result<bool, String> {
    let mut magic = [0u8; 4];
    let mut file = File::open(path).map_err(io_error("read native image"))?;
    match file.read_exact(&mut magic) {
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(false),
        Err(e) => return Err(format!("read native image: {e}")),
        _ => (),
    }
    Ok(matches!(
        magic,
        [0xcf | 0xce, 0xfa, 0xed, 0xfe]
            | [0xfe, 0xed, 0xfa, 0xcf | 0xce]
            | [0xca, 0xfe, 0xba, 0xbe]
            | [0xbe, 0xba, 0xfe, 0xca]
            | [0x7f, b'E', b'L', b'F']
    ) || &magic[..2] == b"MZ")
}
/// Settings for processing staged runtime copies and their private symbols.
pub(super) struct SymbolProcessor<'a, 'target> {
    pub(super) root: &'a Path,
    pub(super) application_library: &'a Path,
    pub(super) strip: bool,
    pub(super) destination: Option<&'a Path>,
    pub(super) force: bool,
    pub(super) identifier: &'a str,
    pub(super) target: &'a PackageTarget<'target>,
}
impl SymbolProcessor<'_, '_> {
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare(self) -> Result<Option<PreparedSymbols>, String> {
        let Self {
            root,
            application_library,
            strip,
            destination,
            force,
            identifier,
            target,
        } = self;
        let mut symbol_record = None;
        let mut staging = None;
        let destination = destination
            .map(|p| absolute_path(p.to_path_buf()))
            .transpose()?;
        if let Some(destination) = &destination {
            let parent = destination
                .parent()
                .ok_or("symbols directory has no parent")?;
            fs::create_dir_all(parent).map_err(io_error("create symbols parent"))?;
            let destination = fs::canonicalize(parent)
                .map_err(io_error("resolve symbols parent"))?
                .join(destination.file_name().ok_or("invalid symbols directory")?);
            if root.starts_with(&destination)
                || destination.starts_with(root)
                || application_library.starts_with(&destination)
            {
                return Err("symbols directory overlaps runtime or build outputs".into());
            }
            validate_output(&destination, None, force)?;
            let temporary =
                destination.with_extension(format!("symbols-staging.{}", std::process::id()));
            if temporary.exists() {
                return Err("symbols staging directory already exists".into());
            }
            fs::create_dir(&temporary).map_err(io_error("create symbols staging"))?;
            staging = Some((temporary, destination));
        }
        let result: Result<(), String> = (|| {
            if let Some((temporary, _)) = &staging {
                let binary = application_library
                    .file_name()
                    .ok_or("application library filename is missing")?;
                let original = temporary.join(binary);
                copy_regular_file(application_library, &original)?;
                let (_, sha) = hash_file(application_library)?;
                let mut record =
                    serde_json::json!({"binary": binary.to_string_lossy(), "original_sha256": sha});
                if cfg!(target_os = "macos") {
                    let dsym =
                        original.with_file_name(format!("{}.dSYM", binary.to_string_lossy()));
                    run(Command::new("xcrun")
                        .arg("dsymutil")
                        .arg(application_library)
                        .arg("-o")
                        .arg(dsym))?;
                    let uuid = Command::new("xcrun")
                        .args(["dwarfdump", "--uuid"])
                        .arg(application_library)
                        .output()
                        .map_err(io_error("read image UUID"))?;
                    if !uuid.status.success() {
                        return Err("dwarfdump failed".into());
                    }
                    record["uuids"] = serde_json::json!(
                        String::from_utf8_lossy(&uuid.stdout)
                            .lines()
                            .filter_map(|l| l
                                .strip_prefix("UUID:")
                                .and_then(|s| s.split_whitespace().next()))
                            .collect::<Vec<_>>()
                    );
                } else if cfg!(target_os = "windows") {
                    let pdb = application_library.with_extension("pdb");
                    if pdb.is_file() {
                        copy_regular_file(&pdb, &temporary.join(pdb.file_name().unwrap()))?;
                    }
                }
                symbol_record = Some(record);
            }
            let (mut before, mut after) = (0u64, 0u64);
            for relative in collect_files(root)? {
                let path = root.join(relative);
                if !native_image(&path)? {
                    continue;
                }
                before += path.metadata().map_err(io_error("read image size"))?.len();
                if strip {
                    if cfg!(target_os = "macos") {
                        run(Command::new("xcrun").args(["strip", "-S", "-x"]).arg(&path))?;
                    } else if cfg!(target_os = "linux") {
                        run(Command::new("strip")
                            .args(["--strip-debug", "--strip-unneeded"])
                            .arg(&path))?;
                    }
                    // MSVC PDBs are external; the runtime image and exports are retained.
                }
                if cfg!(target_os = "macos") {
                    run(Command::new("codesign")
                        .args(["--force", "--sign", "-"])
                        .arg(&path))?;
                }
                after += path
                    .metadata()
                    .map_err(io_error("read stripped image size"))?
                    .len();
            }
            tracing::info!(
                before,
                after,
                saved = before.saturating_sub(after),
                "native package bytes"
            );
            if let Some((temporary, _)) = &staging {
                let runtime = root.join(
                    application_library
                        .file_name()
                        .ok_or("missing runtime filename")?,
                );
                let (_, runtime_hash) = hash_file(&runtime)?;
                let mut record = symbol_record
                    .take()
                    .ok_or("missing application symbol metadata")?;
                record["runtime_sha256"] = runtime_hash.into();
                fs::write(temporary.join("symbols.json"), serde_json::to_vec_pretty(&serde_json::json!({"schema_version":1, "application":identifier,"target":target.triple,"images":[record]})).map_err(|e|e.to_string())?).map_err(io_error("write symbols metadata"))?;
            }
            Ok(())
        })();
        let prepared = staging.map(|(staging, destination)| PreparedSymbols {
            staging,
            destination,
        });
        result?;
        Ok(prepared)
    }
}
