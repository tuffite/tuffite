//! Typed, build-visible application declarations. These contain no live services.
use serde::Serialize;
use std::collections::BTreeMap;

/// Immutable application declarations shared by build scripts and application setup.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Configuration {
    pub app: Application,
    pub browser: Browser,
}
impl Configuration {
    #[must_use]
    pub fn new(schemes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            app: Application {
                schemes: schemes.into_iter().map(Into::into).collect(),
                ..Application::default()
            },
            ..Self::default()
        }
    }
    #[must_use]
    pub fn route(mut self, asset: Route) -> Self {
        self.app.assets.push(asset);
        self
    }
    #[must_use]
    pub fn window(mut self, window: Window) -> Self {
        self.app.windows.push(window);
        self
    }
    #[must_use]
    pub fn rule(mut self, rule: Rule) -> Self {
        self.browser.rules.push(rule);
        self
    }
    pub fn to_bytes(&self) -> crate::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(self).map_err(|e| crate::Error::new(e.to_string()))?;
        if bytes.len() > 1024 * 1024 {
            return Err(crate::Error::new("application configuration exceeds 1 MiB"));
        }
        Ok(bytes)
    }
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Application {
    pub schemes: Vec<String>,
    pub assets: Vec<Route>,
    pub windows: Vec<Window>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<Network>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub isolated_worlds: BTreeMap<String, IsolatedWorld>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Route {
    pub origins: Vec<String>,
    pub paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolver: Option<bool>,
}
impl Default for Route {
    fn default() -> Self {
        Self::new()
    }
}
impl Route {
    #[must_use]
    pub fn new() -> Self {
        Self {
            origins: vec![],
            paths: vec![],
            directory: None,
            file: None,
            resolver: None,
        }
    }
    #[must_use]
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origins.push(origin.into());
        self
    }
    #[must_use]
    pub fn origins(mut self, origins: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.origins.extend(origins.into_iter().map(Into::into));
        self
    }
    #[must_use]
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.paths.push(path.into());
        self
    }
    #[must_use]
    pub fn paths(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.paths.extend(paths.into_iter().map(Into::into));
        self
    }
    #[must_use]
    pub fn directory(mut self, directory: impl Into<String>) -> Self {
        self.directory = Some(directory.into());
        self.file = None;
        self.resolver = None;
        self
    }
    #[must_use]
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self.directory = None;
        self.resolver = None;
        self
    }
    #[must_use]
    pub fn resolver(mut self) -> Self {
        self.resolver = Some(true);
        self.file = None;
        self.directory = None;
        self
    }
}
/// One internal application scheme and its resource/action routes.
#[derive(Clone, Debug)]
pub struct Protocol {
    scheme: String,
    routes: Vec<Route>,
}
impl Protocol {
    #[must_use]
    pub fn new(scheme: impl Into<String>) -> Self {
        Self {
            scheme: scheme.into(),
            routes: vec![],
        }
    }
    #[must_use]
    pub fn route(mut self, route: Route) -> Self {
        self.routes.push(route);
        self
    }
}
impl Configuration {
    #[must_use]
    pub fn protocol(mut self, protocol: Protocol) -> Self {
        if !self.app.schemes.contains(&protocol.scheme) {
            self.app.schemes.push(protocol.scheme);
        }
        self.app.assets.extend(protocol.routes);
        self
    }
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Window {
    pub label: String,
    pub url: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focused: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decorations: Option<bool>,
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Network {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_bytes: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_http2: Option<bool>,
}
#[derive(Clone, Debug, Serialize)]
pub struct IsolatedWorld {
    pub directory: String,
    pub entry: String,
    pub origins: Vec<String>,
}
#[derive(Clone, Debug, Serialize)]
pub struct Browser {
    pub devtools: bool,
    pub rules: Vec<Rule>,
}
impl Default for Browser {
    fn default() -> Self {
        Self {
            devtools: true,
            rules: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
    pub origins: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shell_api: Vec<Grant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer: Option<Explorer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub csp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub devtools: Option<bool>,
}
impl Rule {
    #[must_use]
    pub fn new(origins: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            origins: origins.into_iter().map(Into::into).collect(),
            ..Self::default()
        }
    }
    #[must_use]
    pub fn grant(mut self, grant: impl Into<Grant>) -> Self {
        self.shell_api.push(grant.into());
        self
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Grant {
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<Scope>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub deny: Vec<Scope>,
}
impl From<&str> for Grant {
    fn from(method: &str) -> Self {
        Self {
            method: method.into(),
            allow: None,
            deny: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Path(String),
    Url(String),
    Window(String),
}
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Explorer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "executeJavaScript")]
    pub execute_javascript: Option<bool>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub behavior: BTreeMap<Action, Behavior>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Action {
    Navigation,
    Dialog,
    ExternalProtocol,
    NewWindow,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Browser,
    Delegate,
    Block,
}
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Behavior {
    pub mode: Mode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn declarations_serialize_native_names_without_package_identity() {
        let mut config = Configuration::new(["example", "example-resource"]);
        config = config.route(
            Route::new()
                .origin("example://main")
                .path("/*")
                .directory("ui"),
        );
        let mut rule = Rule::new(["example://main"]).grant("test.Commands.*");
        rule.explorer = Some(Explorer {
            execute_javascript: Some(true),
            behavior: [(
                Action::ExternalProtocol,
                Behavior {
                    mode: Mode::Delegate,
                    timeout_ms: Some(15000),
                },
            )]
            .into_iter()
            .collect(),
            ..Explorer::default()
        });
        config = config.rule(rule);
        let value: serde_json::Value = serde_json::from_slice(&config.to_bytes().unwrap()).unwrap();
        assert!(value["app"].get("identifier").is_none());
        assert_eq!(
            value["browser"]["rules"][0]["explorer"]["executeJavaScript"],
            true
        );
        assert_eq!(value["app"]["assets"][0]["directory"], "ui");
        assert_eq!(
            value["browser"]["rules"][0]["explorer"]["behavior"]["externalProtocol"]["timeoutMs"],
            15000
        );
        assert_eq!(
            value["browser"]["rules"][0]["shellApi"][0]["method"],
            "test.Commands.*"
        );
    }
    #[test]
    fn oversized_declarations_are_rejected_before_export() {
        let mut config = Configuration::new(["example"]);
        config = config.rule(Rule {
            csp: Some("x".repeat(1024 * 1024)),
            ..Rule::default()
        });
        assert!(config.to_bytes().is_err());
    }
}
