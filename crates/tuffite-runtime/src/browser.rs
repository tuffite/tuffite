//! Browser-process state and trusted native capabilities.

use crate::abi::NativeHost;
use crate::executor::ChromiumExecutor;
use crate::javascript_extension::JavaScriptExtensionHost;
use crate::shell_api::ShellApiHost;
use crate::utility_host::UtilitySessionHost;
use crate::{ApplicationError, ApplicationResult, CommandResult};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tuffite_abi::{RuntimeEventKind, Status};

/// Typed application state shared only within one Browser runtime.
#[derive(Clone, Default)]
pub(crate) struct SharedState(Arc<RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>);

impl SharedState {
    pub(crate) fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.0
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&TypeId::of::<T>())
            .cloned()?
            .downcast()
            .ok()
    }
    pub(crate) fn set<T: Send + Sync + 'static>(&self, value: Arc<T>) {
        self.0
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(TypeId::of::<T>(), value);
    }
}

impl std::fmt::Debug for SharedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SharedState")
    }
}

/// Immutable startup data supplied when one browser instance is created.
#[derive(Clone, Debug)]
pub struct BrowserContext {
    configuration: Arc<[u8]>,
    events: EventEmitter,
    executor: ChromiumExecutor,
    utility_sessions: UtilitySessionHost,
    javascript_extensions: JavaScriptExtensionHost,
    shell_api: ShellApiHost,
    state: SharedState,
    providers: Arc<crate::utility::V8Providers>,
}

impl BrowserContext {
    pub(crate) fn new(
        configuration: Vec<u8>,
        runtime: Arc<AtomicU64>,
        callbacks: NativeHost,
        session: tuffite_abi::UtilitySessionClient,
        shell_api: Option<tuffite_abi::ShellApi>,
    ) -> Self {
        let executor = ChromiumExecutor::new(callbacks);
        let utility_sessions = UtilitySessionHost::new(session);
        let javascript_extensions = JavaScriptExtensionHost::new(utility_sessions.clone());
        let shell_api = ShellApiHost::new(shell_api);
        let configuration: Arc<[u8]> = configuration.into();
        let state = SharedState::default();
        let providers = Arc::new(crate::utility::V8Providers::new(
            configuration.clone(),
            executor.clone(),
            state.clone(),
        ));
        Self {
            configuration,
            state,
            providers,
            events: EventEmitter { runtime, callbacks },
            executor,
            utility_sessions,
            javascript_extensions,
            shell_api,
        }
    }

    /// Installs typed state shared with V8 providers in this Browser runtime.
    /// Existing holders retain their Arc if this type is replaced. No state crosses IPC.
    pub fn set_state<T: Send + Sync + 'static>(&self, value: Arc<T>) {
        self.state.set(value);
    }

    /// Obtains typed Browser-process state.
    #[must_use]
    pub fn state<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.state.get()
    }

    pub(crate) fn providers(&self) -> Arc<crate::utility::V8Providers> {
        self.providers.clone()
    }

    /// Returns the exact validated application-manifest bytes supplied at
    /// startup. Applications may deserialize these into their own settings.
    #[must_use]
    pub fn configuration(&self) -> &[u8] {
        &self.configuration
    }

    /// Returns a cloneable emitter for browser-to-renderer application events.
    #[must_use]
    pub fn events(&self) -> EventEmitter {
        self.events.clone()
    }

    /// Returns a cloneable executor backed by Chromium's process-wide thread
    /// pool. Use it for finite work that benefits from Chromium scheduling.
    #[must_use]
    pub fn executor(&self) -> ChromiumExecutor {
        self.executor.clone()
    }

    /// Returns the client for explicitly registered Rust and V8 utilities.
    #[must_use]
    pub fn utilities(&self) -> crate::utility::Host {
        crate::utility::Host::new(
            self.utility_sessions.clone(),
            crate::utility::v8::app_scheme(&self.configuration),
            self.providers.clone(),
        )
    }

    /// Returns the trusted Host API for long-lived Utility sessions.
    #[must_use]
    pub fn utility_sessions(&self) -> UtilitySessionHost {
        self.utility_sessions.clone()
    }

    /// Returns the trusted Host API for restricted JavaScript extension realms.
    #[must_use]
    #[deprecated(note = "declare a named V8 utility and use utilities().create_v8(name, ...)")]
    pub fn javascript_extensions(&self) -> JavaScriptExtensionHost {
        self.javascript_extensions.clone()
    }

    /// Returns the trusted Host API backed by Chromium Remote Desktop.
    #[must_use]
    pub fn shell_api(&self) -> ShellApiHost {
        self.shell_api.clone()
    }
}

/// Cloneable, thread-safe handle for broadcasting named binary events.
#[derive(Clone, Debug)]
pub struct EventEmitter {
    runtime: Arc<AtomicU64>,
    callbacks: NativeHost,
}

impl EventEmitter {
    const MAX_NAME_BYTES: usize = 256;
    const MAX_PAYLOAD_BYTES: usize = 1024 * 1024;

    /// Broadcasts one event to all currently authorized renderer documents.
    pub fn emit(&self, name: &str, payload: &[u8]) -> ApplicationResult<()> {
        if name.is_empty() || name.len() > Self::MAX_NAME_BYTES || name.as_bytes().contains(&0) {
            return Err(ApplicationError::new("event name is invalid"));
        }
        if payload.len() > Self::MAX_PAYLOAD_BYTES {
            return Err(ApplicationError::new("event payload is too large"));
        }
        let name_length = u16::try_from(name.len())
            .map_err(|_| ApplicationError::new("event name is invalid"))?;
        let mut envelope = Vec::with_capacity(2 + name.len() + payload.len());
        envelope.extend_from_slice(&name_length.to_le_bytes());
        envelope.extend_from_slice(name.as_bytes());
        envelope.extend_from_slice(payload);
        let runtime = self.runtime.load(Ordering::Acquire);
        if runtime == 0 {
            return Err(ApplicationError::new("runtime is not ready"));
        }
        let status =
            self.callbacks
                .post_event(runtime, RuntimeEventKind::ApplicationEvent, &envelope);
        match status {
            Status::Ok => Ok(()),
            _ => Err(ApplicationError::new(format!(
                "unable to emit renderer event: {status:?}"
            ))),
        }
    }
}

/// One decoded binary `ShellAPI` command borrowed for the duration of dispatch.
#[derive(Clone, Copy, Debug)]
pub struct Command<'a> {
    id: u64,
    schema_fingerprint: u64,
    payload: &'a [u8],
}

impl<'a> Command<'a> {
    pub(crate) const fn new(id: u64, schema_fingerprint: u64, payload: &'a [u8]) -> Self {
        Self {
            id,
            schema_fingerprint,
            payload,
        }
    }

    /// Stable ID derived from the canonical command name.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Normalized request/result schema fingerprint sent by the caller.
    #[must_use]
    pub const fn schema_fingerprint(&self) -> u64 {
        self.schema_fingerprint
    }

    /// Command payload without JSON serialization.
    #[must_use]
    pub const fn payload(&self) -> &'a [u8] {
        self.payload
    }
}

/// Stateful backend created only in the browser process.
///
/// Command dispatch is serialized on the Rust dispatch worker. Shutdown runs
/// after the backend rejects new commands. Utility children never construct
/// this state or receive its trusted native capabilities.
pub trait Browser: Send + 'static {
    /// Creates browser state from validated startup configuration.
    fn new(context: BrowserContext) -> ApplicationResult<Self>
    where
        Self: Sized;

    /// Handles one binary `ShellAPI` command.
    fn dispatch(&mut self, _command: Command<'_>) -> CommandResult {
        Err(crate::CommandError::UnknownCommand)
    }

    /// Cleans up browser state after new dispatches are denied.
    fn shutdown(&mut self) -> ApplicationResult<()> {
        Ok(())
    }
}

/// Type metadata emitted by the browser attribute and checked by main.
#[doc(hidden)]
pub trait BrowserDefinition: Browser {
    const TYPE_NAME: &'static str;
}
