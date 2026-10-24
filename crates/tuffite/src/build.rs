//! Build-time validation and source generation for Tuffite applications.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

pub mod framework;
mod mojom;

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
#[serde(rename_all = "camelCase")]
struct BuildConfig {
    mojom_sources: Vec<PathBuf>,
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

fn render_mojom_error(error: &tuffite_mojom::Error) -> String {
    let mut output = String::new();
    miette::GraphicalReportHandler::new()
        .with_theme(miette::GraphicalTheme::unicode_nocolor())
        .with_width(100)
        .render_report(&mut output, error)
        .expect("writing a diagnostic to String cannot fail");
    output
}

fn read_shell_api_sources(
    manifest_path: &Path,
    definitions: &[PathBuf],
) -> Result<Vec<(PathBuf, String)>, String> {
    if definitions.is_empty() {
        return Err("build.mojomSources must not be empty".into());
    }
    tuffite_mojom::read_sources(
        manifest_path.parent().unwrap_or_else(|| Path::new(".")),
        definitions,
    )
    .map_err(|error| render_mojom_error(&error))
}

fn generate_bindings(
    manifest_path: &Path,
    facade_override: Option<Facade>,
) -> Result<(GeneratedBindings, Vec<PathBuf>), String> {
    let serialized = fs::read(manifest_path)
        .map_err(|error| format!("unable to read {}: {error}", manifest_path.display()))?;
    let manifest: Manifest = serde_json::from_slice(&serialized)
        .map_err(|error| format!("invalid {}: {error}", manifest_path.display()))?;
    let definitions = &manifest.build.mojom_sources;
    let facade = facade_override.unwrap_or_default();
    if manifest.app.identifier.is_empty() {
        return Err("app.identifier is invalid".into());
    }
    let sources = if facade == Facade::Framework {
        let parent = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        let first = definitions.first().ok_or("Framework contracts are empty")?;
        let first = parent
            .join(first)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let source_root = first.parent().ok_or("missing Framework source directory")?;
        let definitions = definitions
            .iter()
            .map(|name| {
                let path = parent
                    .join(name)
                    .canonicalize()
                    .map_err(|e| e.to_string())?;
                if path.parent() != Some(source_root) {
                    return Err("Framework contracts must share a source directory".to_owned());
                }
                Ok(PathBuf::from(path.file_name().ok_or("missing filename")?))
            })
            .collect::<Result<Vec<_>, String>>()?;
        // Framework contracts use Chromium source-root-relative imports, as
        // required by its native Mojom parser. Application imports remain
        // confined to their manifest directory.
        let import_root = source_root
            .ancestors()
            .nth(3)
            .ok_or("missing Chromium source root")?;
        let definitions = definitions
            .into_iter()
            .map(|name| {
                source_root
                    .join(name)
                    .strip_prefix(import_root)
                    .map(Path::to_path_buf)
                    .map_err(|e| e.to_string())
            })
            .collect::<Result<Vec<_>, _>>()?;
        tuffite_mojom::read_sources(import_root, &definitions)
            .map_err(|error| render_mojom_error(&error))?
            .into_iter()
            .map(|(path, source)| (import_root.join(path), source))
            .collect()
    } else {
        read_shell_api_sources(manifest_path, definitions)?
    };
    let root = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let inputs = sources
        .iter()
        .map(|(path, _)| root.join(path))
        .collect::<Vec<_>>();
    let bindings = if facade == Facade::Framework {
        mojom::generate_framework(&sources)?
    } else {
        mojom::generate(&sources, &manifest.app.identifier)?
    };
    Ok((bindings, inputs))
}

/// Package-owned IDL registry. Framework method IDs are unused by its raw
/// transport, but including their names lets native validation fail closed.
#[cfg(feature = "cli")]
pub(crate) fn security_command_ids(
    manifest_path: &Path,
) -> Result<BTreeMap<String, String>, String> {
    let (bindings, _) = generate_bindings(manifest_path, None)?;
    let mut ids = bindings.command_ids;
    let sources = vec![
        (
            PathBuf::from("common.mojom"),
            include_str!("../../../framework/shell_api/common.mojom").to_owned(),
        ),
        (
            PathBuf::from("dialog.mojom"),
            include_str!("../../../framework/shell_api/dialog.mojom").to_owned(),
        ),
        (
            PathBuf::from("path.mojom"),
            include_str!("../../../framework/shell_api/path.mojom").to_owned(),
        ),
        (
            PathBuf::from("fs.mojom"),
            include_str!("../../../framework/shell_api/fs.mojom").to_owned(),
        ),
        (
            PathBuf::from("storage.mojom"),
            include_str!("../../../framework/shell_api/storage.mojom").to_owned(),
        ),
        (
            PathBuf::from("log.mojom"),
            include_str!("../../../framework/shell_api/log.mojom").to_owned(),
        ),
        (
            PathBuf::from("network.mojom"),
            include_str!("../../../framework/shell_api/network.mojom").to_owned(),
        ),
        (
            PathBuf::from("utility.mojom"),
            include_str!("../../../framework/shell_api/utility.mojom").to_owned(),
        ),
        (
            PathBuf::from("remoting.mojom"),
            include_str!("../../../framework/shell_api/remoting.mojom").to_owned(),
        ),
        (
            PathBuf::from("window.mojom"),
            include_str!("../../../framework/shell_api/window.mojom").to_owned(),
        ),
    ];
    for method in tuffite_mojom::compile(&sources)
        .map_err(|error| render_mojom_error(&error))?
        .methods
    {
        ids.entry(format!("tuffite.{}.{}", method.interface, method.name))
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
    if entry.get("service").is_some() {
        return Err("service grants are no longer supported; use module.Interface.method".into());
    }
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
    let mut compiled = Vec::new();
    for method in methods {
        let mut scope_kinds = std::collections::BTreeSet::new();
        for field in ["allow", "deny"] {
            let Some(values) = entry.get(field) else {
                continue;
            };
            for value in values.as_array().ok_or("scope must be an array")? {
                validate_security_scope(method, value)?;
                scope_kinds.insert(value.as_object().unwrap().keys().next().unwrap());
            }
        }

        if scope_kinds.len() > 1 {
            return Err("a grant cannot mix scope kinds".into());
        }
        let mut grant = entry.clone();
        if grant.is_string() {
            grant = serde_json::Value::String(method.clone());
        } else {
            grant["method"] = serde_json::Value::String(method.clone());
        }
        compiled.push(grant);
    }
    Ok(compiled)
}

#[cfg(feature = "cli")]
fn validate_security_scope(method: &str, value: &serde_json::Value) -> Result<(), String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("{method}: expected scope object"))?;
    let kind = object
        .keys()
        .next()
        .map(String::as_str)
        .ok_or("empty scope object")?;
    let key = match kind {
        "path" => "path",
        "url" => "url",
        "window" => "window",
        _ => return Err(format!("{method}: unknown scope kind {kind}")),
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

    #[test]
    fn mojom_generation_errors_show_named_source_and_help() {
        let source = "module demo;\nstruct Data { string value }";
        let error = tuffite_mojom::compile(&[("broken.mojom".into(), source.into())]).unwrap_err();
        let rendered = render_mojom_error(&error);
        for expected in ["broken.mojom", "expected \";\"", "string value", "help:"] {
            assert!(rendered.contains(expected), "{rendered}");
        }
        assert!(
            !rendered.contains('\u{1b}'),
            "build output should not contain ANSI escapes"
        );
    }

    #[test]
    fn application_config_cannot_select_reserved_framework_codegen() {
        let root =
            std::env::temp_dir().join(format!("tuffite-reserved-module-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(
            root.join("api.mojom"),
            "module tuffite; interface App { Run() => (); };",
        )
        .unwrap();
        let manifest = root.join("tuffite.json");
        fs::write(&manifest, r#"{"app":{"identifier":"example"},"build":{"mojomSources":["api.mojom"],"shellApi":{"facade":"framework"}}}"#).unwrap();
        let error = shell_api::Generator::new()
            .manifest(&manifest)
            .generate()
            .unwrap_err();
        assert!(error.contains("reserved Mojom module"), "{error}");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(feature = "cli")]
    #[test]
    fn framework_registry_includes_remoting_methods() {
        let manifest =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/tuffite/tuffite.json");
        let ids = generate_bindings(&manifest, Some(Facade::Framework))
            .unwrap()
            .0
            .command_ids;
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
            assert!(ids.contains_key(&format!(
                "tuffite.Remoting.{}{}",
                method[..1].to_uppercase(),
                &method[1..]
            )));
        }
        assert!(validate_security_entry(&serde_json::json!("tuffite.Remoting.*"), &ids).is_ok());
    }

    #[cfg(feature = "cli")]
    #[test]
    fn namespace_group_scopes_are_preserved_for_api_validation() {
        let ids = [
            "tuffite.Window.Current",
            "tuffite.Window.Focus",
            "tuffite.Path.Directories",
        ]
        .into_iter()
        .map(|method| (method.to_owned(), "0".to_owned()))
        .collect();
        let grants = compile_security_entry(
            &serde_json::json!({
                "method": "tuffite.Window.*", "allow": [{"window": "*"}]
            }),
            &ids,
        )
        .unwrap();
        let current = grants
            .iter()
            .find(|grant| grant["method"] == "tuffite.Window.Current")
            .unwrap();
        assert_eq!(current["allow"], serde_json::json!([{"window":"*"}]));
        let focus = grants
            .iter()
            .find(|grant| grant["method"] == "tuffite.Window.Focus")
            .unwrap();
        assert_eq!(focus["allow"], serde_json::json!([{"window": "*"}]));
        assert!(
            compile_security_entry(
                &serde_json::json!({
                    "method": "tuffite.Window.*", "allow": [{"path": "data/**"}]
                }),
                &ids
            )
            .is_ok()
        );
        assert!(
            compile_security_entry(
                &serde_json::json!({
                    "method": "tuffite.Path.*", "allow": [{"path": "data/**"}]
                }),
                &ids
            )
            .is_ok()
        );
    }

    #[cfg(feature = "cli")]
    #[test]
    fn shell_api_patterns_and_scopes_are_module_validated() {
        let ids: BTreeMap<String, String> = [
            "tuffite.Path.Directories",
            "tuffite.Fs.ReadFile",
            "tuffite.Fs.WriteFile",
            "tuffite.Window.Focus",
        ]
        .into_iter()
        .map(|name| (name.into(), "0".into()))
        .collect();
        assert!(validate_security_entry(&serde_json::json!("*"), &ids).is_ok());
        assert!(!method_matches("tuffite.*.readFile", "tuffite.Fs.ReadFile"));
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"*","allow":[{"path":"data/**"}]}),
                &ids
            )
            .is_ok()
        );
        assert!(validate_security_entry(&serde_json::json!("tuffite.Path.*"), &ids).is_ok());
        assert!(validate_security_entry(&serde_json::json!("tuffite.unknown.*"), &ids).is_err());
        assert!(validate_security_entry(&serde_json::json!({"method":"tuffite.Fs.ReadFile","allow":[{"path":"resource/**"}]}), &ids).is_ok());
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"tuffite.Fs.*","allow":[{"path":"resource/**"}]}),
                &ids
            )
            .is_ok()
        );
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"tuffite.Path.*","allow":[]}),
                &ids
            )
            .is_ok()
        );
        for value in [
            serde_json::json!(true),
            serde_json::json!(null),
            serde_json::json!({"typo":"data/**"}),
        ] {
            assert!(
                validate_security_entry(
                    &serde_json::json!({"method":"tuffite.Fs.ReadFile","allow":[value]}),
                    &ids
                )
                .is_err()
            );
        }
        assert!(
            validate_security_entry(
                &serde_json::json!({"method":"tuffite.Window.Focus","allow":[{"window":"*"}]}),
                &ids
            )
            .is_ok()
        );
    }

    #[cfg(feature = "cli")]
    #[test]
    fn mojom_interface_permissions_use_canonical_names() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../playground");
        let root = std::env::temp_dir().join(format!(
            "tuffite-typed-route-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        for name in ["shell_api.mojom", "mojom_data.mojom"] {
            fs::copy(source.join(name), root.join(name)).unwrap();
        }
        let base: serde_json::Value =
            serde_json::from_slice(&fs::read(source.join("tuffite.json")).unwrap()).unwrap();
        let manifest = root.join("tuffite.json");
        fs::write(&manifest, serde_json::to_vec(&base).unwrap()).unwrap();
        assert!(security_command_ids(&manifest).is_ok());
        for good in [
            serde_json::json!("playground.Echo.RoundTrip"),
            serde_json::json!("playground.Echo.*"),
            serde_json::json!({"method":"playground.Aux.RoundTrip"}),
        ] {
            let mut config = base.clone();
            config["app"]["security"]["capabilities"][0]["shellApi"] = serde_json::json!([good]);
            fs::write(&manifest, serde_json::to_vec(&config).unwrap()).unwrap();
            assert!(security_command_ids(&manifest).is_ok());
        }
        for bad in [
            serde_json::json!("playground.echo.roundTrip"),
            serde_json::json!("playground.Echo.Missing"),
            serde_json::json!({"method":"playground.Echo.RoundTrip","service":"legacy"}),
        ] {
            let mut config = base.clone();
            config["app"]["security"]["capabilities"][0]["shellApi"] = serde_json::json!([bad]);
            fs::write(&manifest, serde_json::to_vec(&config).unwrap()).unwrap();
            assert!(security_command_ids(&manifest).is_err());
        }
        assert_eq!(root.parent(), Some(std::env::temp_dir().as_path()));
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(feature = "cli")]
    #[test]
    fn mojom_permissions_resolve_idl_endpoints_without_application_globals() {
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../playground/tuffite.json");
        let ids = security_command_ids(&manifest).unwrap();
        let endpoint = "playground.Commands.EchoMojom";
        assert!(ids.contains_key(endpoint));
        assert_eq!(
            compile_security_entry(&serde_json::json!(endpoint), &ids).unwrap(),
            vec![serde_json::json!(endpoint)]
        );
        assert!(
            compile_security_entry(&serde_json::json!("playground.Commands.Missing"), &ids)
                .is_err()
        );
        let scoped = serde_json::json!({"method":endpoint,"allow":[{"path":"data/**"}]});
        assert_eq!(compile_security_entry(&scoped, &ids).unwrap(), vec![scoped]);
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
    "mojomSources": [
        "shell_api.mojom"
      ]
  }
}"#,
        )
        .unwrap();
        fs::write(
            root.join("shell_api.mojom"),
            "module tuffite; interface Commands { Ping() => (string result); };",
        )
        .unwrap();
        let error = shell_api::Generator::new()
            .manifest(&manifest)
            .generate()
            .unwrap_err();
        assert!(error.contains("reserved Mojom module"));
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
        let definition = root.join("shell_api.mojom");
        fs::write(
            &manifest,
            r#"{
  "app": {
    "identifier": "com.example.builder"
  },
  "build": {
    "mojomSources": [
        "shell_api.mojom"
      ]
  }
}"#,
        )
        .unwrap();
        fs::write(
            &definition,
            "module api; interface Commands { Ping() => (string result); };",
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
        let build_generation = shell_api::Generator::new()
            .manifest(&manifest)
            .rust(&generated, OutputMode::Verify)
            .javascript(&generated, OutputMode::Verify)
            .generate_for_build()
            .unwrap();
        assert!(build_generation.rerun_paths.contains(&manifest));
        assert!(build_generation.rerun_paths.contains(&definition));
        assert!(build_generation.rerun_paths.contains(&javascript));
        assert!(generated.join("bindings.rs").is_file());
        assert!(generated.join("mojom.rs").is_file());
        assert!(generated.join("index.d.ts").is_file());
        for name in [
            "mojom.js",
            "mojom.d.ts",
            ".tuffite-javascript-outputs.json",
            ".tuffite-declaration-outputs.json",
        ] {
            assert!(!generated.join(name).exists(), "{name}");
        }
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
    "mojomSources": [
        "alpha.mojom",
        "beta.mojom"
      ]
  }
}"#,
        )
        .unwrap();
        fs::write(
            root.join("alpha.mojom"),
            "module api; interface Alpha { Ping() => (string result); };",
        )
        .unwrap();
        fs::write(
            root.join("beta.mojom"),
            "module api; interface Beta { Pong() => (double result); };",
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
        assert!(generated.declarations.contains("ApiApi"));
        fs::remove_dir_all(root).unwrap();
    }
}
