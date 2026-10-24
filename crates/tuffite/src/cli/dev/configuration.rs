//! Validated project settings and CLI overrides.
use super::super::filesystem::reject_symlink;
use super::Options;
use super::server::ServerConfiguration;
use serde::Deserialize;
use std::ffi::OsString;
use std::fs::File;
use std::path::{Component, Path, PathBuf};

#[derive(Deserialize)]
struct AppManifest {
    #[serde(default)]
    build: Build,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BeforeDevCommand {
    Script(String),
    Options(BeforeDevCommandOptions),
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BeforeDevCommandOptions {
    script: String,
    cwd: Option<PathBuf>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Build {
    dev_package: Option<PathBuf>,
    dev_url: Option<String>,
    before_dev_command: Option<BeforeDevCommand>,
    devtools_port: Option<u16>,
}

pub(super) struct Configuration {
    pub(super) root: PathBuf,
    pub(super) manifest: Option<PathBuf>,
    pub(super) package: Option<PathBuf>,
    pub(super) server: ServerConfiguration,
    pub(super) devtools_port: Option<u16>,
}

impl Configuration {
    pub(super) fn load(options: &Options) -> Result<Configuration, String> {
        let requested_manifest =
            super::super::config::discover_manifest(options.manifest.as_deref())?;
        let (root, manifest, build) = if let Some(requested) = requested_manifest {
            let manifest_path = std::fs::canonicalize(&requested).map_err(|error| {
                format!(
                    "resolve development manifest {}: {error}",
                    requested.display()
                )
            })?;
            reject_symlink(&manifest_path)?;
            super::super::config::check(&manifest_path)?;
            let application: AppManifest = serde_json::from_reader(
                File::open(&manifest_path)
                    .map_err(|error| format!("open {}: {error}", manifest_path.display()))?,
            )
            .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
            let root = manifest_path
                .parent()
                .ok_or_else(|| "development manifest has no parent directory".to_owned())?
                .to_path_buf();
            (root, Some(manifest_path), application.build)
        } else {
            (
                std::env::current_dir()
                    .map_err(|error| format!("read current directory: {error}"))?,
                None,
                Build::default(),
            )
        };
        let package = options.package.clone().or(build.dev_package);
        if package.is_none() && manifest.is_none() {
            return Err("pass --package when running without an application manifest".to_owned());
        }
        let url = options
            .url
            .clone()
            .or(build.dev_url)
            .unwrap_or_else(|| "http://127.0.0.1:5173/".to_owned());
        let cli_command = options.web_command.is_some() || !options.web_arg.is_empty();
        let (web_command, web_args, configured_cwd) = if cli_command {
            let command = options
                .web_command
                .clone()
                .unwrap_or_else(|| PathBuf::from("bun"));
            let args = if options.web_arg.is_empty() {
                vec![OsString::from("run"), OsString::from("dev")]
            } else {
                options.web_arg.clone()
            };
            (command, args, root.clone())
        } else {
            shell_command(
                build
                    .before_dev_command
                    .unwrap_or_else(|| BeforeDevCommand::Script("bun run dev".to_owned())),
                &root,
            )
        };
        let web_cwd = options
            .web_cwd
            .clone()
            .map_or(configured_cwd, |path| resolve_from(&root, path));
        let devtools_port = options.devtools_port.or(build.devtools_port);
        if devtools_port == Some(0) {
            return Err("build.devtoolsPort must be between 1 and 65535".to_owned());
        }
        Ok(Configuration {
            manifest,
            package: package.map(|path| resolve_from(&root, path)),
            server: ServerConfiguration {
                url: super::server::validate_development_url(&url)?,
                command: web_command,
                args: web_args,
                cwd: web_cwd,
            },
            devtools_port,
            root,
        })
    }
}

fn shell_command(command: BeforeDevCommand, root: &Path) -> (PathBuf, Vec<OsString>, PathBuf) {
    let (script, cwd) = match command {
        BeforeDevCommand::Script(script) => (script, root.to_path_buf()),
        BeforeDevCommand::Options(options) => (
            options.script,
            options
                .cwd
                .map_or_else(|| root.to_path_buf(), |path| resolve_from(root, path)),
        ),
    };
    if cfg!(windows) {
        (
            PathBuf::from("cmd"),
            vec![OsString::from("/S"), OsString::from("/C"), script.into()],
            cwd,
        )
    } else {
        (
            PathBuf::from("/bin/sh"),
            vec![OsString::from("-c"), script.into()],
            cwd,
        )
    }
}

fn resolve_from(root: &Path, path: PathBuf) -> PathBuf {
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized.file_name().is_some() {
                    normalized.pop();
                }
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn project_with_build(build: &str) -> String {
        let mut configuration: serde_json::Value = serde_json::from_str(build).unwrap();
        configuration["mojomSources"] = serde_json::json!(["shell_api.mojom"]);
        format!(
            r#"{{
                          "app": {{
                "identifier": "com.example.dev",
                "security": {{"capabilities": []}}
              }},
              "build": {configuration}
            }}"#
        )
    }

    #[test]
    fn reads_camel_case_build_defaults_from_manifest() {
        let root =
            std::env::temp_dir().join(format!("tuffite-dev-configuration-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("ui")).unwrap();
        let manifest = root.join("tuffite.json");
        fs::write(
            &manifest,
            project_with_build(
                r#"{
                "devPackage": "../dist/demo",
                "devUrl": "http://127.0.0.1:4321/",
                "beforeDevCommand": {
                  "script": "mise exec -- bun run dev",
                  "cwd": "ui"
                },
                "devtoolsPort": 9222
              }"#,
            ),
        )
        .unwrap();
        let options = Options {
            manifest: Some(manifest),
            package: None,
            url: None,
            web_command: None,
            web_arg: Vec::new(),
            web_cwd: None,
            app_arg: Vec::new(),
            devtools_port: None,
            prepare_only: false,
            release: false,
            force_download_framework: false,
            no_watch: false,
        };
        let configured = Configuration::load(&options).unwrap();
        let canonical_root = std::fs::canonicalize(&root).unwrap();
        assert_eq!(
            configured.package,
            Some(canonical_root.parent().unwrap().join("dist/demo"))
        );
        assert_eq!(configured.server.url.as_str(), "http://127.0.0.1:4321/");
        assert_eq!(
            configured.server.command,
            if cfg!(windows) {
                PathBuf::from("cmd")
            } else {
                PathBuf::from("/bin/sh")
            }
        );
        assert_eq!(configured.server.cwd, canonical_root.join("ui"));
        assert_eq!(configured.devtools_port, Some(9222));
        assert_eq!(
            configured.server.args,
            if cfg!(windows) {
                vec![
                    OsString::from("/S"),
                    OsString::from("/C"),
                    OsString::from("mise exec -- bun run dev"),
                ]
            } else {
                vec![
                    OsString::from("-c"),
                    OsString::from("mise exec -- bun run dev"),
                ]
            }
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn accepts_string_before_dev_command() {
        let root =
            std::env::temp_dir().join(format!("tuffite-dev-string-command-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let manifest = root.join("tuffite.json");
        fs::write(
            &manifest,
            project_with_build(
                r#"{
                "beforeDevCommand": "bun run dev"
              }"#,
            ),
        )
        .unwrap();
        let configured = Configuration::load(&Options {
            manifest: Some(manifest),
            package: None,
            url: None,
            web_command: None,
            web_arg: Vec::new(),
            web_cwd: None,
            app_arg: Vec::new(),
            devtools_port: None,
            prepare_only: false,
            release: false,
            force_download_framework: false,
            no_watch: false,
        })
        .unwrap();
        assert_eq!(configured.server.cwd, fs::canonicalize(&root).unwrap());
        assert_eq!(configured.server.args.last().unwrap(), "bun run dev");
        assert!(configured.package.is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_removed_development_configuration() {
        let root = std::env::temp_dir().join(format!(
            "tuffite-dev-removed-configuration-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let manifest = root.join("tuffite.json");
        fs::write(&manifest, r#"{"development":{"package":"dist/demo"}}"#).unwrap();
        let error = Configuration::load(&Options {
            manifest: Some(manifest),
            package: None,
            url: None,
            web_command: None,
            web_arg: Vec::new(),
            web_cwd: None,
            app_arg: Vec::new(),
            devtools_port: None,
            prepare_only: false,
            release: false,
            force_download_framework: false,
            no_watch: false,
        })
        .err()
        .unwrap();
        assert!(error.contains("unknown field"));
        fs::remove_dir_all(root).unwrap();
    }
}
