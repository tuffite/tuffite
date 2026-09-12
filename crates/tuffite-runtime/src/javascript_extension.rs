//! Restricted JavaScript extension sessions backed by Chromium's Utility process.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::utility_host::{
    UtilitySessionClosed, UtilitySessionHandle, UtilitySessionHost, UtilitySessionHostError,
    UtilitySessionOptions,
};

const SERVICE_NAME: &str = "tuffite_js_extension";
const MAX_MODULES: usize = 64;
const MAX_MODULE_NAME_BYTES: usize = 512;
const MAX_MODULE_SOURCE_BYTES: usize = 512 * 1024;
const MAX_MODULE_GRAPH_BYTES: usize = 1024 * 1024;
const MAX_METHOD_BYTES: usize = 128;
const MAX_ARGUMENTS: usize = 16;

/// Complete immutable ECMAScript module graph supplied by trusted Rust code.
///
/// Module names are logical slash-separated paths. Static imports may only be
/// relative and must resolve to another module in this graph. The Utility
/// process never reads extension files itself.
#[derive(Clone, Debug)]
pub struct JavaScriptExtensionPackage {
    entry: String,
    modules: BTreeMap<String, String>,
}

impl JavaScriptExtensionPackage {
    /// Creates an empty graph whose eventual entry module has `entry` as its
    /// logical name.
    pub fn new(entry: impl Into<String>) -> Result<Self, JavaScriptExtensionError> {
        let entry = entry.into();
        validate_module_name(&entry)?;
        Ok(Self {
            entry,
            modules: BTreeMap::new(),
        })
    }

    /// Adds one source-text module. Names must be unique, relative logical
    /// paths ending in `.mjs`; sources and the whole graph are bounded.
    pub fn add_module(
        mut self,
        name: impl Into<String>,
        source: impl Into<String>,
    ) -> Result<Self, JavaScriptExtensionError> {
        let name = name.into();
        let source = source.into();
        validate_module_name(&name)?;
        if source.is_empty() || source.len() > MAX_MODULE_SOURCE_BYTES {
            return Err(JavaScriptExtensionError::InvalidModule);
        }
        if self.modules.len() >= MAX_MODULES || self.modules.contains_key(&name) {
            return Err(JavaScriptExtensionError::InvalidModule);
        }
        let total = self
            .modules
            .values()
            .try_fold(source.len(), |total, value| total.checked_add(value.len()))
            .ok_or(JavaScriptExtensionError::PackageTooLarge)?;
        if total > MAX_MODULE_GRAPH_BYTES {
            return Err(JavaScriptExtensionError::PackageTooLarge);
        }
        self.modules.insert(name, source);
        Ok(self)
    }

    pub(crate) fn encode(&self) -> Result<Vec<u8>, JavaScriptExtensionError> {
        if !self.modules.contains_key(&self.entry) {
            return Err(JavaScriptExtensionError::MissingEntry);
        }
        let modules = self
            .modules
            .iter()
            .map(|(name, source)| {
                serde_json::json!({
                    "name": name,
                    "source": source,
                })
            })
            .collect::<Vec<_>>();
        serde_json::to_vec(&serde_json::json!({
            "schemaVersion": 1,
            "entry": self.entry,
            "modules": modules,
        }))
        .map_err(|_| JavaScriptExtensionError::InvalidModule)
    }

    /// Returns the number of immutable modules in the graph.
    #[must_use]
    pub fn module_count(&self) -> usize {
        self.modules.len()
    }
}

/// Error detected while constructing or invoking a restricted extension realm.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum JavaScriptExtensionError {
    #[error("JavaScript extension module is invalid")]
    InvalidModule,
    #[error("JavaScript extension entry module is missing")]
    MissingEntry,
    #[error("JavaScript extension package exceeds its size limit")]
    PackageTooLarge,
    #[error("JavaScript extension method is invalid")]
    InvalidMethod,
    #[error("JavaScript extension command could not be encoded")]
    InvalidCommand,
    #[error("JavaScript extension returned invalid JSON")]
    InvalidResponse,
    #[error(transparent)]
    Session(#[from] UtilitySessionHostError),
}

/// Cloneable trusted entrypoint for restricted JavaScript extension sessions.
#[derive(Clone, Debug)]
pub struct JavaScriptExtensionHost {
    sessions: UtilitySessionHost,
}

impl JavaScriptExtensionHost {
    pub(crate) const fn new(sessions: UtilitySessionHost) -> Self {
        Self { sessions }
    }

    /// Creates a capability-free V8 realm in a sandboxed Utility process.
    pub fn create<C, O>(
        &self,
        package: &JavaScriptExtensionPackage,
        on_closed: C,
        on_created: O,
    ) -> Result<JavaScriptExtensionSessionHandle, JavaScriptExtensionError>
    where
        C: FnOnce(UtilitySessionClosed) + Send + 'static,
        O: FnOnce(Result<(), UtilitySessionHostError>) + Send + 'static,
    {
        self.create_named(SERVICE_NAME, package, on_closed, on_created)
    }

    pub(crate) fn create_resolved<C, O>(
        &self,
        name: &str,
        entry: &str,
        on_closed: C,
        on_created: O,
    ) -> Result<JavaScriptExtensionSessionHandle, JavaScriptExtensionError>
    where
        C: FnOnce(UtilitySessionClosed) + Send + 'static,
        O: FnOnce(Result<(), UtilitySessionHostError>) + Send + 'static,
    {
        crate::utility::v8::validate_url(name, entry, None)
            .map_err(|_| JavaScriptExtensionError::InvalidModule)?;
        let init = serde_json::to_vec(&serde_json::json!({"schemaVersion": 2, "entry": entry}))
            .map_err(|_| JavaScriptExtensionError::InvalidModule)?;
        let handle = self.sessions.create(
            name,
            &UtilitySessionOptions {
                init,
                initial_event_credits: 1,
                max_event_bytes: 1024,
            },
            |_| false,
            on_closed,
            on_created,
        )?;
        Ok(JavaScriptExtensionSessionHandle { inner: handle })
    }

    pub(crate) fn create_named<C, O>(
        &self,
        name: &str,
        package: &JavaScriptExtensionPackage,
        on_closed: C,
        on_created: O,
    ) -> Result<JavaScriptExtensionSessionHandle, JavaScriptExtensionError>
    where
        C: FnOnce(UtilitySessionClosed) + Send + 'static,
        O: FnOnce(Result<(), UtilitySessionHostError>) + Send + 'static,
    {
        let handle = self.sessions.create(
            name,
            &UtilitySessionOptions {
                init: package.encode()?,
                initial_event_credits: 1,
                max_event_bytes: 1024,
            },
            |_| false,
            on_closed,
            on_created,
        )?;
        Ok(JavaScriptExtensionSessionHandle { inner: handle })
    }
}

/// Opaque handle for one restricted JavaScript extension realm.
#[derive(Clone, Debug)]
pub struct JavaScriptExtensionSessionHandle {
    inner: UtilitySessionHandle,
}

impl JavaScriptExtensionSessionHandle {
    /// Calls one exported extension method with JSON arguments. Calls are ordered
    /// within a session; sibling sessions can run concurrently.
    pub fn call<F>(
        &self,
        method: &str,
        arguments: &[Value],
        reply: F,
    ) -> Result<u64, JavaScriptExtensionError>
    where
        F: FnOnce(Result<Value, JavaScriptExtensionError>) + Send + 'static,
    {
        if !valid_method(method) || arguments.len() > MAX_ARGUMENTS {
            return Err(JavaScriptExtensionError::InvalidMethod);
        }
        let command = serde_json::to_vec(&serde_json::json!({
            "method": method,
            "args": arguments,
        }))
        .map_err(|_| JavaScriptExtensionError::InvalidCommand)?;
        let command_id = self
            .inner
            .send(&command, move |result| {
                let result = result
                    .map_err(JavaScriptExtensionError::Session)
                    .and_then(|bytes| {
                        serde_json::from_slice(&bytes)
                            .map_err(|_| JavaScriptExtensionError::InvalidResponse)
                    });
                reply(result);
            })
            .map_err(JavaScriptExtensionError::Session)?;
        Ok(command_id)
    }

    /// Cancels one queued or running extension call. Running V8 execution is
    /// terminated from the Utility control sequence.
    pub fn cancel(&self, command_id: u64) -> Result<(), JavaScriptExtensionError> {
        self.inner
            .cancel(command_id)
            .map_err(JavaScriptExtensionError::Session)
    }

    /// Closes the realm and releases its bounded V8 heap.
    pub fn close<F>(&self, reason: u32, reply: F) -> Result<(), JavaScriptExtensionError>
    where
        F: FnOnce(Result<Vec<u8>, UtilitySessionHostError>) + Send + 'static,
    {
        self.inner
            .close(reason, reply)
            .map_err(JavaScriptExtensionError::Session)
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.inner.is_active()
    }
}

pub(crate) fn validate_module_name(name: &str) -> Result<(), JavaScriptExtensionError> {
    let valid = !name.is_empty()
        && name.len() <= MAX_MODULE_NAME_BYTES
        && std::path::Path::new(name)
            .extension()
            .is_some_and(|extension| extension == "mjs")
        && !name.starts_with('/')
        && !name.ends_with('/')
        && !name.contains(['\\', '\0'])
        && name
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..");
    valid
        .then_some(())
        .ok_or(JavaScriptExtensionError::InvalidModule)
}

fn valid_method(method: &str) -> bool {
    !method.is_empty()
        && method.len() <= MAX_METHOD_BYTES
        && method
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_enforces_entry_and_graph_limits() {
        let missing = JavaScriptExtensionPackage::new("entry.mjs").unwrap();
        assert_eq!(
            missing.encode(),
            Err(JavaScriptExtensionError::MissingEntry)
        );
        let package = missing
            .add_module("entry.mjs", "export default {};")
            .unwrap()
            .add_module("lib/helper.mjs", "export const value = 1;")
            .unwrap();
        let encoded = package.encode().unwrap();
        assert!(encoded.starts_with(b"{"));
        assert!(JavaScriptExtensionPackage::new("../escape.mjs").is_err());
    }

    #[test]
    fn method_names_are_narrow_identifiers() {
        assert!(valid_method("onStdout"));
        assert!(!valid_method("__proto__.pollute"));
        assert!(!valid_method(""));
    }
}
