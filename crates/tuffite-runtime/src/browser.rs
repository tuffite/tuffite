//! Browser-process state and trusted native capabilities.

use crate::abi::NativeHost;
use crate::executor::ChromiumExecutor;
use crate::javascript_extension::JavaScriptExtensionHost;
use crate::lifecycle::BoxFuture;
use crate::utility_host::UtilitySessionHost;
use crate::{CommandResult, Error, Result, ShutdownContext, StopSignal};
use std::any::Any;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tuffite_abi::{RuntimeEventKind, Status};

/// Immutable startup data supplied when one browser instance is created.
#[derive(Clone, Debug)]
pub struct BrowserContext {
    pub(crate) application: Option<crate::ApplicationContext>,
    pub(crate) profile_id: Option<crate::ProfileId>,
    pub(crate) context_identity: Arc<AtomicU64>,
    pub(crate) native_calls: Arc<RwLock<bool>>,
    configuration: Arc<[u8]>,
    events: EventEmitter,
    executor: ChromiumExecutor,
    utility_sessions: UtilitySessionHost,
    javascript_extensions: JavaScriptExtensionHost,
    isolated: crate::isolated::Host,
    pub(crate) shortcut_callbacks: crate::shortcuts::Callbacks,
    pub(crate) context_slot: crate::context_data::Slot,
    providers: Arc<crate::utility::V8Providers>,
    stop: StopSignal,
}

impl AsRef<Self> for BrowserContext {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl BrowserContext {
    pub(crate) fn new(
        configuration: Vec<u8>,
        runtime: Arc<AtomicU64>,
        callbacks: NativeHost,
        session: tuffite_abi::UtilitySessionClient,
        isolated: Option<tuffite_abi::Isolated>,
    ) -> Self {
        let executor = ChromiumExecutor::new(callbacks);
        let utility_sessions = UtilitySessionHost::new(session);
        let javascript_extensions = JavaScriptExtensionHost::new(utility_sessions.clone());
        let configuration: Arc<[u8]> = configuration.into();
        let context_slot = crate::context_data::Slot::default();
        let providers = Arc::new(crate::utility::V8Providers::new(
            configuration.clone(),
            executor.clone(),
            context_slot.clone(),
        ));
        Self {
            application: None,
            profile_id: None,
            context_identity: Arc::new(AtomicU64::new(0)),
            native_calls: Arc::new(RwLock::new(true)),
            configuration,
            stop: StopSignal(crate::supervisor::CancellationToken::new()),
            context_slot,
            shortcut_callbacks: crate::shortcuts::Callbacks::default(),
            providers,
            events: EventEmitter { runtime, callbacks },
            executor,
            utility_sessions,
            javascript_extensions,
            isolated: crate::isolated::Host::new(isolated),
        }
    }

    /// Owning application. Available for contexts created by the profile manager.
    #[must_use]
    pub fn application(&self) -> Option<crate::ApplicationContext> {
        self.application.clone()
    }
    #[must_use]
    pub fn profile_id(&self) -> Option<&crate::ProfileId> {
        self.profile_id.as_ref()
    }
    /// The generation is published before `initialize`. Constructors can retain
    /// the context and its clients, then obtain this identity during initialization.
    pub fn handle(&self) -> Result<crate::BrowserContextHandle> {
        crate::BrowserContextHandle::from_u64(self.context_identity.load(Ordering::Acquire))
    }
    pub(crate) fn remoting_host(&self) -> crate::native_remoting::Host {
        crate::native_remoting::Host::discover(
            self.events.callbacks,
            self.stop_signal(),
            self.native_calls.clone(),
        )
    }
    pub(crate) fn context_client(&self) -> crate::contexts::Client {
        crate::contexts::Client::discover(
            self.events.callbacks,
            self.stop_signal(),
            self.native_calls.clone(),
        )
    }
    #[must_use]
    pub fn views(&self) -> crate::Views {
        crate::Views::new(self.context_client(), self.context_identity.clone())
    }

    /// Read-only signal fired before draining ordinary Browser work.
    #[must_use]
    pub fn stop_signal(&self) -> StopSignal {
        self.stop.clone()
    }

    /// Restores the business context installed by the setup Builder's factory.
    pub fn downcast<T: Any + Send + Sync>(&self) -> Result<Arc<T>> {
        self.context_slot.require()
    }
    #[must_use]
    pub fn try_downcast<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.context_slot.get()
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
        crate::utility::Host::new(self.utility_sessions.clone(), self.providers.clone())
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

    /// Returns the native registry for dynamically installed DOM UI scripts.
    #[must_use]
    pub fn isolated(&self) -> crate::isolated::Host {
        self.isolated.clone()
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
    pub fn emit(&self, name: &str, payload: &[u8]) -> Result<()> {
        if name.is_empty() || name.len() > Self::MAX_NAME_BYTES || name.as_bytes().contains(&0) {
            return Err(Error::new("event name is invalid"));
        }
        if payload.len() > Self::MAX_PAYLOAD_BYTES {
            return Err(Error::new("event payload is too large"));
        }
        let name_length =
            u16::try_from(name.len()).map_err(|_| Error::new("event name is invalid"))?;
        let mut envelope = Vec::with_capacity(2 + name.len() + payload.len());
        envelope.extend_from_slice(&name_length.to_le_bytes());
        envelope.extend_from_slice(name.as_bytes());
        envelope.extend_from_slice(payload);
        let runtime = self.runtime.load(Ordering::Acquire);
        if runtime == 0 {
            return Err(Error::new("runtime is not ready"));
        }
        let status =
            self.callbacks
                .post_event(runtime, RuntimeEventKind::ApplicationEvent, &envelope);
        match status {
            Status::Ok => Ok(()),
            _ => Err(Error::new(format!(
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

/// Trusted browser configuration collected before native services and windows.
/// These denials supplement the manifest; neither source can enable an origin
/// denied by the other. This configuration is never exposed to renderer scripts.
pub struct Builder<B = ()> {
    pub(crate) devtools_denied_origins: Vec<String>,
    pub(crate) interfaces: crate::app::InterfaceRegistry<B>,
    pub(crate) profile_id: Option<crate::ProfileId>,
    pub(crate) context_factory: crate::context_data::Factory<BrowserContext>,
}
impl<B> Default for Builder<B> {
    fn default() -> Self {
        Self {
            devtools_denied_origins: Vec::new(),
            interfaces: crate::app::InterfaceRegistry::default(),
            profile_id: None,
            context_factory: crate::context_data::Factory::default(),
        }
    }
}
impl<B> Builder<B> {
    /// Registers construction; the closure is evaluated only for a live runtime.
    #[must_use]
    pub fn context<C: Any + Send + Sync>(
        mut self,
        factory: impl Fn(BrowserContext) -> Result<C> + Send + Sync + 'static,
    ) -> Self {
        self.context_factory = crate::context_data::Factory::new(factory);
        self
    }

    #[must_use]
    pub fn disable_devtools_for_origin(mut self, origin: impl Into<String>) -> Self {
        self.devtools_denied_origins.push(origin.into());
        self
    }
    #[must_use]
    pub fn interfaces(
        mut self,
        register: impl FnOnce(crate::app::InterfaceRegistry<B>) -> crate::app::InterfaceRegistry<B>,
    ) -> Self {
        self.interfaces
            .commands
            .extend(register(crate::app::InterfaceRegistry::default()).commands);
        self
    }
    /// None while static interface metadata is collected, Some during profile construction.
    #[must_use]
    pub fn profile_id(&self) -> Option<&crate::ProfileId> {
        self.profile_id.as_ref()
    }
    #[cfg(test)]
    pub(crate) fn retype<T>(self) -> Builder<T> {
        Builder {
            devtools_denied_origins: self.devtools_denied_origins,
            context_factory: self.context_factory,
            ..Builder::default()
        }
    }
}

/// Stateful backend created only in the browser process.
///
/// Command dispatch is serialized on the Rust dispatch worker. Shutdown runs
/// after the backend rejects new commands. Utility children never construct
/// this state or receive its trusted native capabilities.
pub trait Browser: Send + 'static {
    /// Collects static interfaces and native policy without a profile, then runs
    /// again before each profile backend is constructed. `profile_id()` identifies
    /// the latter pass. Interfaces and native denials must match the frozen pass;
    /// setup has no runtime capabilities. Errors reject startup or the profile.
    fn setup(builder: Builder<Self>) -> Result<Builder<Self>>
    where
        Self: Sized,
    {
        Ok(builder)
    }

    /// Creates browser state from validated startup configuration.
    fn new(context: BrowserContext) -> Result<Self>
    where
        Self: Sized;

    /// Runs once after `BrowserContext` services and their task runner are bound,
    /// before startup windows and business dispatch. Runs on the Rust worker.
    fn initialize(&mut self) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }

    /// Resolves resource/action URLs within this profile.
    fn resolve(
        &mut self,
        _url: &crate::Url,
        _context: crate::ResolveContext,
    ) -> impl Future<Output = Result<crate::Resolution>> + Send {
        async { Ok(crate::Resolution::Pass) }
    }

    /// Low-level dispatch hook used by standalone runtime adapters. Embedded
    /// applications dispatch through their explicitly registered interfaces.
    fn dispatch(&mut self, _command: Command<'_>) -> CommandResult {
        Err(crate::CommandError::UnknownCommand)
    }

    /// Cleans up after dispatch, Utility sessions and module workers stop.
    /// The Chromium executor remains available; UI-dependent host calls reject.
    /// Pending futures have the supplied cleanup budget; blocking polls cannot be interrupted.
    fn shutdown(&mut self, _context: ShutdownContext) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

pub(crate) trait ErasedBrowser: Send {
    fn initialize(&mut self) -> BoxFuture<'_, Result<()>>;
    fn dispatch(&mut self, command: Command<'_>) -> CommandResult;
    fn resolve<'a>(
        &'a mut self,
        url: &'a crate::Url,
        context: crate::ResolveContext,
    ) -> BoxFuture<'a, Result<crate::Resolution>>;
    fn shutdown(&mut self, context: ShutdownContext) -> BoxFuture<'_, Result<()>>;
}
impl<B: Browser> ErasedBrowser for B {
    fn resolve<'a>(
        &'a mut self,
        url: &'a crate::Url,
        context: crate::ResolveContext,
    ) -> BoxFuture<'a, Result<crate::Resolution>> {
        Box::pin(Browser::resolve(self, url, context))
    }
    fn initialize(&mut self) -> BoxFuture<'_, Result<()>> {
        Box::pin(Browser::initialize(self))
    }
    fn dispatch(&mut self, command: Command<'_>) -> CommandResult {
        Browser::dispatch(self, command)
    }
    fn shutdown(&mut self, context: ShutdownContext) -> BoxFuture<'_, Result<()>> {
        Box::pin(Browser::shutdown(self, context))
    }
}
