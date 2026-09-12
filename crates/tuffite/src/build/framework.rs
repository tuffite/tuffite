//! Tuffite Framework Distribution discovery, validation, and Cargo linking.

use std::collections::BTreeSet;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, BufReader, IsTerminal as _, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use url::Url;
use zip::ZipArchive;

use tuffite_abi::{ABI_MAJOR, ABI_MINOR, ABI_PATCH};

pub const MANIFEST_FILE: &str = "tuffite-framework.json";
const GITHUB_API_VERSION: &str = "2022-11-28";

/// Framework checkout used to discover locally packaged native output.
pub const FRAMEWORK_ENV: &str = "TUFFITE_FRAMEWORK_DIR";
/// Exact developer-managed Distribution; never downloaded into or replaced.
pub const DIR_ENV: &str = "TUFFITE_FRAMEWORK_DIST";
/// Root of Tuffite's versioned Framework cache.
pub const CACHE_ENV: &str = "TUFFITE_FRAMEWORK_DIST_CACHE";
/// Exact destination for an automatically downloaded Distribution.
pub const DOWNLOAD_DIR_ENV: &str = "TUFFITE_FRAMEWORK_DIST_DOWNLOAD_DIR";
/// Explicit Framework Rust target triple.
pub const TARGET_ENV: &str = "TUFFITE_FRAMEWORK_DIST_TARGET";
/// Enables or disables automatic download on a cache miss.
pub const AUTO_DOWNLOAD_ENV: &str = "TUFFITE_FRAMEWORK_DIST_AUTO_DOWNLOAD";
/// Enables or disables transfer progress messages for automatic downloads.
pub const PROGRESS_ENV: &str = "TUFFITE_FRAMEWORK_DIST_PROGRESS";
pub const REPOSITORY_ENV: &str = "TUFFITE_GITHUB_REPOSITORY";
pub const RELEASE_TAG_ENV: &str = "TUFFITE_FRAMEWORK_DIST_RELEASE_TAG";
/// Environment variables that affect Framework resolution in a Cargo build script.
pub const ENV_VARS: &[&str] = &[
    DIR_ENV,
    FRAMEWORK_ENV,
    CACHE_ENV,
    DOWNLOAD_DIR_ENV,
    TARGET_ENV,
    AUTO_DOWNLOAD_ENV,
    PROGRESS_ENV,
    REPOSITORY_ENV,
    RELEASE_TAG_ENV,
];

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub framework_version: String,
    pub chromium_revision: String,
    pub target: String,
    pub profile: String,
    #[serde(default = "default_variant")]
    pub variant: String,
    pub component_build: bool,
    #[serde(default)]
    pub development_enabled: bool,
    pub features: Vec<String>,
    pub ffi_abi_major: u16,
    pub ffi_abi_minor: u16,
    /// Compatible ABI patch release.
    pub ffi_abi_patch: u16,
    pub link_directory: String,
    pub runtime_directory: String,
    pub bootstrap: FileEntry,
    pub runtime_files: Vec<FileEntry>,
}

fn default_variant() -> String {
    "full".to_owned()
}

#[derive(Clone, Debug)]
pub struct Distribution {
    root: PathBuf,
    manifest: Manifest,
}

impl Distribution {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let root = fs::canonicalize(path.as_ref()).map_err(|error| {
            format!(
                "resolve Tuffite Framework distribution {}: {error}",
                path.as_ref().display()
            )
        })?;
        if !root.is_dir() {
            return Err(format!(
                "Tuffite Framework distribution is not a directory: {}",
                root.display()
            ));
        }
        let manifest_path = root.join(MANIFEST_FILE);
        let manifest: Manifest = serde_json::from_reader(BufReader::new(
            File::open(&manifest_path)
                .map_err(|error| format!("open {}: {error}", manifest_path.display()))?,
        ))
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
        let distribution = Self { root, manifest };
        distribution.validate_identity()?;
        distribution.validate_layout()?;
        Ok(distribution)
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    #[must_use]
    pub const fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    pub fn runtime_root(&self) -> Result<PathBuf, String> {
        safe_join(
            &self.root,
            &self.manifest.runtime_directory,
            "runtime_directory",
        )
    }

    pub fn runtime_files(&self, verify_hashes: bool) -> Result<Vec<(PathBuf, PathBuf)>, String> {
        let runtime_root = self.runtime_root()?;
        let mut files = Vec::with_capacity(self.manifest.runtime_files.len());
        for entry in &self.manifest.runtime_files {
            let relative = safe_relative(&entry.path, "runtime file")?;
            let source = runtime_root.join(&relative);
            let metadata = source.metadata().map_err(|error| {
                format!(
                    "inspect Framework runtime file {}: {error}",
                    source.display()
                )
            })?;
            if !metadata.is_file() || metadata.len() != entry.size {
                return Err(format!(
                    "Framework runtime file size mismatch: {}",
                    source.display()
                ));
            }
            if verify_hashes && hash_file(&source)? != entry.sha256 {
                return Err(format!(
                    "Framework runtime file hash mismatch: {}",
                    source.display()
                ));
            }
            files.push((source, relative));
        }
        Ok(files)
    }

    pub fn bootstrap(&self, verify_hash: bool) -> Result<PathBuf, String> {
        let relative = safe_relative(&self.manifest.bootstrap.path, "bootstrap")?;
        let source = self.root.join(relative);
        let metadata = source.metadata().map_err(|error| {
            format!("inspect Framework bootstrap {}: {error}", source.display())
        })?;
        if !metadata.is_file() || metadata.len() != self.manifest.bootstrap.size {
            return Err(format!(
                "Framework bootstrap size mismatch: {}",
                source.display()
            ));
        }
        if verify_hash && hash_file(&source)? != self.manifest.bootstrap.sha256 {
            return Err(format!(
                "Framework bootstrap hash mismatch: {}",
                source.display()
            ));
        }
        Ok(source)
    }

    fn validate_identity(&self) -> Result<(), String> {
        if self.manifest.framework_version != env!("CARGO_PKG_VERSION") {
            return Err(format!(
                "incompatible Tuffite Framework Distribution at {}: the Distribution was built for Tuffite {}, but the current tuffite CLI is {}. Download the matching Release asset or rebuild it with `python3 x.py package --destination \"{}\"`",
                self.root.display(),
                self.manifest.framework_version,
                env!("CARGO_PKG_VERSION"),
                self.root.display(),
            ));
        }
        if !matches!(self.manifest.variant.as_str(), "full" | "desktop") {
            return Err(format!(
                "unsupported Tuffite Framework variant {}",
                self.manifest.variant
            ));
        }
        if !tuffite_abi::is_compatible(
            (
                self.manifest.ffi_abi_major,
                self.manifest.ffi_abi_minor,
                self.manifest.ffi_abi_patch,
            ),
            (ABI_MAJOR, ABI_MINOR, ABI_PATCH),
        ) {
            return Err(format!(
                "Tuffite Framework FFI ABI {}.{}.{} is incompatible with {}.{}.{}",
                self.manifest.ffi_abi_major,
                self.manifest.ffi_abi_minor,
                self.manifest.ffi_abi_patch,
                ABI_MAJOR,
                ABI_MINOR,
                ABI_PATCH
            ));
        }
        Ok(())
    }

    fn validate_target(&self) -> Result<(), String> {
        if let Ok(target) = env::var("TARGET")
            && target != self.manifest.target
        {
            return Err(format!(
                "Tuffite Framework target {} does not match Cargo target {target}",
                self.manifest.target
            ));
        }
        Ok(())
    }

    fn validate_layout(&self) -> Result<(), String> {
        self.validate_target()?;
        let link = safe_join(&self.root, &self.manifest.link_directory, "link_directory")?;
        let runtime = self.runtime_root()?;
        if !link.is_dir() || !runtime.is_dir() {
            return Err(format!(
                "Tuffite Framework distribution is incomplete: {}",
                self.root.display()
            ));
        }
        self.bootstrap(false)?;
        if self.manifest.runtime_files.is_empty() {
            return Err("Tuffite Framework runtime file list is empty".to_owned());
        }
        for path in [
            "licenses/LICENSE.tuffite.txt",
            "licenses/LICENSE.chromium.txt",
            "licenses/THIRD_PARTY_NOTICES.chromium.txt",
        ] {
            if !self
                .manifest
                .runtime_files
                .iter()
                .any(|file| file.path == path && file.size > 0)
            {
                return Err(format!(
                    "Framework Distribution at {} is missing notice {path}; repackage this directory with `python3 x.py package --destination \"{}\"`, or update TUFFITE_FRAMEWORK_DIST",
                    self.root.display(),
                    self.root.display()
                ));
            }
        }
        let mut paths = BTreeSet::new();
        for file in &self.manifest.runtime_files {
            let path = safe_relative(&file.path, "runtime file")?;
            if !paths.insert(path) {
                return Err(format!("duplicate Framework runtime file: {}", file.path));
            }
            if file.sha256.len() != 64 || !file.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
            {
                return Err(format!("invalid Framework runtime hash: {}", file.path));
            }
        }
        Ok(())
    }
}

fn local_distribution(
    framework: &Path,
    profile: &str,
    targets: &[String],
) -> Result<Option<Distribution>, String> {
    let suffix = if profile == "release" {
        String::new()
    } else {
        format!("-{profile}")
    };
    for target in targets {
        let cpu = match target.split('-').next() {
            Some("x86_64") => "x64",
            Some("aarch64") => "arm64",
            _ => continue,
        };
        let platform = if target.contains("apple-darwin") {
            "macos"
        } else if target.contains("windows") {
            "windows"
        } else {
            "linux"
        };
        // x.py owns canonical naming; inspect manifests to prevent cross-platform reuse.
        let candidates = [
            framework
                .join("dist")
                .join(format!("tuffite-framework{suffix}-{platform}-{cpu}")),
            framework
                .join("dist")
                .join(format!("tuffite-framework{suffix}-{cpu}")),
            framework.join("dist/tuffite-framework"),
        ];
        for path in candidates {
            if !path.exists() {
                continue;
            }
            let distribution = Distribution::open(&path)?;
            if distribution.manifest().target == *target
                && distribution.manifest().profile == profile
            {
                return Ok(Some(distribution));
            }
        }
    }
    Ok(None)
}

/// Resolves a local Framework Distribution and, by default, downloads a missing
/// version-matched GitHub Release asset into the user cache.
#[derive(Clone, Debug)]
pub struct Resolver {
    distribution: Option<PathBuf>,
    framework_directory: Option<PathBuf>,
    cache_root: Option<PathBuf>,
    download_directory: Option<PathBuf>,
    target: Option<String>,
    repository: Option<String>,
    release_tag: Option<String>,
    auto_download: bool,
    force_download: bool,
    progress: bool,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            distribution: None,
            framework_directory: None,
            cache_root: None,
            download_directory: None,
            target: None,
            repository: None,
            release_tag: None,
            auto_download: true,
            force_download: false,
            progress: true,
        }
    }

    /// Loads resolver overrides from the Tuffite environment variables.
    pub fn from_env() -> Result<Self, String> {
        let mut resolver = Self::new();
        resolver.distribution = env::var_os(DIR_ENV).map(PathBuf::from);
        resolver.framework_directory = env::var_os(FRAMEWORK_ENV)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        resolver.cache_root = env::var_os(CACHE_ENV).map(PathBuf::from);
        resolver.download_directory = env::var_os(DOWNLOAD_DIR_ENV).map(PathBuf::from);
        resolver.target = env::var(TARGET_ENV)
            .ok()
            .filter(|value| !value.is_empty())
            .or_else(|| env::var("TARGET").ok().filter(|value| !value.is_empty()));
        resolver.repository = env::var(REPOSITORY_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        resolver.release_tag = env::var(RELEASE_TAG_ENV)
            .ok()
            .filter(|value| !value.is_empty());
        if let Ok(value) = env::var(AUTO_DOWNLOAD_ENV) {
            resolver.auto_download = parse_env_bool(AUTO_DOWNLOAD_ENV, &value)?;
        }
        resolver.progress = progress_enabled()?;
        Ok(resolver)
    }

    /// Uses an existing developer-managed Distribution and never replaces it.
    #[must_use]
    pub fn distribution(mut self, path: impl Into<PathBuf>) -> Self {
        self.distribution = Some(path.into());
        self
    }

    /// Ignores local Distribution and Framework checkout overrides and resolves
    /// a Tuffite-managed cache/download destination.
    #[must_use]
    pub fn managed(mut self) -> Self {
        self.distribution = None;
        self.framework_directory = None;
        self
    }

    /// Prefer a compatible locally packaged Framework Distribution.
    #[must_use]
    pub fn framework_directory(mut self, directory: impl Into<PathBuf>) -> Self {
        self.framework_directory = Some(directory.into());
        self
    }

    #[must_use]
    pub fn cache_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_root = Some(path.into());
        self
    }

    /// Sets the exact directory where an automatic download is installed.
    #[must_use]
    pub fn download_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.download_directory = Some(path.into());
        self
    }

    #[must_use]
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Selects the GitHub release repository as `owner/name`. Custom repositories
    /// and release tags use separate managed cache namespaces.
    #[must_use]
    pub fn repository(mut self, repository: impl Into<String>) -> Self {
        self.repository = Some(repository.into());
        self
    }

    /// Selects the exact GitHub release tag, independently of the SDK version.
    #[must_use]
    pub fn release_tag(mut self, tag: impl Into<String>) -> Self {
        self.release_tag = Some(tag.into());
        self
    }

    /// Controls network access on a cache miss. Defaults to `true`.
    #[must_use]
    pub const fn auto_download(mut self, enabled: bool) -> Self {
        self.auto_download = enabled;
        self
    }

    /// Downloads and atomically replaces the managed destination even when a
    /// valid Distribution is already installed.
    #[must_use]
    pub const fn force_download(mut self, enabled: bool) -> Self {
        self.force_download = enabled;
        self
    }

    /// Controls transfer progress messages. Defaults to `true`.
    #[must_use]
    pub const fn progress(mut self, enabled: bool) -> Self {
        self.progress = enabled;
        self
    }

    fn resolve_explicit(&self) -> Result<Option<Distribution>, String> {
        if let Some(path) = self.distribution.as_deref() {
            let distribution = Distribution::open(path)?;
            ensure_runtime_compatibility(&distribution.manifest().target)?;
            tracing::info!(
                framework = %distribution.root().display(),
                source = DIR_ENV,
                "using developer-managed Tuffite Framework Distribution"
            );
            return Ok(Some(distribution));
        }
        Ok(None)
    }

    fn resolve_local(&self, compatible_targets: &[String]) -> Result<Option<Distribution>, String> {
        if self.force_download {
            return Ok(None);
        }
        let profile = "release";
        if let Some(framework) = self.framework_directory.as_deref()
            && let Some(distribution) = local_distribution(framework, profile, compatible_targets)?
        {
            ensure_runtime_compatibility(&distribution.manifest().target)?;
            tracing::info!(framework = %distribution.root().display(), "using local Framework Framework Distribution");
            return Ok(Some(distribution));
        }
        Ok(None)
    }

    fn managed_cache_root(&self, repository: &str, tag: &str) -> Option<PathBuf> {
        let root = self.cache_root.clone().or_else(cache_root)?;
        if self.repository.is_some() || self.release_tag.is_some() {
            let identity = Sha256::digest(format!("{repository}\0{tag}").as_bytes());
            Some(root.join("sources").join(format!("{identity:x}")))
        } else {
            Some(root)
        }
    }

    pub fn resolve(self) -> Result<Distribution, String> {
        if let Some(distribution) = self.resolve_explicit()? {
            return Ok(distribution);
        }
        let target_is_explicit = self.target.is_some();
        let requested_target = self.target.clone().unwrap_or_else(host_target);
        validate_release_identifier(&requested_target, "target")?;
        let compatible_targets = compatible_targets(&requested_target, target_is_explicit);
        if let Some(distribution) = self.resolve_local(&compatible_targets)? {
            return Ok(distribution);
        }
        let repository = resolve_repository(self.repository.as_deref())?;
        let tag = self.release_tag.clone().unwrap_or_else(default_release_tag);
        validate_release_identifier(&tag, "release tag")?;
        let cache_root = self.managed_cache_root(&repository, &tag);

        if !self.force_download {
            if let Some(destination) = self.download_directory.as_deref() {
                if let Ok(distribution) = Distribution::open(destination)
                    && compatible_targets.contains(&distribution.manifest().target)
                {
                    report_target_fallback(&requested_target, &distribution.manifest().target);
                    ensure_runtime_compatibility(&distribution.manifest().target)?;
                    tracing::info!(
                        framework = %distribution.root().display(),
                        "using cached Tuffite Framework Distribution"
                    );
                    return Ok(distribution);
                }
            } else if let Some(root) = cache_root.as_deref() {
                for target in &compatible_targets {
                    let destination = root.join(env!("CARGO_PKG_VERSION")).join(target);
                    if let Ok(distribution) = Distribution::open(&destination) {
                        report_target_fallback(&requested_target, target);
                        ensure_runtime_compatibility(target)?;
                        tracing::info!(
                            framework = %distribution.root().display(),
                            cache_root = %root.display(),
                            "using cached Tuffite Framework Distribution"
                        );
                        return Ok(distribution);
                    }
                }
            }
        }
        if !self.auto_download {
            return Err(format!(
                "no compatible Tuffite Framework Distribution for Tuffite {} targets {}; automatic download is disabled by {AUTO_DOWNLOAD_ENV}. Set {DIR_ENV} to a developer-managed Distribution or enable downloads",
                env!("CARGO_PKG_VERSION"),
                compatible_targets.join(", "),
            ));
        }

        let token = github_token_optional();
        let release = github_release(&repository, &tag, token.as_deref())?
            .ok_or_else(|| format!("GitHub Release {tag} does not exist in {repository}"))?;
        let (target, asset_id) = select_release_asset(&release, &compatible_targets).ok_or_else(|| {
            let assets = compatible_targets
                .iter()
                .map(|target| release_asset_name(target))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "GitHub Release {tag} in {repository} does not contain a compatible Framework asset; looked for {assets}"
            )
        })?;
        tracing::info!(
            repository,
            tag,
            asset = %release_asset_name(target),
            target,
            "selected GitHub Release Framework asset"
        );
        report_target_fallback(&requested_target, target);
        ensure_runtime_compatibility(target)?;
        let destination = if let Some(path) = self.download_directory {
            path
        } else {
            cache_root
                .ok_or_else(|| {
                    "unable to determine the Tuffite Framework cache directory".to_owned()
                })?
                .join(env!("CARGO_PKG_VERSION"))
                .join(target)
        };
        tracing::info!(
            framework = %destination.display(),
            "Tuffite Framework download destination"
        );
        download_distribution(
            &repository,
            &tag,
            target,
            asset_id,
            token.as_deref(),
            &destination,
            self.progress,
        )?;
        Distribution::open(destination)
    }
}

fn compatible_targets(requested: &str, explicit: bool) -> Vec<String> {
    let mut targets = vec![requested.to_owned()];
    if !explicit && requested == "aarch64-apple-darwin" {
        targets.push("x86_64-apple-darwin".to_owned());
    }
    targets
}

fn report_target_fallback(requested: &str, selected: &str) {
    if selected != requested {
        tracing::warn!(
            requested_target = requested,
            selected_target = selected,
            "native macOS ARM64 Framework is unavailable; using the x86_64 artifact through Rosetta 2"
        );
    }
}

fn select_release_asset<'a>(
    release: &'a GithubRelease,
    targets: &'a [String],
) -> Option<(&'a str, u64)> {
    targets.iter().find_map(|target| {
        let name = release_asset_name(target);
        release
            .assets
            .iter()
            .find(|asset| asset.name == name)
            .map(|asset| (target.as_str(), asset.id))
    })
}

fn ensure_runtime_compatibility(target: &str) -> Result<(), String> {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) && target == "x86_64-apple-darwin" {
        let available = Command::new("/usr/bin/arch")
            .args(["-x86_64", "/usr/bin/true"])
            .status()
            .is_ok_and(|status| status.success());
        if !available {
            return Err(
                "the available Framework is x86_64, but Rosetta 2 is not installed. Install Rosetta or publish tuffite-framework-aarch64-apple-darwin.zip"
                    .to_owned(),
            );
        }
    }
    Ok(())
}

/// Resolves the Framework using environment overrides and automatic download.
pub fn resolve() -> Result<Distribution, String> {
    Resolver::from_env()?.resolve()
}

/// Returns whether Framework transfers should report progress according to the
/// process environment. Progress is enabled when the variable is absent.
pub fn progress_enabled() -> Result<bool, String> {
    env::var(PROGRESS_ENV).map_or(Ok(true), |value| parse_env_bool(PROGRESS_ENV, &value))
}

#[must_use]
pub fn cache_root() -> Option<PathBuf> {
    if let Some(path) = env::var_os(CACHE_ENV) {
        return Some(PathBuf::from(path));
    }
    if cfg!(target_os = "windows") {
        env::var_os("LOCALAPPDATA")
            .map(|path| PathBuf::from(path).join("Tuffite").join("framework"))
    } else if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        Some(PathBuf::from(path).join("tuffite").join("framework"))
    } else {
        env::var_os("HOME").map(|path| {
            PathBuf::from(path)
                .join(".cache")
                .join("tuffite")
                .join("framework")
        })
    }
}

#[must_use]
pub fn host_target() -> String {
    if cfg!(target_os = "windows") {
        format!("{}-pc-windows-msvc", env::consts::ARCH)
    } else if cfg!(target_os = "macos") {
        format!("{}-apple-darwin", env::consts::ARCH)
    } else {
        format!("{}-unknown-linux-gnu", env::consts::ARCH)
    }
}

#[derive(Deserialize)]
struct GithubRelease {
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    id: u64,
    name: String,
}

fn download_distribution(
    repository: &str,
    tag: &str,
    target: &str,
    asset_id: u64,
    token: Option<&str>,
    destination: &Path,
    progress: bool,
) -> Result<(), String> {
    let parent = destination.parent().ok_or_else(|| {
        format!(
            "Framework download destination has no parent: {}",
            destination.display()
        )
    })?;
    fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    let asset = release_asset_name(target);
    let url = format!("https://api.github.com/repos/{repository}/releases/assets/{asset_id}");
    let archive = parent.join(format!(".{asset}.{}.download", std::process::id()));
    let staging = parent.join(format!(
        ".{}.{}.staging",
        destination
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("framework"),
        std::process::id()
    ));
    remove_path(&archive)?;
    remove_path(&staging)?;
    let result = (|| {
        tracing::info!(repository, tag, asset, "downloading Framework Distribution");
        download_archive(&url, &archive, token, progress)?;
        extract_archive(&archive, &staging)?;
        let distribution = Distribution::open(&staging)?;
        if distribution.manifest().target != target {
            return Err(format!(
                "downloaded Framework target {} does not match requested {target}",
                distribution.manifest().target
            ));
        }
        distribution.runtime_files(true)?;
        distribution.bootstrap(true)?;
        install_distribution(&staging, destination)?;
        tracing::info!(
            framework = %destination.display(),
            repository,
            tag,
            asset,
            "installed downloaded Tuffite Framework Distribution"
        );
        Ok(())
    })();
    let _ = remove_path(&archive);
    if result.is_err() {
        let _ = remove_path(&staging);
    }
    result
}

fn install_distribution(staging: &Path, destination: &Path) -> Result<(), String> {
    let parent = destination.parent().ok_or_else(|| {
        format!(
            "Framework destination has no parent: {}",
            destination.display()
        )
    })?;
    let backup = parent.join(format!(
        ".{}.{}.backup",
        destination
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("framework"),
        std::process::id()
    ));
    remove_path(&backup)?;
    let had_destination = destination.exists();
    if had_destination {
        fs::rename(destination, &backup).map_err(|error| {
            format!(
                "prepare to replace Framework {}: {error}",
                destination.display()
            )
        })?;
    }
    if let Err(error) = fs::rename(staging, destination) {
        if had_destination {
            let _ = fs::rename(&backup, destination);
        }
        return Err(format!(
            "install downloaded Framework {}: {error}",
            destination.display()
        ));
    }
    if had_destination {
        remove_path(&backup)?;
    }
    Ok(())
}

fn download_archive(
    url: &str,
    destination: &Path,
    token: Option<&str>,
    progress: bool,
) -> Result<(), String> {
    let mut request = ureq::get(url)
        .header("User-Agent", "tuffite-build")
        .header("Accept", "application/octet-stream")
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION);
    if let Some(token) = token {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let response = request
        .call()
        .map_err(|error| format!("download {url}: {error}"))?;
    let total = response
        .headers()
        .get("content-length")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse().ok());
    let reader = response.into_body().into_reader();
    let mut reader = TransferProgress::new(reader, total, "download", progress);
    let mut output = File::create(destination)
        .map_err(|error| format!("create {}: {error}", destination.display()))?;
    io::copy(&mut reader, &mut output)
        .map_err(|error| format!("write {}: {error}", destination.display()))?;
    output
        .sync_all()
        .map_err(|error| format!("sync {}: {error}", destination.display()))
}

pub(crate) struct TransferProgress<R> {
    reader: R,
    total: Option<u64>,
    transferred: u64,
    next_report: u64,
    action: &'static str,
    enabled: bool,
    interactive: bool,
    progress_bar: Option<ProgressBar>,
    finished: bool,
}

impl<R> TransferProgress<R> {
    pub(crate) fn new(reader: R, total: Option<u64>, action: &'static str, enabled: bool) -> Self {
        let interactive = enabled && io::stderr().is_terminal();
        let progress_bar = interactive.then(|| {
            let bar = total.map_or_else(ProgressBar::no_length, ProgressBar::new);
            bar.set_draw_target(ProgressDrawTarget::stderr_with_hz(10));
            let template = if total.is_some() {
                "{spinner:.cyan} {msg} [{bar:36.cyan/blue}] {bytes}/{total_bytes} {bytes_per_sec} ETA {eta}"
            } else {
                "{spinner:.cyan} {msg} {bytes} {bytes_per_sec} {elapsed_precise}"
            };
            bar.set_style(
                ProgressStyle::with_template(template)
                    .expect("the Tuffite transfer progress template must be valid")
                    .progress_chars("=>-"),
            );
            bar.set_message(match action {
                "upload" => "Uploading Framework",
                "download" => "Downloading Framework",
                _ => "Transferring Framework",
            });
            bar
        });
        Self {
            reader,
            total,
            transferred: 0,
            next_report: 64 * 1024 * 1024,
            action,
            enabled,
            interactive,
            progress_bar,
            finished: false,
        }
    }

    fn report(&mut self, finished: bool) {
        if let Some(bar) = &self.progress_bar {
            bar.set_position(self.transferred);
            if finished && !self.finished {
                bar.finish_and_clear();
                self.finished = true;
            }
            return;
        }
        if !self.enabled
            || self.interactive
            || self.finished
            || (!finished && self.transferred < self.next_report)
        {
            return;
        }
        self.next_report = self.transferred.saturating_add(64 * 1024 * 1024);
        self.finished = finished;
        if let Some(total) = self.total {
            let percent = if total == 0 {
                100
            } else {
                (self.transferred.saturating_mul(100) / total).min(100)
            };
            if finished {
                tracing::info!(
                    action = self.action,
                    bytes = self.transferred,
                    total_bytes = total,
                    percent,
                    "Framework transfer complete"
                );
            } else {
                tracing::info!(
                    action = self.action,
                    bytes = self.transferred,
                    total_bytes = total,
                    percent,
                    "Framework transfer progress"
                );
            }
        } else if finished {
            tracing::info!(
                action = self.action,
                bytes = self.transferred,
                "Framework transfer complete"
            );
        } else {
            tracing::info!(
                action = self.action,
                bytes = self.transferred,
                "Framework transfer progress"
            );
        }
    }
}

impl<R: io::Read> io::Read for TransferProgress<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self.reader.read(buffer)?;
        self.transferred = self.transferred.saturating_add(read as u64);
        self.report(read == 0);
        Ok(read)
    }
}

impl<R> Drop for TransferProgress<R> {
    fn drop(&mut self) {
        if let Some(bar) = &self.progress_bar {
            bar.finish_and_clear();
        }
    }
}

fn extract_archive(archive: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("create {}: {error}", destination.display()))?;
    let file = File::open(archive)
        .map_err(|error| format!("open downloaded archive {}: {error}", archive.display()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|error| format!("open ZIP {}: {error}", archive.display()))?;
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("read ZIP entry: {error}"))?;
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("unsafe ZIP entry: {}", entry.name()))?;
        let output = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&output)
                .map_err(|error| format!("create {}: {error}", output.display()))?;
            continue;
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170_000 == 0o120_000)
        {
            return Err(format!("ZIP entry is a symbolic link: {}", entry.name()));
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        let mut file = File::create(&output)
            .map_err(|error| format!("create {}: {error}", output.display()))?;
        io::copy(&mut entry, &mut file)
            .map_err(|error| format!("extract {}: {error}", output.display()))?;
        file.flush()
            .map_err(|error| format!("flush {}: {error}", output.display()))?;
        restore_permissions(&output, entry.unix_mode())?;
    }
    Ok(())
}

#[cfg_attr(not(unix), allow(clippy::unnecessary_wraps))]
fn restore_permissions(path: &Path, mode: Option<u32>) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        if let Some(mode) = mode {
            fs::set_permissions(path, fs::Permissions::from_mode(mode & 0o777))
                .map_err(|error| format!("set permissions on {}: {error}", path.display()))?;
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, mode);
    }
    Ok(())
}

fn github_release(
    repository: &str,
    tag: &str,
    token: Option<&str>,
) -> Result<Option<GithubRelease>, String> {
    let url = format!("https://api.github.com/repos/{repository}/releases/tags/{tag}");
    let mut request = ureq::get(&url)
        .header("User-Agent", "tuffite-build")
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", GITHUB_API_VERSION);
    if let Some(token) = token {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let mut response = match request.call() {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(404)) => return Ok(None),
        Err(error) => return Err(format!("read GitHub Release {tag}: {error}")),
    };
    response
        .body_mut()
        .read_json()
        .map(Some)
        .map_err(|error| format!("parse GitHub Release {tag}: {error}"))
}

fn resolve_repository(override_repository: Option<&str>) -> Result<String, String> {
    if let Some(repository) = override_repository {
        validate_repository(repository)?;
        return Ok(repository.to_owned());
    }
    let repository_url = env!("CARGO_PKG_REPOSITORY");
    let url = Url::parse(repository_url)
        .map_err(|error| format!("parse compiled repository URL {repository_url}: {error}"))?;
    if url.host_str() != Some("github.com") {
        return Err(format!(
            "compiled package repository is not hosted on GitHub: {repository_url}"
        ));
    }
    let mut segments = url
        .path_segments()
        .ok_or_else(|| format!("compiled repository URL has no path: {repository_url}"))?
        .filter(|segment| !segment.is_empty());
    let owner = segments
        .next()
        .ok_or_else(|| format!("compiled repository URL has no owner: {repository_url}"))?;
    let name = segments
        .next()
        .ok_or_else(|| format!("compiled repository URL has no name: {repository_url}"))?
        .trim_end_matches(".git");
    if segments.next().is_some() {
        return Err(format!(
            "compiled repository URL is not a GitHub repository root: {repository_url}"
        ));
    }
    let repository = format!("{owner}/{name}");
    validate_repository(&repository)?;
    Ok(repository)
}

fn validate_repository(repository: &str) -> Result<(), String> {
    let mut parts = repository.split('/');
    let valid = parts.next().is_some_and(valid_github_name)
        && parts.next().is_some_and(valid_github_name)
        && parts.next().is_none();
    if valid {
        Ok(())
    } else {
        Err(format!(
            "invalid GitHub repository {repository:?}; expected owner/name"
        ))
    }
}

fn valid_github_name(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn validate_release_identifier(value: &str, label: &str) -> Result<(), String> {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        Ok(())
    } else {
        Err(format!("invalid Framework {label} {value:?}"))
    }
}

fn parse_env_bool(name: &str, value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        _ => Err(format!(
            "{name} must be one of 1, 0, true, false, yes, no, on, or off; got {value:?}"
        )),
    }
}

fn default_release_tag() -> String {
    format!("v{}", env!("CARGO_PKG_VERSION"))
}

#[must_use]
pub fn release_asset_name(target: &str) -> String {
    format!("tuffite-framework-{target}.zip")
}

fn github_token_optional() -> Option<String> {
    env::var("GITHUB_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| env::var("GH_TOKEN").ok().filter(|value| !value.is_empty()))
        .or_else(github_cli_token)
}

fn github_cli_token() -> Option<String> {
    let output = Command::new("gh").args(["auth", "token"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8(output.stdout).ok()?;
    let token = token.trim();
    (!token.is_empty()).then(|| token.to_owned())
}

fn remove_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path).map_err(|error| format!("remove {}: {error}", path.display()))
    } else {
        fs::remove_file(path).map_err(|error| format!("remove {}: {error}", path.display()))
    }
}

fn safe_join(root: &Path, value: &str, field: &str) -> Result<PathBuf, String> {
    Ok(root.join(safe_relative(value, field)?))
}

fn safe_relative(value: &str, field: &str) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if value.is_empty() || path.is_absolute() {
        return Err(format!("invalid Framework {field}: {value}"));
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) if part != OsStr::new("") => result.push(part),
            _ => return Err(format!("unsafe Framework {field}: {value}")),
        }
    }
    Ok(result)
}

fn hash_file(path: &Path) -> Result<String, String> {
    use std::io::Read as _;
    let mut file = File::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 64 * 1024].into_boxed_slice();
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("read {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use std::io::Read as _;

    use super::{
        GithubAsset, GithubRelease, Resolver, TransferProgress, compatible_targets, parse_env_bool,
        release_asset_name, select_release_asset, validate_release_identifier,
    };

    #[test]
    fn boolean_environment_values_are_explicit() {
        for value in ["1", "true", "YES", "on"] {
            assert!(parse_env_bool("TEST", value).unwrap());
        }
        for value in ["0", "false", "NO", "off"] {
            assert!(!parse_env_bool("TEST", value).unwrap());
        }
        assert!(parse_env_bool("TEST", "sometimes").is_err());
    }

    #[test]
    fn release_identifiers_cannot_escape_urls_or_paths() {
        assert!(validate_release_identifier("x86_64-pc-windows-msvc", "target").is_ok());
        assert!(validate_release_identifier("../windows", "target").is_err());
        assert_eq!(
            release_asset_name("aarch64-apple-darwin"),
            "tuffite-framework-aarch64-apple-darwin.zip"
        );
    }

    #[test]
    fn macos_arm_can_fall_back_to_x86_64_only_when_target_is_implicit() {
        assert_eq!(
            compatible_targets("aarch64-apple-darwin", false),
            ["aarch64-apple-darwin", "x86_64-apple-darwin"]
        );
        assert_eq!(
            compatible_targets("aarch64-apple-darwin", true),
            ["aarch64-apple-darwin"]
        );
        assert_eq!(
            compatible_targets("aarch64-unknown-linux-gnu", false),
            ["aarch64-unknown-linux-gnu"]
        );
    }

    #[test]
    fn release_selection_prefers_native_then_rosetta_asset() {
        let targets = compatible_targets("aarch64-apple-darwin", false);
        let x64_only = GithubRelease {
            assets: vec![GithubAsset {
                id: 7,
                name: release_asset_name("x86_64-apple-darwin"),
            }],
        };
        assert_eq!(
            select_release_asset(&x64_only, &targets),
            Some(("x86_64-apple-darwin", 7))
        );
        let both = GithubRelease {
            assets: vec![
                GithubAsset {
                    id: 7,
                    name: release_asset_name("x86_64-apple-darwin"),
                },
                GithubAsset {
                    id: 8,
                    name: release_asset_name("aarch64-apple-darwin"),
                },
            ],
        };
        assert_eq!(
            select_release_asset(&both, &targets),
            Some(("aarch64-apple-darwin", 8))
        );
    }

    #[test]
    fn disabling_download_fails_without_network_access() {
        let destination = std::env::temp_dir().join(format!(
            "tuffite-missing-framework-{}-{}",
            std::process::id(),
            line!()
        ));
        let error = Resolver::new()
            .download_directory(destination)
            .target("x86_64-test-none")
            .auto_download(false)
            .resolve()
            .unwrap_err();
        assert!(error.contains("automatic download is disabled"), "{error}");
    }

    #[test]
    fn progress_reader_preserves_the_stream() {
        let input = b"tuffite".as_slice();
        let mut reader = TransferProgress::new(input, Some(6), "tested", false);
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, b"tuffite");
    }
    struct LocalFixture(std::path::PathBuf);
    impl LocalFixture {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tuffite-local-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&root).unwrap();
            Self(root)
        }
        fn artifact(&self, path: &str, target: &str) -> std::path::PathBuf {
            let root = self.0.join(path);
            std::fs::create_dir_all(root.join("lib")).unwrap();
            std::fs::create_dir_all(root.join("runtime")).unwrap();
            std::fs::write(root.join("launcher"), []).unwrap();
            let mut runtime_files = vec![
                serde_json::json!({"path": "runtime.bin", "size": 0, "sha256": "0".repeat(64)}),
            ];
            for name in [
                "LICENSE.tuffite.txt",
                "LICENSE.chromium.txt",
                "THIRD_PARTY_NOTICES.chromium.txt",
            ] {
                let path = format!("licenses/{name}");
                std::fs::create_dir_all(root.join("runtime/licenses")).unwrap();
                std::fs::write(root.join("runtime").join(&path), b"license").unwrap();
                runtime_files
                    .push(serde_json::json!({"path": path, "size": 7, "sha256": "0".repeat(64)}));
            }
            let manifest = serde_json::json!({
                "framework_version": env!("CARGO_PKG_VERSION"), "chromium_revision": "fixture", "target": target,
                "profile": "release", "variant": "full", "component_build": false, "features": [],
                "ffi_abi_major": tuffite_abi::ABI_MAJOR, "ffi_abi_minor": tuffite_abi::ABI_MINOR, "ffi_abi_patch": tuffite_abi::ABI_PATCH,
                "link_directory": "lib", "runtime_directory": "runtime",
                "bootstrap": {"path": "launcher", "size": 0, "sha256": "0".repeat(64)},
                "runtime_files": runtime_files});
            std::fs::write(
                root.join(super::MANIFEST_FILE),
                serde_json::to_vec(&manifest).unwrap(),
            )
            .unwrap();
            root.canonicalize().unwrap()
        }
    }
    impl Drop for LocalFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn legacy_distribution_without_license_entries_requires_repackaging() {
        let fixture = LocalFixture::new();
        let root = fixture.artifact("legacy", "x86_64-apple-darwin");
        let path = root.join(super::MANIFEST_FILE);
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        manifest["runtime_files"]
            .as_array_mut()
            .unwrap()
            .retain(|file| !file["path"].as_str().unwrap().starts_with("licenses/"));
        std::fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        let error = super::Distribution::open(root).unwrap_err();
        assert!(error.contains("missing notice"), "{error}");
    }

    #[test]
    fn local_framework_output_precedes_managed_cache() {
        let fixture = LocalFixture::new();
        let target = super::host_target();
        let cpu = if target.starts_with("aarch64") {
            "arm64"
        } else {
            "x64"
        };
        let local = fixture.artifact(&format!("dist/tuffite-framework-{cpu}"), &target);
        fixture.artifact(
            &format!("cache/{}/{target}", env!("CARGO_PKG_VERSION")),
            &target,
        );
        let resolved = Resolver::new()
            .framework_directory(&fixture.0)
            .cache_root(fixture.0.join("cache"))
            .auto_download(false)
            .target(target)
            .resolve()
            .unwrap();
        assert_eq!(resolved.root(), local);
    }
    #[test]
    fn managed_resolution_bypasses_local_framework_output() {
        let fixture = LocalFixture::new();
        let target = super::host_target();
        let cache = fixture.artifact(
            &format!("cache/{}/{target}", env!("CARGO_PKG_VERSION")),
            &target,
        );
        let cpu = if target.starts_with("aarch64") {
            "arm64"
        } else {
            "x64"
        };
        let local = fixture.artifact(&format!("dist/tuffite-framework-{cpu}"), &target);
        let resolver = Resolver::new()
            .framework_directory(&fixture.0)
            .cache_root(fixture.0.join("cache"))
            .target(target)
            .auto_download(false);
        assert_eq!(resolver.clone().resolve().unwrap().root(), local);
        assert_eq!(resolver.managed().resolve().unwrap().root(), cache);
    }
    #[test]
    fn custom_release_does_not_reuse_default_or_other_source_cache() {
        use sha2::Digest as _;
        let fixture = LocalFixture::new();
        let target = super::host_target();
        let cache_root = fixture.0.join("cache");
        fixture.artifact(
            &format!("cache/{}/{target}", env!("CARGO_PKG_VERSION")),
            &target,
        );
        for resolver in [
            Resolver::new().repository("example/framework"),
            Resolver::new().release_tag("v-custom"),
        ] {
            assert!(
                resolver
                    .target(&target)
                    .cache_root(&cache_root)
                    .auto_download(false)
                    .resolve()
                    .is_err()
            );
        }
        let resolver = Resolver::new()
            .repository("example/framework")
            .release_tag("v-custom")
            .target(&target)
            .cache_root(&cache_root)
            .auto_download(false);
        assert!(
            resolver
                .clone()
                .resolve()
                .unwrap_err()
                .contains("automatic download is disabled")
        );
        let identity = super::Sha256::digest(b"example/framework\0v-custom");
        let cached = fixture.artifact(
            &format!(
                "cache/sources/{identity:x}/{}/{target}",
                env!("CARGO_PKG_VERSION")
            ),
            &target,
        );
        assert_eq!(resolver.clone().resolve().unwrap().root(), cached);
        assert!(resolver.clone().release_tag("v-other").resolve().is_err());
        assert!(resolver.repository("other/framework").resolve().is_err());
    }

    #[test]
    fn local_discovery_checks_full_target_and_explicit_override_wins() {
        let fixture = LocalFixture::new();
        let target = super::host_target();
        let cpu = if target.starts_with("aarch64") {
            "arm64"
        } else {
            "x64"
        };
        fixture.artifact(
            &format!("dist/tuffite-framework-{cpu}"),
            "x86_64-other-platform",
        );
        assert!(
            super::local_distribution(&fixture.0, "release", std::slice::from_ref(&target))
                .unwrap()
                .is_none()
        );
        let explicit = fixture.artifact("explicit", &target);
        assert_eq!(
            Resolver::new()
                .framework_directory(&fixture.0)
                .distribution(&explicit)
                .resolve()
                .unwrap()
                .root(),
            explicit
        );
    }
}
