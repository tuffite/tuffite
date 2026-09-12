use std::fs;
use std::path::Path;
use std::process::Command;

#[allow(clippy::too_many_lines)]
pub fn init(directory: &Path, name: Option<&str>) -> Result<(), String> {
    if directory.exists()
        && fs::read_dir(directory)
            .map_err(|error| format!("inspect {}: {error}", directory.display()))?
            .next()
            .is_some()
    {
        return Err(format!(
            "initialization directory is not empty: {}",
            directory.display()
        ));
    }
    let inferred = directory
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("tuffite-app");
    let package = sanitize_name(name.unwrap_or(inferred));
    if package.is_empty() {
        return Err("project name must contain an ASCII letter or digit".into());
    }
    // The namespace is both a TypeScript identifier and the application URL scheme.
    let mut namespace = package.replace('-', "");
    if namespace.starts_with(|character: char| character.is_ascii_digit()) || namespace == "tuffite"
    {
        namespace.insert_str(0, "app");
    }
    let identifier = format!("org.tuffite.{namespace}");
    fs::create_dir_all(directory.join("src"))
        .map_err(|error| format!("create project directory: {error}"))?;
    fs::create_dir_all(directory.join("ui"))
        .map_err(|error| format!("create UI directory: {error}"))?;
    write(
        &directory.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{package}"
version = "0.1.0"
edition = "2024"
build = "build.rs"

[lib]
crate-type = ["cdylib"]

[dependencies]
tuffite = {{ version = "{}", default-features = false }}

[build-dependencies]
tuffite = {{ version = "{}", default-features = false, features = ["build"] }}
"#,
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_VERSION")
        ),
    )?;
    write(
        &directory.join("build.rs"),
        r#"use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(std::env::var_os("OUT_DIR").expect("OUT_DIR"));
    let framework = tuffite::build::framework::Resolver::from_env()
        .expect("configure Tuffite Framework resolver");
    let shell_api = tuffite::build::shell_api::Generator::new()
        .rust(out, tuffite::build::shell_api::OutputMode::Write)
        .javascript("generated", tuffite::build::shell_api::OutputMode::Write);
    tuffite::build::Builder::new()
        .add(framework)
        .add(shell_api)
        .build()
        .expect("prepare Tuffite application build");
}
"#,
    )?;
    write(
        &directory.join("src/lib.rs"),
        r#"use tuffite::{BrowserContext, ApplicationResult, CommandError};

mod shell_api { include!(concat!(env!("OUT_DIR"), "/bindings.rs")); }

struct AppBrowser;

#[tuffite::browser]
impl tuffite::Browser for AppBrowser {
    fn new(_context: BrowserContext) -> ApplicationResult<Self> { Ok(Self) }
}

#[tuffite::main(browser = AppBrowser)]
fn main() -> tuffite::app::Builder<AppBrowser> {
    tuffite::app::Builder::new().commands(tuffite::generate_handler![ping])
}

#[tuffite::command(shell_api::commands::Ping)]
fn ping(_app: &mut AppBrowser, message: String) -> Result<String, CommandError> { Ok(message) }
"#,
    )?;
    write(
        &directory.join("shell_api.d.ts"),
        &format!(
            "export namespace {namespace} {{ export namespace commands {{ export function ping(message: string): Promise<string>; }} }}\n"
        ),
    )?;
    let manifest = serde_json::json!({
        "$schema": "https://tuffite.org/schema/v2/tuffite.schema.json",
        "productName": package,
        "app": {
            "identifier": identifier,
            "security": {"csp": "default-src 'self'", "capabilities": [{
                "identifier": "app-shell", "origins": [format!("{namespace}://main")],
                "shellApi": [format!("{namespace}.*")]
            }]},
            "windows": [{
                "label": "main", "url": format!("{namespace}://main/index.html"),
                "title": package, "width": 1000, "height": 700
            }],
            "assets": [{"origins": [format!("{namespace}://main")], "paths": ["/index.html"], "file": "ui/index.html"}],
        },
        "build": {"shellApi": {"namespace": namespace, "definitions": ["shell_api.d.ts"]}},
        "bundle": {"resources": ["ui"]}
    });
    write(
        &directory.join("tuffite.json"),
        &(serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())? + "\n"),
    )?;
    write(
        &directory.join("ui/index.html"),
        "<!doctype html><meta charset=\"utf-8\"><title>Tuffite</title><h1>Tuffite is ready</h1>\n",
    )?;
    Ok(())
}

pub fn build(context: &mut super::Context, manifest: &Path, release: bool) -> Result<(), String> {
    super::config::check(manifest)?;
    let root = manifest
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let framework = context.framework()?;
    let mut command = super::package::cargo::cargo_command();
    command
        .arg("build")
        .arg("--lib")
        .arg("--target")
        .arg(&framework.manifest().target);
    command.env("TUFFITE_FRAMEWORK_DIST", framework.root());
    if release {
        command.arg("--release");
    }
    let status = command
        .current_dir(root)
        .status()
        .map_err(|error| format!("start cargo build: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo build exited with {status}"))
    }
}

pub fn doctor(context: &mut super::Context, manifest: &Path) -> Result<(), String> {
    super::config::check(manifest)?;
    tracing::info!(manifest = %manifest.display(), "manifest is valid");
    tracing::info!(
        host_target = crate::build::framework::host_target(),
        "host toolchain"
    );
    tracing::info!(cargo = %tool_version("cargo")?, rustc = %tool_version("rustc")?, "Rust toolchain");
    let distribution = context.framework()?;
    let identity = distribution.manifest();
    tracing::info!(
        framework = %distribution.root().display(),
        target = %identity.target,
        profile = %identity.profile,
        component = identity.component_build,
        development = identity.development_enabled,
        "Framework Distribution"
    );
    Ok(())
}

fn tool_version(tool: &str) -> Result<String, String> {
    let output = Command::new(tool)
        .arg("--version")
        .output()
        .map_err(|error| format!("run {tool} --version: {error}"))?;
    if !output.status.success() {
        return Err(format!("{tool} --version exited with {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn sanitize_name(value: &str) -> String {
    let value = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>();
    value.trim_matches('-').to_owned()
}

fn write(path: &Path, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|error| format!("write {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialized_projects_defer_binding_generation_to_build_script() {
        let root = std::env::temp_dir().join(format!("tuffite-init-{}", std::process::id()));
        for (name, namespace) in [
            ("hello-world", "helloworld"),
            ("123-demo", "app123demo"),
            ("tuffite", "apptuffite"),
        ] {
            let directory = root.join(name);
            init(&directory, None).unwrap();
            let script = fs::read_to_string(directory.join("build.rs")).unwrap();
            assert!(script.contains(
                ".javascript(\"generated\", tuffite::build::shell_api::OutputMode::Write)"
            ));
            assert!(!directory.join("generated").exists());
            let manifest = directory.join("tuffite.json");
            super::super::config::check(&manifest).unwrap();
            let value: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(manifest).unwrap()).unwrap();
            assert_eq!(value["build"]["shellApi"]["namespace"], namespace);
        }
        let invalid = root.join("invalid");
        assert!(init(&invalid, Some("---")).is_err());
        assert!(!invalid.exists());
        fs::remove_dir_all(root).unwrap();
    }
}
