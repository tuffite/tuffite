#[cfg(test)]
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

#[derive(clap::Args, Debug)]
pub(super) struct ManifestOptions {
    /// Application configuration; otherwise search the current directory and its parents.
    #[arg(value_name = "MANIFEST", conflicts_with = "manifest")]
    path: Option<PathBuf>,
    /// Explicit application configuration (overrides automatic discovery).
    #[arg(long, value_name = "MANIFEST")]
    manifest: Option<PathBuf>,
}

impl ManifestOptions {
    pub(super) fn resolve(&self) -> Result<PathBuf, String> {
        resolve_manifest(self.manifest.as_deref().or(self.path.as_deref()))
    }
}

pub(super) fn resolve_manifest(requested: Option<&Path>) -> Result<PathBuf, String> {
    discover_manifest(requested)?.ok_or_else(|| {
        "no tuffite.json found in the current directory or its parents; pass --manifest <PATH>"
            .to_owned()
    })
}

pub(super) fn discover_manifest(requested: Option<&Path>) -> Result<Option<PathBuf>, String> {
    if let Some(path) = requested {
        return super::filesystem::canonical_file(path).map(Some);
    }
    let current =
        std::env::current_dir().map_err(super::filesystem::io_error("read current directory"))?;
    find_manifest(&current)
}

fn find_manifest(start: &Path) -> Result<Option<PathBuf>, String> {
    for directory in start.ancestors() {
        let path = directory.join("tuffite.json");
        match fs::symlink_metadata(&path) {
            Ok(_) => return super::filesystem::canonical_file(&path).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("inspect {}: {error}", path.display())),
        }
    }
    Ok(None)
}

pub fn check(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let root: Value = serde_json::from_slice(&bytes)
        .map_err(|error| format!("parse {}: {error}", path.display()))?;
    let root = object(&root, "$")?;
    known_fields(
        root,
        "$",
        &["$schema", "productName", "app", "build", "bundle"],
    )?;
    if let Some(name) = root.get("productName") {
        validate_product_name(nonempty_string(Some(name), "productName")?)?;
    }
    let build = required(root, "build", "build")?;
    check_build(build)?;
    check_app(required(root, "app", "app")?)?;
    if let Some(bundle) = root.get("bundle") {
        check_bundle(bundle)?;
    }
    Ok(Value::Object(root.clone()))
}

pub(super) fn validate_product_name(value: &str) -> Result<(), String> {
    if value.len() > 128 {
        return Err("productName must not exceed 128 UTF-8 bytes".to_owned());
    }
    validate_output_name(value).map_err(|error| error.replace("output name", "productName"))
}

pub(super) fn validate_output_name(value: &str) -> Result<(), String> {
    let stem = value
        .split('.')
        .next()
        .unwrap_or(value)
        .to_ascii_uppercase();
    if value.trim().is_empty()
        || value.len() > 240
        || value.ends_with(['.', ' '])
        || value.chars().any(|c| {
            c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
        })
        || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit())
    {
        return Err("output name must be a portable filename of 1..=240 bytes without reserved characters or names".to_owned());
    }
    Ok(())
}

#[cfg(test)]
fn check_schemes(value: &Value) -> Result<Vec<&str>, String> {
    let schemes = array(value, "app.schemes")?;
    let mut unique = BTreeSet::new();
    if schemes.is_empty() || schemes.len() > 16 {
        return Err("app.schemes: expected 1..=16 schemes".to_owned());
    }
    schemes
        .iter()
        .map(|value| {
            let scheme = nonempty_string(Some(value), "app.schemes")?;
            if scheme.len() > 64
                || !scheme.as_bytes()[0].is_ascii_lowercase()
                || !scheme.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'+' | b'-' | b'.')
                })
                || matches!(
                    scheme,
                    "http"
                        | "https"
                        | "ws"
                        | "wss"
                        | "ftp"
                        | "file"
                        | "filesystem"
                        | "blob"
                        | "data"
                        | "javascript"
                        | "about"
                        | "chrome"
                        | "chrome-untrusted"
                        | "chrome-extension"
                        | "devtools"
                        | "view-source"
                        | "vbscript"
                        | "shell"
                        | "ms-msdt"
                        | "search-ms"
                )
                || !unique.insert(scheme)
            {
                return Err(
                    "app.schemes: expected unique lowercase non-reserved schemes".to_owned(),
                );
            }
            Ok(scheme)
        })
        .collect::<Result<Vec<_>, String>>()
}

fn check_app(value: &Value) -> Result<(), String> {
    let app = object(value, "app")?;
    known_fields(app, "app", &["identifier"])?;
    nonempty_string(app.get("identifier"), "app.identifier")?;
    Ok(())
}

#[cfg(test)]
fn check_runtime_app(value: &Value) -> Result<(), String> {
    let app = object(value, "app")?;
    known_fields(
        app,
        "app",
        &[
            "identifier",
            "schemes",
            "windows",
            "assets",
            "network",
            "isolatedWorlds",
        ],
    )?;
    let schemes = check_schemes(required(app, "schemes", "app.schemes")?)?;
    if let Some(worlds) = app.get("isolatedWorlds") {
        let worlds = object(worlds, "app.isolatedWorlds")?;
        if worlds.len() > 32 {
            return Err("app.isolatedWorlds: at most 32 worlds".to_owned());
        }
        for (name, value) in worlds {
            if name.is_empty()
                || name.len() > 128
                || !name.as_bytes()[0].is_ascii_alphabetic()
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.'))
            {
                return Err("app.isolatedWorlds: invalid name".to_owned());
            }
            let path = format!("app.isolatedWorlds.{name}");
            let value = object(value, &path)?;
            known_fields(value, &path, &["entry", "directory", "origins"])?;
            if let Some(directory) = value.get("directory") {
                super::filesystem::safe_relative_path(
                    nonempty_string(Some(directory), &format!("{path}.directory"))?,
                    &format!("{path}.directory"),
                )?;
            }
            let entry = nonempty_string(value.get("entry"), &format!("{path}.entry"))?;
            super::filesystem::safe_relative_path(entry, &format!("{path}.entry"))?;
            let origins = array(
                required(value, "origins", &path)?,
                &format!("{path}.origins"),
            )?;
            if origins.is_empty() || origins.len() > 32 {
                return Err(format!("{path}.origins: expected 1..=32 origins"));
            }
            for origin in origins {
                check_origin_pattern(nonempty_string(Some(origin), &path)?, &path, false)?;
            }
        }
    }
    nonempty_string(app.get("identifier"), "app.identifier")?;
    if let Some(assets) = app.get("assets") {
        check_assets(assets, &schemes)?;
    }
    if let Some(network) = app.get("network") {
        check_network(network)?;
    }
    if let Some(windows) = app.get("windows") {
        check_windows(windows)?;
        for window in windows.as_array().unwrap() {
            let scheme = window["url"]
                .as_str()
                .unwrap()
                .split("://")
                .next()
                .unwrap_or("");
            if !schemes.contains(&scheme) {
                return Err("app.windows URLs must use a declared app.schemes entry".to_owned());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn check_network(value: &Value) -> Result<(), String> {
    let path = "app.network";
    let network = object(value, path)?;
    known_fields(
        network,
        path,
        &["disableHttp2", "maxResponseBytes", "timeoutSeconds"],
    )?;
    if let Some(value) = network.get("disableHttp2") {
        boolean(Some(value), "app.network.disableHttp2")?;
    }
    for (name, maximum) in [
        ("maxResponseBytes", 5 * 1024 * 1024),
        ("timeoutSeconds", 300),
    ] {
        if let Some(value) = network.get(name) {
            let value = integer(Some(value), &format!("{path}.{name}"))?;
            if value == 0 || value > maximum {
                return Err(format!("{path}.{name}: expected integer in 1..={maximum}"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn check_windows(value: &Value) -> Result<(), String> {
    let windows = array(value, "app.windows")?;
    if windows.is_empty() {
        return Err("app.windows must not be empty".to_owned());
    }
    let mut labels = BTreeSet::new();
    for (index, window) in windows.iter().enumerate() {
        let path = format!("app.windows[{index}]");
        let window = object(window, &path)?;
        known_fields(
            window,
            &path,
            &[
                "label",
                "url",
                "title",
                "x",
                "y",
                "width",
                "height",
                "minWidth",
                "minHeight",
                "maxWidth",
                "maxHeight",
                "focused",
                "decorations",
            ],
        )?;
        let label = nonempty_string(window.get("label"), &format!("{path}.label"))?;
        if !labels.insert(label) {
            return Err(format!("{path}.label: duplicate window label"));
        }
        nonempty_string(window.get("url"), &format!("{path}.url"))?;
        for name in ["width", "height"] {
            let field = format!("{path}.{name}");
            let value = integer(window.get(name), &field)?;
            if value == 0 || value > 16_384 {
                return Err(format!("{field}: expected integer in 1..=16384"));
            }
        }
        for (dimension, min, max) in [
            ("width", "minWidth", "maxWidth"),
            ("height", "minHeight", "maxHeight"),
        ] {
            let read = |key: &str, default| -> Result<u64, String> {
                let value = match window.get(key) {
                    Some(v) => integer(Some(v), &format!("{path}.{key}"))?,
                    None => default,
                };
                if value == 0 || value > 16384 {
                    return Err(format!("{path}.{key}: expected integer in 1..=16384"));
                }
                Ok(value)
            };
            let low = read(min, 1)?;
            let high = read(max, 16384)?;
            let size = window[dimension].as_u64().unwrap();
            if low > high || size < low || size > high {
                return Err(format!("{path}: inconsistent {dimension} constraints"));
            }
        }
        if window.contains_key("x") != window.contains_key("y") {
            return Err(format!("{path}: x and y must be declared together"));
        }
        for name in ["focused", "decorations"] {
            if let Some(value) = window.get(name) {
                boolean(Some(value), &format!("{path}.{name}"))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn origins_overlap(left: &str, right: &str) -> bool {
    let sample = |origin: &str| url::Url::parse(&origin.replacen("*.", "sample.", 1)).unwrap();
    let matches = |pattern: &str, candidate: &url::Url| {
        let base = sample(pattern);
        if base.scheme() != candidate.scheme() || base.port() != candidate.port() {
            return false;
        }
        let host = candidate.host_str().unwrap().to_ascii_lowercase();
        let expected = base.host_str().unwrap().to_ascii_lowercase();
        if pattern.contains("://*.") {
            host.ends_with(&expected[6..]) && host.len() > expected.len() - 6
        } else {
            host == expected
        }
    };
    matches(left, &sample(right)) || matches(right, &sample(left))
}

#[cfg(test)]
fn check_assets(value: &Value, schemes: &[&str]) -> Result<(), String> {
    let mut routes: Vec<(String, String)> = vec![];
    for (index, route) in array(value, "app.assets")?.iter().enumerate() {
        let path = format!("app.assets[{index}]");
        let route = object(route, &path)?;
        known_fields(route, &path, &["origins", "paths", "file", "directory"])?;
        strings(
            required(route, "origins", &path)?,
            &format!("{path}.origins"),
            false,
        )?;
        strings(
            required(route, "paths", &path)?,
            &format!("{path}.paths"),
            false,
        )?;
        if route.contains_key("file") == route.contains_key("directory") {
            return Err(format!("{path}: specify exactly one of file/directory"));
        }
        let source = nonempty_string(route.get("file").or_else(|| route.get("directory")), &path)?;
        if source.contains(['\\', ':'])
            || std::path::Path::new(source)
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err(format!(
                "{path}: asset source must be a safe project-relative path"
            ));
        }
        for origin in route["origins"].as_array().unwrap() {
            let origin = origin.as_str().unwrap();
            check_origin_pattern(origin, &path, false)?;
            if !origin
                .split("://")
                .next()
                .is_some_and(|scheme| schemes.contains(&scheme))
            {
                return Err(format!(
                    "{path}: assets must use a declared app.schemes entry"
                ));
            }
            for pattern in route["paths"].as_array().unwrap() {
                let pattern = pattern.as_str().unwrap();
                if pattern == "/tuffite.js" {
                    return Err(format!("{path}: reserved framework route"));
                }
                let prefix = pattern.strip_suffix('*').unwrap_or(pattern);
                if !prefix.starts_with('/')
                    || prefix.contains(['*', '?', '#', '\\', '%'])
                    || prefix
                        .split('/')
                        .any(|segment| matches!(segment, "." | ".."))
                    || (pattern.ends_with('*') && !prefix.ends_with('/'))
                    || (route.contains_key("directory") && !pattern.ends_with('*'))
                {
                    return Err(format!(
                        "{path}: paths must be exact paths or a terminal /* prefix; directories require /*"
                    ));
                }
                if routes
                    .iter()
                    .any(|(o, p)| p == pattern && origins_overlap(origin, o))
                {
                    return Err(format!("{path}: ambiguous asset route {pattern}"));
                }
                routes.push((origin.to_owned(), pattern.to_owned()));
            }
        }
    }
    Ok(())
}

fn check_mojom_sources(value: &Value) -> Result<(), String> {
    let path = "build.mojomSources";
    strings(value, path, false)?;
    if value.as_array().unwrap().iter().any(|value| {
        Path::new(value.as_str().unwrap())
            .extension()
            .and_then(|s| s.to_str())
            != Some("mojom")
    }) {
        return Err("build.mojomSources: only .mojom contracts are supported".into());
    }
    Ok(())
}

#[cfg(test)]
fn check_origin_pattern(pattern: &str, path: &str, allow_self: bool) -> Result<(), String> {
    if pattern == "*" || (allow_self && pattern == "self") {
        return Ok(());
    }
    let fail = || format!("{path}: expected an origin with an optional leading *. hostname");
    if pattern.chars().any(char::is_whitespace) || pattern.contains([',', '\\']) {
        return Err(fail());
    }
    let (scheme, authority) = pattern.split_once("://").ok_or_else(fail)?;
    let candidate = if let Some(host) = authority.strip_prefix("*.") {
        format!("{scheme}://wildcard.{host}")
    } else {
        pattern.to_owned()
    };
    if candidate.contains('*') {
        return Err(fail());
    }
    let url = url::Url::parse(&candidate).map_err(|_| fail())?;
    if url.host_str().is_none()
        || url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(url.path(), "" | "/")
        || matches!(scheme, "file" | "data" | "blob" | "about" | "javascript")
    {
        return Err(fail());
    }
    Ok(())
}

#[cfg(test)]
fn check_browser(value: &Value) -> Result<(), String> {
    let browser = object(value, "browser")?;
    known_fields(browser, "browser", &["rules", "devtools"])?;
    if let Some(value) = browser.get("devtools") {
        boolean(Some(value), "browser.devtools")?;
    }
    check_origin_rules(required(browser, "rules", "browser.rules")?)
}

#[cfg(test)]
fn check_origin_rules(value: &Value) -> Result<(), String> {
    let path = "browser.rules";
    let mut identifiers = BTreeSet::new();
    for (index, rule) in array(value, path)?.iter().enumerate() {
        let path = format!("{path}[{index}]");
        let rule = object(rule, &path)?;
        known_fields(
            rule,
            &path,
            &[
                "identifier",
                "description",
                "origins",
                "shellApi",
                "explorer",
                "devtools",
                "csp",
            ],
        )?;
        if let Some(value) = rule.get("identifier") {
            let identifier = nonempty_string(Some(value), &format!("{path}.identifier"))?;
            if !identifier
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'$' | b'-'))
                || !identifiers.insert(identifier)
            {
                return Err(format!("{path}.identifier: expected unique identifier"));
            }
        }
        if let Some(value) = rule.get("csp") {
            check_csp(value, &format!("{path}.csp"))?;
        }
        if let Some(value) = rule.get("devtools") {
            boolean(Some(value), &format!("{path}.devtools"))?;
        }
        if let Some(description) = rule.get("description") {
            nonempty_string(Some(description), &format!("{path}.description"))?;
        }
        strings(
            required(rule, "origins", &format!("{path}.origins"))?,
            &format!("{path}.origins"),
            false,
        )?;
        for origin in rule["origins"].as_array().unwrap() {
            check_origin_pattern(origin.as_str().unwrap(), &format!("{path}.origins"), false)?;
        }
        if let Some(value) = rule.get("shellApi") {
            for (i, entry) in array(value, &format!("{path}.shellApi"))?
                .iter()
                .enumerate()
            {
                let field = format!("{path}.shellApi[{i}]");
                let method = if entry.is_string() {
                    nonempty_string(Some(entry), &field)?
                } else {
                    let object = object(entry, &field)?;
                    known_fields(object, &field, &["method", "allow", "deny"])?;
                    for key in ["allow", "deny"] {
                        if let Some(value) = object.get(key) {
                            array(value, &format!("{field}.{key}"))?;
                        }
                    }
                    nonempty_string(object.get("method"), &format!("{field}.method"))?
                };
                let exact = method.strip_suffix(".*").unwrap_or(method);
                if method != "*"
                    && ((!method.ends_with(".*") && !exact.contains('.'))
                        || exact.split('.').any(|part| {
                            part.is_empty()
                                || !part
                                    .bytes()
                                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'$')
                        }))
                {
                    return Err(format!(
                        "{field}: expected *, qualified method or trailing .* pattern"
                    ));
                }
            }
        }
        if let Some(value) = rule.get("explorer") {
            let field = format!("{path}.explorer");
            let explorer = object(value, &field)?;
            known_fields(
                explorer,
                &field,
                &["create", "executeJavaScript", "behavior"],
            )?;
            for (key, value) in explorer {
                if key != "behavior" {
                    boolean(Some(value), &format!("{field}.{key}"))?;
                }
            }
            if let Some(value) = explorer.get("behavior") {
                check_explorer_behavior(value, &format!("{field}.behavior"))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
fn check_csp(value: &Value, field: &str) -> Result<(), String> {
    let csp = nonempty_string(Some(value), field)?;
    if csp.len() > 4096 || csp.contains(['\r', '\n', '\0']) {
        return Err(format!("{field}: invalid header value"));
    }
    Ok(())
}

#[cfg(test)]
fn check_explorer_behavior(value: &Value, field: &str) -> Result<(), String> {
    let behavior = object(value, field)?;
    known_fields(
        behavior,
        field,
        &["navigation", "dialog", "externalProtocol", "newWindow"],
    )?;
    for (operation, value) in behavior {
        let field = format!("{field}.{operation}");
        let entry = object(value, &field)?;
        known_fields(entry, &field, &["mode", "timeoutMs"])?;
        let mode = nonempty_string(entry.get("mode"), &format!("{field}.mode"))?;
        if !matches!(mode, "browser" | "delegate" | "block") {
            return Err(format!("{field}.mode: unknown behavior mode"));
        }
        if let Some(timeout) = entry.get("timeoutMs")
            && (operation == "navigation"
                || mode != "delegate"
                || !timeout.as_u64().is_some_and(|n| (1..=300_000).contains(&n)))
        {
            return Err(format!(
                "{field}.timeoutMs: requires a delegated controller and 1..=300000"
            ));
        }
    }
    Ok(())
}

fn check_build(value: &Value) -> Result<(), String> {
    let build = object(value, "build")?;
    known_fields(
        build,
        "build",
        &[
            "mojomSources",
            "devPackage",
            "devUrl",
            "beforeDevCommand",
            "beforeBuildCommand",
            "beforePackageCommand",
            "devtoolsPort",
            "profile",
            "framework",
        ],
    )?;
    if let Some(framework) = build.get("framework") {
        let config: crate::build::framework::ReleaseConfig =
            serde_json::from_value(framework.clone())
                .map_err(|error| format!("build.framework: {error}"))?;
        config.validate()?;
    }
    check_mojom_sources(required(build, "mojomSources", "build.mojomSources")?)?;
    for hook in ["beforeBuildCommand", "beforePackageCommand"] {
        if let Some(command) = build.get(hook) {
            let command: super::hooks::BuildCommand = serde_json::from_value(command.clone())
                .map_err(|error| format!("build.{hook}: {error}"))?;
            command.validate(hook)?;
        }
    }
    if let Some(port) = build.get("devtoolsPort") {
        let port = integer(Some(port), "build.devtoolsPort")?;
        if port == 0 || port > u64::from(u16::MAX) {
            return Err("build.devtoolsPort: expected integer in 1..=65535".to_owned());
        }
    }
    Ok(())
}

fn check_bundle(value: &Value) -> Result<(), String> {
    let bundle = object(value, "bundle")?;
    known_fields(bundle, "bundle", &["executable", "resources", "icons"])?;
    if let Some(icons) = bundle.get("icons") {
        let icons = object(icons, "bundle.icons")?;
        known_fields(icons, "bundle.icons", &["macos", "windows"])?;
        for (platform, extension) in [("macos", "icns"), ("windows", "ico")] {
            if let Some(value) = icons.get(platform) {
                let field = format!("bundle.icons.{platform}");
                let value = nonempty_string(Some(value), &field)?;
                let path = super::filesystem::safe_relative_path(value, &field)?;
                if path.extension().and_then(std::ffi::OsStr::to_str) != Some(extension) {
                    return Err(format!("{field}: requires a .{extension} file"));
                }
            }
        }
    }
    if let Some(resources) = bundle.get("resources") {
        strings(resources, "bundle.resources", true)?;
    }
    Ok(())
}

fn known_fields(value: &Map<String, Value>, path: &str, allowed: &[&str]) -> Result<(), String> {
    for name in value.keys() {
        if !allowed.contains(&name.as_str()) {
            let suggestion = allowed
                .iter()
                .min_by_key(|candidate| edit_distance(name, candidate));
            let hint = suggestion
                .filter(|candidate| edit_distance(name, candidate) <= 3)
                .map_or(String::new(), |candidate| {
                    format!("; did you mean {candidate:?}?")
                });
            return Err(format!("{path}.{name}: unknown field{hint}"));
        }
    }
    Ok(())
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut costs: Vec<usize> = (0..=right.len()).collect();
    for (i, a) in left.bytes().enumerate() {
        let mut diagonal = i;
        costs[0] = i + 1;
        for (j, b) in right.bytes().enumerate() {
            let old = costs[j + 1];
            costs[j + 1] = if a == b {
                diagonal
            } else {
                1 + diagonal.min(costs[j]).min(old)
            };
            diagonal = old;
        }
    }
    costs[right.len()]
}

fn required<'a>(
    value: &'a Map<String, Value>,
    name: &str,
    path: &str,
) -> Result<&'a Value, String> {
    value
        .get(name)
        .ok_or_else(|| format!("{path}: required field is missing"))
}

fn object<'a>(value: &'a Value, path: &str) -> Result<&'a Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{path}: expected object"))
}

fn array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{path}: expected array"))
}

fn integer(value: Option<&Value>, path: &str) -> Result<u64, String> {
    value
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("{path}: expected non-negative integer"))
}

#[cfg(test)]
fn boolean(value: Option<&Value>, path: &str) -> Result<bool, String> {
    value
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("{path}: expected boolean"))
}

fn nonempty_string<'a>(value: Option<&'a Value>, path: &str) -> Result<&'a str, String> {
    value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{path}: expected non-empty string"))
}

fn strings(value: &Value, path: &str, empty_allowed: bool) -> Result<(), String> {
    let values = array(value, path)?;
    if !empty_allowed && values.is_empty() {
        return Err(format!("{path}: expected non-empty array"));
    }
    for (index, value) in values.iter().enumerate() {
        nonempty_string(Some(value), &format!("{path}[{index}]"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_world_packages_require_scoped_origins_and_safe_entries() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/runtime.json")).unwrap();
        let mut app = fixture["app"].clone();
        assert!(check_runtime_app(&app).is_ok());
        for name in ["io.example.visual-blocks", "extension_a", "world"] {
            app["isolatedWorlds"] =
                serde_json::json!({name: {"entry":"card.js","origins":["playground://main"]}});
            assert!(check_runtime_app(&app).is_ok(), "{name}");
        }
        for name in ["", "9bad", "app::World", "_bad", "bad/name"] {
            app["isolatedWorlds"] =
                serde_json::json!({name: {"entry":"card.js","origins":["playground://main"]}});
            assert!(check_runtime_app(&app).is_err(), "{name}");
        }
        for definition in [
            serde_json::json!({"entry":"../escape.js","origins":["playground://main"]}),
            serde_json::json!({"entry":"card.js","origins":[]}),
            serde_json::json!({"entry":"card.js","origins":["playground://main"],"source":"no"}),
        ] {
            app["isolatedWorlds"] = serde_json::json!({"weather": definition});
            assert!(check_runtime_app(&app).is_err());
        }
    }

    #[test]
    fn discovery_uses_nearest_manifest_and_explicit_paths_never_fall_back() {
        let root = std::env::temp_dir().join(format!("tuffite-discovery-{}", std::process::id()));
        fs::create_dir_all(root.join("nested/ui/src")).unwrap();
        let outer = root.join("tuffite.json");
        fs::write(&outer, "{}").unwrap();
        assert_eq!(
            find_manifest(&root.join("nested/ui/src")).unwrap(),
            Some(fs::canonicalize(&outer).unwrap())
        );
        let inner = root.join("nested/tuffite.json");
        fs::write(&inner, "invalid JSON").unwrap();
        let found = find_manifest(&root.join("nested/ui/src")).unwrap().unwrap();
        assert_eq!(found, fs::canonicalize(&inner).unwrap());
        assert!(check(&found).unwrap_err().contains("parse"));
        assert_eq!(
            resolve_manifest(Some(&outer)).unwrap(),
            fs::canonicalize(&outer).unwrap()
        );
        assert!(resolve_manifest(Some(&root.join("missing.json"))).is_err());
        fs::remove_file(&inner).unwrap();
        fs::create_dir(&inner).unwrap();
        assert!(
            find_manifest(&root.join("nested/ui/src"))
                .unwrap_err()
                .contains("expected a file")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_configuration_has_no_schema_version() {
        let root =
            std::env::temp_dir().join(format!("tuffite-source-version-{}", std::process::id()));
        super::super::workflow::init(&root, Some("example")).unwrap();
        let path = root.join("tuffite.json");
        let mut source = check(&path).unwrap();
        assert!(source.get("schemaVersion").is_none());
        assert_eq!(source["productName"], "example");
        crate::build::shell_api::Generator::new()
            .manifest(&path)
            .generate()
            .unwrap();
        for field in ["assets", "schemes", "windows", "network", "isolatedWorlds"] {
            let mut invalid = source.clone();
            invalid["app"][field] = serde_json::json!([]);
            fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
            assert!(
                check(&path).unwrap_err().contains("unknown field"),
                "{field}"
            );
        }
        let mut invalid = source.clone();
        invalid["browser"] = serde_json::json!({"rules": []});
        fs::write(&path, serde_json::to_vec(&invalid).unwrap()).unwrap();
        assert!(check(&path).unwrap_err().contains("unknown field"));
        source["schemaVersion"] = 2.into();
        fs::write(&path, serde_json::to_vec(&source).unwrap()).unwrap();
        assert!(check(&path).unwrap_err().contains("unknown field"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn product_names_support_unicode_and_spaces_without_unsafe_filenames() {
        for name in ["My App", "小应用", "A&B", "Example $App"] {
            assert!(validate_product_name(name).is_ok(), "{name}");
        }
        for name in [
            "",
            " ",
            ".",
            "..",
            "../app",
            "app/test",
            "app\\test",
            "app:beta",
            "app\n",
            "app.",
            "CON",
            "aux.exe",
            "LPT1",
        ] {
            assert!(validate_product_name(name).is_err(), "{name}");
        }
    }

    #[test]
    fn asset_routes_reject_ambiguity_and_traversal_but_allow_shared_origins() {
        let shared = serde_json::json!([
            {"origins": ["demo://*.artifact", "demo://*.preview"], "paths": ["/assets/*"], "directory": "ui/assets"},
            {"origins": ["demo://main"], "paths": ["/home"], "file": "ui/index.html"}
        ]);
        assert!(check_assets(&shared, &["demo"]).is_ok());
        let collision = serde_json::json!([
            {"origins": ["demo://*.artifact"], "paths": ["/home"], "file": "ui/a.html"},
            {"origins": ["demo://a.artifact"], "paths": ["/home"], "file": "ui/b.html"}
        ]);
        assert!(
            check_assets(&collision, &["demo"])
                .unwrap_err()
                .contains("ambiguous")
        );
        let traversal = serde_json::json!([{"origins": ["demo://main"], "paths": ["/assets/*"], "directory": "../secret"}]);
        assert!(check_assets(&traversal, &["demo"]).is_err());
    }

    #[test]
    fn window_constraints_are_local_and_old_sections_are_rejected() {
        assert!(
            check_windows(&serde_json::json!([
                {"label":"a", "url":"demo://a/home", "width":400, "height":300, "maxWidth":500},
                {"label":"b", "url":"demo://b/home", "width":900, "height":600, "minWidth":800}
            ]))
            .is_ok()
        );
        assert!(check_windows(&serde_json::json!([{ "label":"a", "url":"demo://a/home", "width":400, "height":300, "minWidth":500 }])).is_err());
        assert!(
            check_app(&serde_json::json!({"identifier":"demo", "capabilities":{}}))
                .unwrap_err()
                .contains("unknown field")
        );
        assert!(check_network(&serde_json::json!({"allow":["*"]})).is_err());
    }

    #[test]
    fn reports_nested_type_and_unknown_field_paths() {
        let error = check_origin_rules(&serde_json::json!([{
            "identifier": "app-shell", "origins": ["myapp://main"],
            "explorer": {"create": "true"}
        }]))
        .unwrap_err();
        assert_eq!(error, "browser.rules[0].explorer.create: expected boolean");

        let mut object = Map::new();
        object.insert("devPackge".to_owned(), Value::String("dist".to_owned()));
        let error = check_build(&Value::Object(object)).unwrap_err();
        assert!(error.contains("did you mean \"devPackage\""), "{error}");
    }

    #[test]
    fn devtools_policy_accepts_booleans_and_rejects_strings() {
        for value in [serde_json::json!(false), serde_json::json!(true)] {
            assert!(
                check_origin_rules(&serde_json::json!([{
                    "identifier": "inspection", "origins": ["https://example.com"],
                    "devtools": value
                }]))
                .is_ok()
            );
        }
        let error = check_origin_rules(&serde_json::json!([{
            "identifier": "inspection", "origins": ["https://example.com"],
            "devtools": "false"
        }]))
        .unwrap_err();
        assert!(error.contains("devtools: expected boolean"), "{error}");
    }

    #[test]
    fn devtools_master_switch_is_typed_and_utilities_are_removed() {
        let fixture: Value =
            serde_json::from_str(include_str!("../../tests/fixtures/runtime.json")).unwrap();
        for value in [serde_json::json!(true), serde_json::json!(false)] {
            let mut manifest = fixture.clone();
            manifest["browser"]["devtools"] = value;
            assert!(check_browser(&manifest["browser"]).is_ok());
        }
        for value in [
            serde_json::json!("false"),
            serde_json::json!(null),
            serde_json::json!({}),
        ] {
            let mut manifest = fixture.clone();
            manifest["browser"]["devtools"] = value;
            assert!(
                check_browser(&manifest["browser"])
                    .unwrap_err()
                    .contains("browser.devtools")
            );
        }
        let mut app = fixture["app"].clone();
        app["utilities"] = serde_json::json!({});
        assert!(check_runtime_app(&app).unwrap_err().contains("utilities"));
    }

    #[test]
    fn browser_rules_validate_behavior_parameters_and_reject_typos() {
        let mut rule = serde_json::json!([{
            "origins": ["demo://main"],
            "explorer": {"behavior": {
                "externalProtocol": {"mode": "delegate", "timeoutMs": 15000}
            }}
        }]);
        assert!(check_origin_rules(&rule).is_ok());
        rule[0]["explorer"]["behavior"]["externalProtocol"]["mode"] = "browser".into();
        assert!(check_origin_rules(&rule).unwrap_err().contains("timeoutMs"));
        rule[0]["explorer"]["behavior"]["externalProtocol"]["mode"] = "delegate".into();
        rule[0]["explorer"]["behavior"]["externalProtocol"]["timeoutMs"] = 0.into();
        assert!(check_origin_rules(&rule).is_err());
        rule[0]["explorer"]["behavior"] =
            serde_json::json!({"externalProtcol": {"mode": "browser"}});
        assert!(
            check_origin_rules(&rule)
                .unwrap_err()
                .contains("externalProtcol")
        );
        rule[0]["explorer"]["behavior"] = serde_json::json!({"dialog": {"mode": "allow"}});
        assert!(check_origin_rules(&rule).unwrap_err().contains("mode"));
    }

    #[test]
    fn bundle_icons_are_optional_safe_platform_paths() {
        assert!(check_bundle(&serde_json::json!({})).is_ok());
        assert!(
            check_bundle(
                &serde_json::json!({"icons":{"macos":"icons/app.icns", "windows":"icons/app.ico"}})
            )
            .is_ok()
        );
        for icons in [
            serde_json::json!({"macos":"../private.icns"}),
            serde_json::json!({"macos":"app.png"}),
            serde_json::json!({"windows":""}),
            serde_json::json!({"linux":"app.png"}),
        ] {
            assert!(check_bundle(&serde_json::json!({"icons":icons})).is_err());
        }
    }
}
