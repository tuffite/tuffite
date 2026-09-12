//! Build-time validation and source generation for Tuffite applications.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub mod framework;
mod typescript;

const DOMAIN: &[u8] = b"tuffite.command.v1\0";

/// ShellAPI source generation and output verification.
pub mod shell_api;
use shell_api::{Facade, GeneratedBindings, Output, OutputMode};

/// Result of preparing both the native Framework and generated `ShellAPI` code.
#[derive(Debug)]
pub struct BuildOutput {
    pub framework: framework::Distribution,
    pub shell_api: shell_api::GeneratedBindings,
}

/// Composes Framework resolution and `ShellAPI` code generation for application
/// build scripts.
#[derive(Debug, Default)]
pub struct Builder {
    steps: Vec<Step>,
}

/// Supported application build components, accepted by [`Builder::add`].
#[derive(Debug)]
pub enum Step {
    Framework(framework::Resolver),
    ShellApi(shell_api::Generator),
}
impl From<framework::Resolver> for Step {
    fn from(value: framework::Resolver) -> Self {
        Self::Framework(value)
    }
}
impl From<shell_api::Generator> for Step {
    fn from(value: shell_api::Generator) -> Self {
        Self::ShellApi(value)
    }
}

impl Builder {
    #[must_use]
    pub const fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Adds a Framework resolver or `ShellAPI` generator. Each may occur once;
    /// Framework resolution always precedes source generation.
    #[must_use]
    // Build composition follows Builder.add(component), rather than arithmetic.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, step: impl Into<Step>) -> Self {
        self.steps.push(step.into());
        self
    }

    pub fn build(self) -> Result<BuildOutput, String> {
        let mut resolver = None;
        let mut generator = None;
        for step in self.steps {
            match step {
                Step::Framework(value) => {
                    if resolver.replace(value).is_some() {
                        return Err("duplicate Framework Resolver".into());
                    }
                }
                Step::ShellApi(value) => {
                    if generator.replace(value).is_some() {
                        return Err("duplicate ShellAPI Generator".into());
                    }
                }
            }
        }
        let resolver = resolver.ok_or("tuffite::build::Builder requires a Framework Resolver")?;
        let codegen = generator.ok_or("tuffite::build::Builder requires ShellAPI Generator")?;
        let framework = resolver.resolve()?;
        let generated = codegen.generate_for_build()?;
        for name in framework::ENV_VARS {
            println!("cargo:rerun-if-env-changed={name}");
        }
        for path in generated.rerun_paths {
            println!("cargo:rerun-if-changed={}", path.display());
        }
        Ok(BuildOutput {
            framework,
            shell_api: generated.bindings,
        })
    }
}

fn emit_files(
    directory: Option<&Output>,
    files: &BTreeMap<String, String>,
    description: &str,
    plan_name: &str,
    manifest: &Path,
) -> Result<(), String> {
    let Some(directory) = directory else {
        return Ok(());
    };
    let plan_path = directory.path.join(plan_name);
    let previous = fs::read(&plan_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Vec<String>>(&bytes).ok())
        .unwrap_or_default();
    for (name, contents) in files {
        let output = Output {
            path: directory.path.join(name),
            mode: directory.mode,
        };
        emit(
            Some(&output),
            contents,
            &format!("{description} {name}"),
            manifest,
        )?;
    }
    let planned = files.keys().cloned().collect::<Vec<_>>();
    match directory.mode {
        OutputMode::Write => {
            for stale in previous.iter().filter(|name| !files.contains_key(*name)) {
                let path = directory.path.join(stale);
                if path.is_file() {
                    let contents = fs::read_to_string(&path).map_err(|error| {
                        format!("inspect stale generated file {}: {error}", path.display())
                    })?;
                    if !contents.starts_with("// Generated from ") {
                        return Err(format!(
                            "refusing to remove unmanaged stale output {}",
                            path.display()
                        ));
                    }
                    fs::remove_file(&path).map_err(|error| {
                        format!("remove stale generated file {}: {error}", path.display())
                    })?;
                }
            }
            let contents = serde_json::to_string_pretty(&planned)
                .map_err(|error| format!("serialize generation output plan: {error}"))?
                + "\n";
            write_if_changed(&plan_path, &contents)?;
        }
        OutputMode::Verify => {
            if !previous.is_empty() && previous != planned {
                return Err(format!(
                    "generated output plan {} is stale; run `tuff build {}`",
                    plan_path.display(),
                    manifest.display()
                ));
            }
        }
    }
    Ok(())
}

fn emit(
    output: Option<&Output>,
    contents: &str,
    description: &str,
    manifest: &Path,
) -> Result<(), String> {
    let Some(output) = output else {
        return Ok(());
    };
    match output.mode {
        OutputMode::Write => {
            write_if_changed(&output.path, contents)?;
            Ok(())
        }
        OutputMode::Verify => {
            let current = fs::read_to_string(&output.path).map_err(|error| {
                format!(
                    "unable to verify {description} {}: {error}",
                    output.path.display()
                )
            })?;
            if current == contents {
                Ok(())
            } else {
                Err(format!(
                    "{description} {} is stale; run `tuff build {}`",
                    output.path.display(),
                    manifest.display()
                ))
            }
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    app: AppConfig,
    build: BuildConfig,
}

#[derive(Deserialize)]
struct AppConfig {
    identifier: String,
}

#[derive(Deserialize)]
struct BuildConfig {
    #[serde(rename = "shellApi")]
    shell_api: ShellApi,
}

#[derive(Deserialize)]
struct ShellApi {
    namespace: String,
    definitions: Vec<PathBuf>,
    #[serde(default)]
    facade: Facade,
}

fn command_id(name: &str) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(DOMAIN);
    hasher.update(name.as_bytes());
    let digest = hasher.finalize();
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest.as_bytes()[..8]);
    u64::from_le_bytes(bytes)
}

fn read_shell_api_sources(
    manifest_path: &Path,
    definitions: &[PathBuf],
) -> Result<Vec<(PathBuf, String)>, String> {
    if definitions.is_empty() {
        return Err("build.shellApi.definitions must not be empty".to_owned());
    }
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    definitions
        .iter()
        .map(|declaration| {
            if !declaration
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.ends_with(".d.ts"))
            {
                return Err(format!(
                    "ShellAPI declaration must use the .d.ts extension: {}",
                    declaration.display()
                ));
            }
            let path = root.join(declaration);
            let source = fs::read_to_string(&path)
                .map_err(|error| format!("unable to read {}: {error}", path.display()))?;
            Ok((declaration.clone(), source))
        })
        .collect()
}

fn generate_bindings(
    manifest_path: &Path,
    facade_override: Option<Facade>,
    cpp_include_root: &str,
) -> Result<(GeneratedBindings, Vec<PathBuf>), String> {
    let serialized = fs::read(manifest_path)
        .map_err(|error| format!("unable to read {}: {error}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&serialized)
        .map_err(|error| format!("invalid {}: {error}", manifest_path.display()))?;
    let shell_api = &manifest.build.shell_api;
    let facade = facade_override.unwrap_or(shell_api.facade);
    if manifest.app.identifier.is_empty()
        || !is_identifier(&shell_api.namespace)
        || (shell_api.namespace == "tuffite" && facade != Facade::Framework)
    {
        return Err("app.identifier or shellApi.namespace is invalid".into());
    }
    let sources = read_shell_api_sources(manifest_path, &shell_api.definitions)?;
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let inputs = sources
        .iter()
        .map(|(path, _)| root.join(path))
        .collect::<Vec<_>>();
    if cpp_include_root.is_empty()
        || cpp_include_root.starts_with('/')
        || cpp_include_root.contains("..")
        || cpp_include_root.contains('\\')
    {
        return Err("C++ include root must be a safe source-relative path".to_owned());
    }
    let bindings = typescript::generate_with_cpp_include_root(
        &sources,
        &shell_api.namespace,
        &manifest.app.identifier,
        &shell_api.namespace,
        facade,
        cpp_include_root,
    )?;
    Ok((bindings, inputs))
}

/// Package-owned IDL registry. Framework method IDs are unused by its raw
/// transport, but including their names lets native validation fail closed.
#[cfg(feature = "cli")]
pub(crate) fn security_command_ids(
    manifest_path: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let (bindings, _) = generate_bindings(manifest_path, None, "tuffite/framework/shell_api")?;
    let mut ids = bindings.command_ids;
    let sources = vec![
        (
            PathBuf::from("common.d.ts"),
            include_str!("../../../framework/shell_api/common.d.ts").to_owned(),
        ),
        (
            PathBuf::from("dialog.d.ts"),
            include_str!("../../../framework/shell_api/dialog.d.ts").to_owned(),
        ),
        (
            PathBuf::from("path.d.ts"),
            include_str!("../../../framework/shell_api/path.d.ts").to_owned(),
        ),
        (
            PathBuf::from("fs.d.ts"),
            include_str!("../../../framework/shell_api/fs.d.ts").to_owned(),
        ),
        (
            PathBuf::from("storage.d.ts"),
            include_str!("../../../framework/shell_api/storage.d.ts").to_owned(),
        ),
        (
            PathBuf::from("log.d.ts"),
            include_str!("../../../framework/shell_api/log.d.ts").to_owned(),
        ),
        (
            PathBuf::from("network.d.ts"),
            include_str!("../../../framework/shell_api/network.d.ts").to_owned(),
        ),
        (
            PathBuf::from("utility.d.ts"),
            include_str!("../../../framework/shell_api/utility.d.ts").to_owned(),
        ),
        (
            PathBuf::from("remoting.d.ts"),
            include_str!("../../../framework/shell_api/remoting.d.ts").to_owned(),
        ),
        (
            PathBuf::from("window.d.ts"),
            include_str!("../../../framework/shell_api/window.d.ts").to_owned(),
        ),
    ];
    for command in typescript::command_paths(&sources, "tuffite")? {
        ids.entry(command)
            .or_insert_with(|| "0000000000000000".to_owned());
    }
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(manifest_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    ids.insert(
        "tuffite.devtools.inspectElement".into(),
        "0000000000000000".into(),
    );
    if let Some(capabilities) = value
        .pointer("/app/security/capabilities")
        .and_then(serde_json::Value::as_array)
    {
        for capability in capabilities {
            if let Some(entries) = capability
                .get("shellApi")
                .and_then(serde_json::Value::as_array)
            {
                for entry in entries {
                    validate_security_entry(entry, &ids)?;
                }
            }
        }
    }
    Ok(ids)
}

#[cfg(feature = "cli")]
pub(crate) fn method_matches(pattern: &str, method: &str) -> bool {
    pattern == "*"
        || pattern
            .strip_suffix(".*")
            .map_or(pattern == method, |prefix| {
                method.starts_with(&format!("{prefix}."))
            })
}

#[cfg(feature = "cli")]
fn validate_security_entry(
    entry: &serde_json::Value,
    ids: &BTreeMap<String, String>,
) -> Result<(), String> {
    compile_security_entry(entry, ids).map(|_| ())
}

#[cfg(feature = "cli")]
pub(crate) fn compile_security_entry(
    entry: &serde_json::Value,
    ids: &BTreeMap<String, String>,
) -> Result<Vec<serde_json::Value>, String> {
    let pattern = entry
        .as_str()
        .or_else(|| entry.get("method").and_then(serde_json::Value::as_str))
        .ok_or("shellApi entry requires method")?;
    let methods: Vec<_> = ids
        .keys()
        .filter(|name| method_matches(pattern, name))
        .collect();
    if methods.is_empty() {
        return Err(format!(
            "ShellAPI method pattern matches no registered method: {pattern}"
        ));
    }
    let registry: serde_json::Value = serde_json::from_str(include_str!(
        "../../../framework/shell_api/scope_registry.json"
    ))
    .map_err(|e| e.to_string())?;
    if (entry.get("allow").is_some() || entry.get("deny").is_some())
        && !methods.iter().any(|method| registry.get(*method).is_some())
    {
        return Err(format!("{pattern} does not support scopes"));
    }
    let mut compiled = Vec::new();
    for method in methods {
        let skip_scopes = pattern.ends_with(".*") && registry.get(method).is_none();
        for field in ["allow", "deny"] {
            if skip_scopes {
                continue;
            }
            let Some(values) = entry.get(field) else {
                continue;
            };
            let descriptor = registry
                .get(method)
                .ok_or_else(|| format!("{method} does not support scopes"))?;
            for value in values.as_array().ok_or("scope must be an array")? {
                validate_security_scope(method, descriptor, value)?;
            }
        }

        let mut grant = entry.clone();
        if grant.is_string() {
            grant = serde_json::Value::String(method.clone());
        } else {
            grant["method"] = serde_json::Value::String(method.clone());
            if skip_scopes {
                let fields = grant.as_object_mut().expect("validated grant object");
                fields.remove("allow");
                fields.remove("deny");
            }
        }
        compiled.push(grant);
    }
    Ok(compiled)
}

#[cfg(feature = "cli")]
fn validate_security_scope(
    method: &str,
    descriptor: &serde_json::Value,
    value: &serde_json::Value,
) -> Result<(), String> {
    let kind = descriptor["kind"].as_str().unwrap();
    let object = value
        .as_object()
        .ok_or_else(|| format!("{method}: expected {kind} scope object"))?;
    let key = match kind {
        "path" => "path",
        "url" => "url",
        "window" => "window",
        _ => unreachable!(),
    };
    if object.len() != 1 {
        return Err(format!("{method}: scope must contain only {key}"));
    }
    let text = object
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("{method}: scope requires string {key}"))?;
    match kind {
        "path" => {
            let base = text.split('/').next().unwrap();
            if descriptor["write"] == true && base == "resource" {
                return Err(format!(
                    "{method}: resource is read-only; use data, cache or temp for write scopes (split read and write grants when using a wildcard method)"
                ));
            }
            if !matches!(base, "resource" | "data" | "cache" | "temp" | "*")
                || text.contains(['\\', ':', '?', '\0'])
                || text.split('/').any(|p| p == ".." || p == ".")
            {
                return Err(format!("{method}: invalid path scope {text}"));
            }
        }
        "url" => {
            if text != "*" {
                let candidate = text.replacen("*.", "sample.", 1).replace('*', "sample");
                let url = url::Url::parse(&candidate)
                    .map_err(|_| format!("{method}: expected absolute HTTP(S) URL scope"))?;
                if !matches!(url.scheme(), "http" | "https")
                    || url.host_str().is_none()
                    || !url.username().is_empty()
                    || url.password().is_some()
                    || url.query().is_some()
                    || url.fragment().is_some()
                {
                    return Err(format!("{method}: invalid URL scope"));
                }
            }
        }
        "window" if !matches!(text, "current" | "others" | "*") => {
            return Err(format!(
                "{method}: window scope must be current, others or *"
            ));
        }
        _ => {}
    }
    Ok(())
}

fn write_if_changed(path: &Path, contents: &str) -> Result<bool, String> {
    if fs::read_to_string(path).ok().as_deref() == Some(contents) {
        return Ok(false);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("unable to create {}: {error}", parent.display()))?;
    }
    fs::write(path, contents)
        .map_err(|error| format!("unable to write {}: {error}", path.display()))?;
    Ok(true)
}

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    matches!(characters.next(), Some(first) if first.is_ascii_alphabetic() || first == '_' || first == '$')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '$'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "cli")]
    #[test]
    fn framework_registry_includes_remoting_methods() {
        let manifest =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/tuffite/tuffite.json");
        let ids = security_command_ids(&manifest).unwrap();
        for method in [
            "capabilities",
            "start",
            "processSignal",
            "takeSignals",
            "sendInput",
            "stop",
            "stats",
            "capture",
        ] {
            assert!(ids.contains_key(&format!("tuffite.remoting.{method}")));
        }
        assert!(validate_security_entry(&serde_json::json!("tuffite.remoting.*"), &ids).is_ok());
    }

    #[cfg(feature = "cli")]
    #[test]
    fn namespace_group_scopes_compile_only_for_resource_methods() {
        let ids = [
            "tuffite.window.current",
            "tuffite.window.focus",
            "tuffite.path.directories",
        ]
        .into_iter()
        .map(|method| (method.to_owned(), "0".to_owned()))
        .collect();
        let grants = compile_security_entry(
            &serde_json::json!({
                "method": "tuffite.window.*", "allow": [{"window": "*"}]
            }),
            &ids,
        )
        .unwrap();
        let current = grants
            .iter()
            .find(|grant| grant["method"] == "tuffite.window.current")
            .unwrap();
        assert!(current.get("allow").is_none());
        let focus = grants
            .iter()
            .find(|grant| grant["method"] == "tuffite.window.focus")
            .unwrap();
        assert_eq!(focus["allow"], serde_json::json!([{"window": "*"}]));
        assert!(
            compile_security_entry(
                &serde_json::json!({
                    "method": "tuffite.window.*", "allow": [{"path": "data/**"}]
                }),
                &ids
            )
            .is_err()
        );
        assert!(
            compile_security_entry(
                &serde_json::json!({
                    "method": "tuffite.path.*", "allow": [{"path": "data/**"}]
                }),
                &ids
            )
            .is_err()
        );
    }

    #[cfg(feature = "cli")]
    #[test]
    fn shell_api_patterns_and_scopes_are_module_validated() {
        let ids: BTreeMap<String, String> = [
            "tuffite.path.directories",
            "tuffite.fs.readFile",
            "tuffite.fs.writeFile",
            "tuffite.window.focus",
        ]
        .into_iter()
        .map(|name| (name.into(), "0".into()))
        .collect();
        assert!(validate_security_entry(&serde_json::json!("*"), &ids).is_ok());
        assert!(!method_matches("tuffite.*.readFile", "tuffite.fs.readFile"));
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"*","allow":[{"path":"data/**"}]}),
                &ids
            )
            .is_err()
        );
        assert!(validate_security_entry(&serde_json::json!("tuffite.path.*"), &ids).is_ok());
        assert!(validate_security_entry(&serde_json::json!("tuffite.unknown.*"), &ids).is_err());
        assert!(validate_security_entry(&serde_json::json!({"method":"tuffite.fs.readFile","allow":[{"path":"resource/**"}]}), &ids).is_ok());
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"tuffite.fs.*","allow":[{"path":"resource/**"}]}),
                &ids
            )
            .is_err()
        );
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"tuffite.path.*","allow":[]}),
                &ids
            )
            .is_err()
        );
        for value in [
            serde_json::json!(true),
            serde_json::json!(null),
            serde_json::json!({"typo":"data/**"}),
        ] {
            assert!(
                validate_security_entry(
                    &serde_json::json!({"method":"tuffite.fs.readFile","allow":[value]}),
                    &ids
                )
                .is_err()
            );
        }
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"tuffite.window.focus","allow":[{"window":"*"}]}),
                &ids
            )
            .is_ok()
        );
    }

    #[test]
    fn application_builder_requires_both_build_phases() {
        let missing_framework = Builder::new().build().unwrap_err();
        assert!(missing_framework.contains("Framework Resolver"));

        let missing_shell_api = Builder::new()
            .add(framework::Resolver::new())
            .build()
            .unwrap_err();
        assert!(missing_shell_api.contains("ShellAPI Generator"));
    }

    #[test]
    fn application_builder_rejects_duplicate_components_before_running_them() {
        let error = Builder::new()
            .add(shell_api::Generator::new())
            .add(shell_api::Generator::new())
            .build()
            .unwrap_err();
        assert!(error.contains("duplicate ShellAPI Generator"));
        let error = Builder::new()
            .add(framework::Resolver::new())
            .add(framework::Resolver::new())
            .build()
            .unwrap_err();
        assert!(error.contains("duplicate Framework Resolver"));
    }

    #[test]
    fn command_id_is_stable() {
        // Golden value derived from DOMAIN = b"tuffite.command.v1\0".
        assert_eq!(
            command_id("com.tuffite.demo:some_thing"),
            0x1cfa_4aba_ac72_7a5b
        );
    }

    #[test]
    fn identifiers_reject_prototype_paths() {
        assert!(is_identifier("some_thing"));
        assert!(!is_identifier("__proto__.polluted"));
        assert!(!is_identifier("has-hyphen"));
    }

    #[test]
    fn reserved_framework_global_is_rejected() {
        let root = std::env::temp_dir().join(format!(
            "tuffite-builder-reserved-global-{}-{}",
            std::process::id(),
            command_id(module_path!())
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let manifest = root.join("tuffite.json");
        fs::write(
            &manifest,
            r#"{
  "app": {
    "identifier": "com.example.reserved"
  },
  "build": {
    "shellApi": {
      "namespace": "tuffite",
      "definitions": [
        "shell_api.d.ts"
      ]
    }
  }
}"#,
        )
        .unwrap();
        fs::write(
            root.join("shell_api.d.ts"),
            "export declare namespace tuffite { export function ping(): Promise<string>; }",
        )
        .unwrap();
        let error = shell_api::Generator::new()
            .manifest(&manifest)
            .generate()
            .unwrap_err();
        assert!(error.contains("shellApi.namespace is invalid"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn builder_writes_and_verifies_each_output() {
        let root = std::env::temp_dir().join(format!(
            "tuffite-builder-{}-{}",
            std::process::id(),
            command_id(module_path!())
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let manifest = root.join("tuffite.json");
        let definition = root.join("shell_api.d.ts");
        fs::write(
            &manifest,
            r#"{
  "app": {
    "identifier": "com.example.builder"
  },
  "build": {
    "shellApi": {
      "namespace": "api",
      "definitions": [
        "shell_api.d.ts"
      ]
    }
  }
}"#,
        )
        .unwrap();
        fs::write(
            &definition,
            "export declare namespace api { export function ping(): Promise<string>; }",
        )
        .unwrap();
        let javascript = root.join("generated/index.js");
        let generated = root.join("generated");
        let write_generation = shell_api::Generator::new()
            .manifest(&manifest)
            .rust(&generated, OutputMode::Write)
            .javascript(&generated, OutputMode::Write)
            .generate_for_build()
            .unwrap();
        assert!(write_generation.rerun_paths.contains(&javascript));
        assert!(
            write_generation
                .rerun_paths
                .contains(&generated.join("shell_api.js"))
        );
        let build_generation = shell_api::Generator::new()
            .manifest(&manifest)
            .rust(&generated, OutputMode::Verify)
            .javascript(&generated, OutputMode::Verify)
            .generate_for_build()
            .unwrap();
        assert!(build_generation.rerun_paths.contains(&manifest));
        assert!(build_generation.rerun_paths.contains(&definition));
        assert!(build_generation.rerun_paths.contains(&javascript));
        assert!(
            build_generation
                .rerun_paths
                .contains(&generated.join("shell_api.js"))
        );
        assert!(
            build_generation
                .rerun_paths
                .contains(&generated.join(".tuffite-javascript-outputs.json"))
        );
        assert!(generated.join("bindings.rs").is_file());
        assert!(generated.join("shell_api.rs").is_file());
        assert!(generated.join("shell_api.js").is_file());
        assert!(generated.join("shell_api.d.ts").is_file());
        let stale = generated.join("removed.js");
        fs::write(&stale, "// Generated from removed.d.ts. Do not edit.\n").unwrap();
        fs::write(
            generated.join(".tuffite-javascript-outputs.json"),
            "[\"removed.js\", \"shell_api.js\"]\n",
        )
        .unwrap();
        shell_api::Generator::new()
            .manifest(&manifest)
            .javascript(&generated, OutputMode::Write)
            .generate()
            .unwrap();
        assert!(!stale.exists());
        fs::write(&javascript, "stale").unwrap();
        let error = shell_api::Generator::new()
            .manifest(&manifest)
            .javascript(&generated, OutputMode::Verify)
            .generate()
            .unwrap_err();
        assert!(error.contains("is stale"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn builder_merges_multiple_declaration_files() {
        let root = std::env::temp_dir().join(format!(
            "tuffite-builder-multiple-{}-{}",
            std::process::id(),
            command_id(module_path!())
        ));
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        fs::create_dir_all(&root).unwrap();
        let manifest = root.join("tuffite.json");
        fs::write(
            &manifest,
            r#"{
  "app": {
    "identifier": "com.example.multiple"
  },
  "build": {
    "shellApi": {
      "namespace": "api",
      "definitions": [
        "alpha.d.ts",
        "beta.d.ts"
      ]
    }
  }
}"#,
        )
        .unwrap();
        fs::write(
            root.join("alpha.d.ts"),
            "export declare namespace api { export namespace alpha { export function ping(): Promise<string>; } }",
        )
        .unwrap();
        fs::write(
            root.join("beta.d.ts"),
            "export declare namespace api { export namespace beta { export function pong(): Promise<number>; } }",
        )
        .unwrap();
        let generated = shell_api::Generator::new()
            .manifest(&manifest)
            .generate()
            .unwrap();
        assert!(generated.rust.contains("pub mod alpha"));
        assert!(generated.rust.contains("pub mod beta"));
        assert!(generated.javascript.contains("alpha.ping"));
        assert!(generated.javascript.contains("beta.pong"));
        assert!(generated.declarations.contains(" & "));
        fs::remove_dir_all(root).unwrap();
    }
}
