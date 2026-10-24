//! Borrowed and owned ESM sources for the native V8 utility backend.

use std::borrow::Cow;

/// Owned V8 startup options, configured before Browser-side provider construction.
/// Framework supplies `entry.mjs` and the named service's manifest configuration.
#[derive(Debug)]
pub struct Builder {
    configuration: super::Builder,
    entry: String,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            configuration: super::Builder::default(),
            entry: "entry.mjs".to_owned(),
        }
    }
}

impl Builder {
    /// Selects a relative module path or an absolute HTTPS/file URL.
    #[must_use]
    pub fn entry(mut self, entry: impl Into<String>) -> Self {
        self.entry = entry.into();
        self
    }

    /// Replaces service-local startup data without granting native capabilities.
    #[must_use]
    pub fn configuration(mut self, configuration: impl Into<Vec<u8>>) -> Self {
        self.configuration = self.configuration.configuration(configuration);
        self
    }

    /// Reads the current service startup data.
    #[must_use]
    pub fn configuration_bytes(&self) -> &[u8] {
        self.configuration.configuration_bytes()
    }

    pub(super) fn finish(
        self,
        scheme: &str,
        name: &str,
    ) -> crate::ApplicationResult<(Vec<u8>, String)> {
        let entry = entry_url(scheme, name, &self.entry)
            .map_err(|error| crate::ApplicationError::new(error.to_string()))?;
        Ok((self.configuration.finish()?, entry))
    }
}

#[cfg(test)]
mod builder_tests {
    use super::Builder;

    #[test]
    fn default_and_overridden_entry_are_canonicalized_at_freeze() {
        let (configuration, entry) = Builder::default().finish("myapp", "parser").unwrap();
        assert!(configuration.is_empty());
        assert_eq!(entry, "myapp://parser.utility/entry.mjs");
        let (configuration, entry) = Builder::default()
            .configuration(b"manifest".to_vec())
            .entry("extension/main.mjs")
            .finish("myapp", "parser")
            .unwrap();
        assert_eq!(configuration, b"manifest");
        assert_eq!(entry, "myapp://parser.utility/extension/main.mjs");
    }

    #[test]
    fn invalid_entry_or_oversized_configuration_cannot_be_frozen() {
        for entry in [
            "",
            "../escape.mjs",
            "http://example.com/main.mjs",
            "myapp://other.utility/main.mjs",
        ] {
            assert!(
                Builder::default()
                    .entry(entry)
                    .finish("myapp", "parser")
                    .is_err(),
                "{entry}"
            );
        }
        assert!(
            Builder::default()
                .configuration(vec![0; 64 * 1024 + 1])
                .finish("myapp", "parser")
                .is_err()
        );
    }
}

/// Failure while obtaining resources or validating a module graph.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Module(#[from] crate::JavaScriptExtensionError),
    #[error(transparent)]
    Resource(#[from] std::io::Error),
}

/// Owned module request, scoped to one native isolate session.
#[derive(Clone, Debug)]
pub struct ModuleRequest {
    url: String,
    session_id: u64,
    deadline: std::time::Instant,
    stop: crate::StopSignal,
}
impl ModuleRequest {
    pub(crate) fn new(url: String, session_id: u64, stop: crate::StopSignal) -> Self {
        Self {
            url,
            session_id,
            deadline: std::time::Instant::now() + crate::lifecycle::LIFECYCLE_TIMEOUT,
            stop,
        }
    }
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
    /// Correlation identity only; never an authorization credential.
    #[must_use]
    pub fn session_id(&self) -> u64 {
        self.session_id
    }
    #[must_use]
    pub fn deadline(&self) -> std::time::Instant {
        self.deadline
    }
    #[must_use]
    pub fn stop_signal(&self) -> crate::StopSignal {
        self.stop.clone()
    }
}

/// One logical module path and its UTF-8 ECMAScript source.
///
/// Static sources remain borrowed until the reply; dynamically generated or
/// loaded strings are owned. Native copies each response into its session cache.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleSource {
    /// Canonical resource URL returned to the resolver.
    pub name: Cow<'static, str>,
    /// Borrowed embedded source or an owned runtime-generated string.
    pub source: Cow<'static, str>,
}

impl ModuleSource {
    /// Accepts static string literals, embedded sources, or owned strings.
    #[must_use]
    pub fn new(name: impl Into<Cow<'static, str>>, source: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
        }
    }
}

/// Reads the application scheme, which native validates against window URLs.
pub(crate) fn app_scheme(configuration: &[u8]) -> Option<String> {
    let manifest = serde_json::from_slice::<serde_json::Value>(configuration).ok()?;
    let entry = manifest.pointer("/app/windows/0/url")?.as_str()?;
    Some(url::Url::parse(entry).ok()?.scheme().to_owned())
}

/// Builds the canonical entry URL, or accepts an explicit HTTPS/file entry.
pub(crate) fn entry_url(scheme: &str, name: &str, entry: &str) -> Result<String, Error> {
    let value = if url::Url::parse(entry).is_ok() {
        entry.to_owned()
    } else {
        crate::javascript_extension::validate_module_name(entry)?;
        url::Url::parse(&format!(
            "{scheme}://{}.utility/",
            name.to_ascii_lowercase()
        ))
        .and_then(|base| base.join(entry))
        .map_err(|_| Error::Module(crate::JavaScriptExtensionError::InvalidModule))?
        .to_string()
    };
    validate_url(name, &value, Some(scheme))?;
    Ok(value)
}

/// Validates a canonical URL and the service's custom-scheme namespace.
/// HTTPS and file resources are still resolved by the registered Rust provider.
pub(crate) fn validate_url(
    name: &str,
    value: &str,
    app_scheme: Option<&str>,
) -> Result<url::Url, Error> {
    let invalid = || Error::Module(crate::JavaScriptExtensionError::InvalidModule);
    let parsed = url::Url::parse(value).map_err(|_| invalid())?;
    if value.len() > 512
        || parsed.as_str() != value
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.cannot_be_a_base()
    {
        return Err(invalid());
    }
    match parsed.scheme() {
        "https" => {
            if parsed.host_str().is_none() {
                return Err(invalid());
            }
        }
        "file" => {
            if parsed.port().is_some() {
                return Err(invalid());
            }
        }
        scheme => {
            if app_scheme.is_some_and(|expected| expected != scheme)
                || matches!(
                    scheme,
                    "http" | "data" | "javascript" | "ftp" | "ws" | "wss"
                )
                || parsed.host_str()
                    != Some(format!("{}.utility", name.to_ascii_lowercase()).as_str())
                || parsed.port().is_some()
            {
                return Err(invalid());
            }
        }
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    #[test]
    fn scheme_comes_from_entry_window_without_shell_api_namespace() {
        let config = br#"{"app":{"windows":[{"url":"assetscheme://main/index.html"}],"shellApi":{"commandIds":{"other.module.Commands.Run":"0000000000000001"}}}}"#;
        assert_eq!(super::app_scheme(config).as_deref(), Some("assetscheme"));
        assert_eq!(super::app_scheme(br#"{"app":{"windows":[]}}"#), None);
    }
    use super::*;

    #[test]
    fn resource_urls_use_the_app_scheme_and_support_standard_urls() {
        let entry = entry_url("myapp", "Parser_1", "extension/entry.mjs").unwrap();
        assert_eq!(entry, "myapp://parser_1.utility/extension/entry.mjs");
        let base = validate_url("Parser_1", &entry, Some("myapp")).unwrap();
        assert_eq!(
            base.join("../sdk/runtime.mjs").unwrap().as_str(),
            "myapp://parser_1.utility/sdk/runtime.mjs"
        );
        for value in [
            "https://example.com/module?version=1#x",
            "file:///tmp/sdk/runtime.mjs",
            "myapp://parser_1.utility/file%20name.mjs",
        ] {
            assert!(validate_url("Parser_1", value, Some("myapp")).is_ok());
        }
        for invalid in [
            "entry.mjs",
            "http://example.com/a.mjs",
            "myapp://other.utility/entry.mjs",
            "myapp://user@parser_1.utility/entry.mjs",
            "foreign://parser_1.utility/a.mjs",
            "myapp://parser_1.utility/a/../entry.mjs",
            "javascript:alert(1)",
        ] {
            assert!(
                validate_url("Parser_1", invalid, Some("myapp")).is_err(),
                "{invalid}"
            );
        }
        assert_eq!(
            entry_url("myapp", "parser", "https://example.com/module.mjs").unwrap(),
            "https://example.com/module.mjs"
        );
    }

    #[test]
    fn sources_support_borrowed_and_dynamic_owned_strings() {
        let static_source = ModuleSource::new("entry.mjs", "export default {};");
        assert!(matches!(static_source.source, Cow::Borrowed(_)));
        let dynamic_source = ModuleSource::new(
            format!("{}.mjs", "generated"),
            format!("export const value = {};", 42),
        );
        assert!(matches!(dynamic_source.name, Cow::Owned(_)));
        assert!(matches!(dynamic_source.source, Cow::Owned(_)));
    }

    #[test]
    fn io_errors_preserve_their_resource_category() {
        let error: Error = std::io::Error::from(std::io::ErrorKind::NotFound).into();
        assert!(matches!(error, Error::Resource(_)));
    }
}
