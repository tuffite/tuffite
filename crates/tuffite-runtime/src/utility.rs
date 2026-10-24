//! Sandboxed utility implementations and browser-side clients.
//!
//! Rust and V8 definitions share the utility attribute. Rust implementations
//! use [`Utility`]; V8 definitions use [`V8`] and [`v8::ModuleSource`].
//! Both use supervised Chromium utility processes and session transport.

use crate::lifecycle::BoxFuture;
use crate::supervisor::CancellationToken;
use crate::utility_session::{UtilitySession, UtilitySessionRequest};
use crate::{ApplicationError, ApplicationResult, CommandResult};
use crate::{ShutdownContext, ShutdownReason, StopSignal};
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Instant;

mod builder;
pub use builder::Builder;

/// Dispatches one typed Mojom command inside a Utility, preserving its envelope.
/// Native routing and origin/service/endpoint authorization happen before delivery.
pub fn dispatch_typed(
    request: &UtilityRequest,
    handler: impl FnOnce(crate::Command<'_>) -> CommandResult,
) -> Vec<u8> {
    crate::protocol::dispatch_with(request.payload(), |id, fingerprint, payload| {
        handler(crate::Command::new(id, fingerprint, payload))
    })
}

/// Immutable service startup data owned by the Utility child process.
#[derive(Clone)]
pub struct UtilityContext {
    name: Arc<str>,
    configuration: Arc<[u8]>,
    cancellation: CancellationToken,
}

impl UtilityContext {
    pub(crate) fn new(name: &str, configuration: Vec<u8>, cancellation: CancellationToken) -> Self {
        Self {
            name: name.into(),
            configuration: configuration.into(),
            cancellation,
        }
    }

    /// Builds standalone startup data for application unit tests, without a native service.
    #[doc(hidden)]
    #[must_use]
    pub fn new_for_test(name: &str, configuration: Vec<u8>) -> Self {
        Self::new(name, configuration, CancellationToken::new())
    }

    /// Returns the registered service name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns this service's frozen startup data after `setup`, not the app manifest.
    /// Framework seeds setup from the service's manifest configuration; missing
    /// configuration is an empty byte slice. Session init is independent.
    #[must_use]
    pub fn configuration(&self) -> &[u8] {
        &self.configuration
    }

    /// Reports service shutdown; retained context clones observe the same signal.
    #[must_use]
    pub fn stop_signal(&self) -> StopSignal {
        StopSignal(self.cancellation.clone())
    }

    /// Returns whether the ordinary-work stop signal has fired.
    #[must_use]
    pub fn is_stopping(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}

/// Browser-process environment for one named V8 resource provider.
#[derive(Clone)]
pub struct V8Context {
    name: Arc<str>,
    configuration: Arc<[u8]>,
    application_configuration: Arc<[u8]>,
    executor: crate::ChromiumExecutor,
    state: crate::browser::SharedState,
    cancellation: CancellationToken,
}

impl V8Context {
    /// Returns the registered service name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns this service's frozen startup data after `setup`.
    /// Framework seeds setup from the service's manifest configuration.
    #[must_use]
    pub fn configuration(&self) -> &[u8] {
        &self.configuration
    }

    /// Returns the Browser's immutable validated application manifest.
    #[must_use]
    pub fn application_configuration(&self) -> &[u8] {
        &self.application_configuration
    }

    /// Returns the Browser process executor. No capability crosses into the JS isolate.
    #[must_use]
    pub fn executor(&self) -> crate::ChromiumExecutor {
        self.executor.clone()
    }

    /// Obtains typed state installed by `BrowserContext::set_state` in this Browser runtime.
    #[must_use]
    pub fn state<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.state.get()
    }

    /// Reports Browser shutdown.
    #[must_use]
    pub fn stop_signal(&self) -> StopSignal {
        StopSignal(self.cancellation.clone())
    }

    /// Returns whether the ordinary-work stop signal has fired.
    #[must_use]
    pub fn is_stopping(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}

type V8Factory = fn(V8Context) -> ApplicationResult<Arc<dyn ErasedV8>>;
type ProviderCell = OnceLock<ApplicationResult<Arc<dyn ErasedV8>>>;

pub(crate) struct V8Providers {
    configuration: Arc<[u8]>,
    executor: crate::ChromiumExecutor,
    state: crate::browser::SharedState,
    cancellation: CancellationToken,
    providers: Mutex<HashMap<String, Arc<ProviderCell>>>,
    sessions: Mutex<HashMap<(String, u64), CancellationToken>>,
    construction: RwLock<()>,
}

impl V8Providers {
    pub(crate) fn new(
        configuration: Arc<[u8]>,
        executor: crate::ChromiumExecutor,
        state: crate::browser::SharedState,
    ) -> Self {
        Self {
            configuration,
            executor,
            state,
            cancellation: CancellationToken::new(),
            providers: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            construction: RwLock::new(()),
        }
    }

    pub(crate) fn get(&self, name: &str) -> ApplicationResult<Arc<dyn ErasedV8>> {
        // Fence constructors started through Host clones as well as resolver workers.
        let _construction = self
            .construction
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.cancellation.is_cancelled() {
            return Err(ApplicationError::new("Browser is stopping"));
        }
        let registration = crate::builder::utility_definition(name)
            .ok_or_else(|| ApplicationError::new("unknown utility service"))?;
        let factory = registration
            .v8
            .ok_or_else(|| ApplicationError::new("utility is not a V8 service"))?;
        let cell = self
            .providers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(name.to_owned())
            .or_default()
            .clone();
        let result = cell.get_or_init(|| {
            let context = V8Context {
                name: name.into(),
                configuration: service_configuration(&self.configuration, name).into(),
                application_configuration: self.configuration.clone(),
                executor: self.executor.clone(),
                state: self.state.clone(),
                cancellation: self.cancellation.clone(),
            };
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| factory(context)))
                .unwrap_or_else(|_| Err(ApplicationError::new("V8 provider constructor panicked")))
        });
        result
            .as_ref()
            .map(Arc::clone)
            .map_err(|error| ApplicationError::new(error.to_string()))
    }

    pub(crate) fn session_signal(&self, name: &str, session: u64) -> StopSignal {
        StopSignal(
            self.sessions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .entry((name.to_owned(), session))
                .or_default()
                .clone(),
        )
    }
    pub(crate) fn cancel_session(&self, name: &str, session: u64) {
        let token = self
            .sessions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&(name.to_owned(), session));
        if let Some(token) = token {
            token.cancel();
        }
    }
    pub(crate) fn stop(&self) {
        self.cancellation.cancel();
        let sessions = std::mem::take(
            &mut *self
                .sessions
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        for (_, token) in sessions {
            token.cancel();
        }
    }

    pub(crate) fn release(&self) -> ApplicationResult<()> {
        let construction = self
            .construction
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let providers = std::mem::take(
            &mut *self
                .providers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        drop(construction);
        // No registry lock or resolver worker is held while user cleanup runs.
        let context = ShutdownContext::new(
            ShutdownReason::Requested,
            Instant::now() + crate::lifecycle::LIFECYCLE_TIMEOUT,
        );
        let mut failed = false;
        for cell in providers.values() {
            if let Some(Ok(provider)) = cell.get() {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    context.run(provider.shutdown(context.clone()))
                }));
                if !matches!(result, Ok(Ok(()))) {
                    failed = true;
                    tracing::error!("V8 provider cleanup failed");
                }
            }
        }
        drop(providers);
        if failed {
            Err(ApplicationError::new("V8 provider cleanup failed"))
        } else {
            Ok(())
        }
    }
}

pub(crate) fn service_configuration(manifest: &[u8], name: &str) -> Vec<u8> {
    serde_json::from_slice::<serde_json::Value>(manifest)
        .ok()
        .and_then(|value| {
            value
                .get("app")?
                .get("utilities")?
                .get(name)?
                .get("configuration")
                .cloned()
        })
        .and_then(|value| serde_json::to_vec(&value).ok())
        .unwrap_or_default()
}

pub(crate) async fn resolve_source(
    provider: &dyn ErasedV8,
    name: &str,
    request: v8::ModuleRequest,
) -> ApplicationResult<Option<Vec<u8>>> {
    let url = request.url().to_owned();
    v8::validate_url(name, &url, None).map_err(|error| ApplicationError::new(error.to_string()))?;
    let Some(module) = provider
        .resolve(request)
        .await
        .map_err(|error| ApplicationError::new(error.to_string()))?
    else {
        return Ok(None);
    };
    if module.name != url || module.source.is_empty() || module.source.len() > 512 * 1024 {
        return Err(ApplicationError::new(
            "resolver returned an invalid module source",
        ));
    }
    Ok(Some(module.source.into_owned().into_bytes()))
}

/// Owned input and cooperative lifecycle controls for one utility invocation.
pub struct UtilityRequest {
    payload: Vec<u8>,
    cancellation: CancellationToken,
    deadline: Instant,
}

impl UtilityRequest {
    pub(crate) const fn new(
        payload: Vec<u8>,
        cancellation: CancellationToken,
        deadline: Instant,
    ) -> Self {
        Self {
            payload,
            cancellation,
            deadline,
        }
    }

    /// Returns the complete binary request body.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Reports whether the browser cancelled this invocation.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }

    /// Returns the absolute host-enforced deadline.
    #[must_use]
    pub const fn deadline(&self) -> Instant {
        self.deadline
    }

    #[doc(hidden)]
    #[must_use]
    pub fn new_for_test(payload: Vec<u8>) -> Self {
        Self::new(
            payload,
            CancellationToken::new(),
            Instant::now() + std::time::Duration::from_secs(30),
        )
    }
}

/// Stateful Rust implementation hosted in a sandboxed Chromium process.
pub trait Utility: Send + Sync + 'static {
    /// Configures this child before state construction. Framework supplies manifest defaults.
    fn setup(builder: Builder) -> ApplicationResult<Builder>
    where
        Self: Sized,
    {
        Ok(builder)
    }

    /// Asynchronously creates state inside the launched sandboxed child.
    /// Ready is reported only after this future succeeds; failure owns its partial rollback.
    fn new(context: UtilityContext) -> impl Future<Output = ApplicationResult<Self>> + Send
    where
        Self: Sized;

    /// Handles one binary request without blocking the Chromium utility sequence.
    /// Implementations should observe `request.is_cancelled()` during long work.
    fn invoke(&self, request: UtilityRequest) -> impl Future<Output = CommandResult> + Send;

    /// Creates one long-lived session in this supervised Utility process.
    fn create_session(
        &self,
        _request: UtilitySessionRequest,
    ) -> ApplicationResult<Box<dyn UtilitySession>> {
        Err(ApplicationError::new("Utility sessions are not supported"))
    }

    /// Performs service-specific cleanup before the process becomes idle.
    fn shutdown(
        &self,
        _context: ShutdownContext,
    ) -> impl Future<Output = ApplicationResult<()>> + Send {
        async { Ok(()) }
    }
}

pub(crate) trait ErasedUtility: Send + Sync {
    fn invoke(&self, request: UtilityRequest) -> BoxFuture<'_, CommandResult>;
    fn create_session(
        &self,
        request: UtilitySessionRequest,
    ) -> ApplicationResult<Box<dyn UtilitySession>>;
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, ApplicationResult<()>>;
}
impl<S: Utility> ErasedUtility for S {
    fn invoke(&self, request: UtilityRequest) -> BoxFuture<'_, CommandResult> {
        Box::pin(Utility::invoke(self, request))
    }
    fn create_session(
        &self,
        request: UtilitySessionRequest,
    ) -> ApplicationResult<Box<dyn UtilitySession>> {
        Utility::create_session(self, request)
    }
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, ApplicationResult<()>> {
        Box::pin(Utility::shutdown(self, context))
    }
}

type UtilityFactory =
    fn(UtilityContext) -> BoxFuture<'static, ApplicationResult<Box<dyn ErasedUtility>>>;

/// Static registration for an application-defined utility service.
#[derive(Clone, Copy)]
pub struct UtilityServiceRegistration {
    name: &'static str,
    factory: UtilityFactory,
    v8: Option<V8Factory>,
    interfaces: &'static [&'static str],
}

impl UtilityServiceRegistration {
    /// Registers a concrete utility service type.
    #[must_use]
    pub const fn new<S: Utility>(name: &'static str) -> Self {
        Self {
            name,
            factory: create_utility::<S>,
            v8: None,
            interfaces: &[],
        }
    }

    /// Registers a V8 resource factory without creating a realm or process.
    #[must_use]
    pub const fn v8<S: V8>(name: &'static str) -> Self {
        Self {
            name,
            factory: native_utility,
            v8: Some(create_v8::<S>),
            interfaces: &[],
        }
    }

    /// Registers multiple `ShellAPI` interfaces on one utility instance and process.
    #[must_use]
    pub const fn typed<S: Utility>(
        name: &'static str,
        interfaces: &'static [&'static str],
    ) -> Self {
        Self {
            interfaces,
            ..Self::new::<S>(name)
        }
    }

    /// Returns the canonical interfaces owned by this utility.
    #[must_use]
    pub const fn interfaces(&self) -> &'static [&'static str] {
        self.interfaces
    }

    /// Returns the native runtime discriminator advertised through the ABI.
    #[must_use]
    pub const fn runtime(&self) -> u32 {
        if self.v8.is_some() { 1 } else { 0 }
    }

    /// Returns the stable service name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn create(
        &self,
        context: UtilityContext,
    ) -> BoxFuture<'static, ApplicationResult<Box<dyn ErasedUtility>>> {
        (self.factory)(context)
    }
}

fn native_utility(
    _: UtilityContext,
) -> BoxFuture<'static, ApplicationResult<Box<dyn ErasedUtility>>> {
    Box::pin(async {
        Err(ApplicationError::new(
            "V8 utilities are created by the native backend",
        ))
    })
}

/// Stateful Browser-side resource provider for the native V8 utility backend.
///
/// The browser obtains module sources only when a named session is requested. This trait
/// creates one reusable Rust provider per name and Browser runtime; native creates
/// an independent JS isolate for each session. Provider methods may run concurrently.
pub trait V8: Send + Sync + 'static {
    /// Configures a lazy Browser-side provider before construction.
    /// Framework supplies entry.mjs and this service's manifest configuration.
    fn setup(builder: v8::Builder) -> ApplicationResult<v8::Builder>
    where
        Self: Sized,
    {
        Ok(builder)
    }

    /// Creates Browser-side provider state on first use. A failure is retained for this runtime.
    fn new(context: V8Context) -> ApplicationResult<Self>
    where
        Self: Sized;

    /// Resolves a canonical `app-scheme://service.utility/path.mjs`, HTTPS or file URL in the browser's worker runtime.
    ///
    /// Application resources use the app scheme and `name.utility` host. HTTPS/file
    /// URLs still pass through this provider; it owns I/O and access policy.
    /// Return Some(source) to override app.assets, None to fall back, or Err to
    /// fail loading without fallback. The default uses app.assets exclusively.
    /// Called on demand for an entry or static import. Each URL is requested
    /// once per session; the utility caches compiled modules and never reads files.
    fn resolve(
        &self,
        _request: v8::ModuleRequest,
    ) -> impl Future<Output = Result<Option<v8::ModuleSource>, v8::Error>> + Send {
        async { Ok(None) }
    }

    /// Releases shared Browser-side resources after module workers have drained.
    fn shutdown(
        &self,
        _context: ShutdownContext,
    ) -> impl Future<Output = ApplicationResult<()>> + Send {
        async { Ok(()) }
    }
}

pub(crate) trait ErasedV8: Send + Sync {
    fn entry(&self) -> &str;
    fn resolve(
        &self,
        request: v8::ModuleRequest,
    ) -> BoxFuture<'_, Result<Option<v8::ModuleSource>, v8::Error>>;
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, ApplicationResult<()>>;
}
struct ConfiguredV8<S> {
    provider: S,
    entry: String,
}

impl<S: V8> ErasedV8 for ConfiguredV8<S> {
    fn entry(&self) -> &str {
        &self.entry
    }
    fn resolve(
        &self,
        request: v8::ModuleRequest,
    ) -> BoxFuture<'_, Result<Option<v8::ModuleSource>, v8::Error>> {
        Box::pin(self.provider.resolve(request))
    }
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, ApplicationResult<()>> {
        Box::pin(self.provider.shutdown(context))
    }
}

/// Resource construction for utilities executed by the native V8 backend.
pub mod v8;

/// Browser client for explicitly registered Rust and V8 utilities.
#[derive(Clone)]
pub struct Host {
    sessions: crate::UtilitySessionHost,
    providers: Arc<V8Providers>,
}

impl Host {
    pub(crate) fn new(sessions: crate::UtilitySessionHost, providers: Arc<V8Providers>) -> Self {
        Self {
            sessions,
            providers,
        }
    }

    /// Returns the binary session client for registered Rust services.
    #[must_use]
    pub fn sessions(&self) -> crate::UtilitySessionHost {
        self.sessions.clone()
    }

    /// Opens a named V8 definition lazily in a supervised utility process.
    pub fn create_v8<C, O>(
        &self,
        name: &str,
        on_closed: C,
        on_created: O,
    ) -> ApplicationResult<crate::JavaScriptExtensionSessionHandle>
    where
        C: FnOnce(crate::UtilitySessionClosed) + Send + 'static,
        O: FnOnce(Result<(), crate::UtilitySessionHostError>) + Send + 'static,
    {
        let provider = self.providers.get(name)?;
        let entry = provider.entry();
        crate::JavaScriptExtensionHost::new(self.sessions.clone())
            .create_resolved(name, entry, on_closed, on_created)
            .map_err(|error| ApplicationError::new(error.to_string()))
    }
}

fn create_utility<S: Utility>(
    mut context: UtilityContext,
) -> BoxFuture<'static, ApplicationResult<Box<dyn ErasedUtility>>> {
    Box::pin(async move {
        let builder = Builder::default().configuration(context.configuration().to_vec());
        context.configuration = S::setup(builder)?.finish()?.into();
        S::new(context)
            .await
            .map(|service| Box::new(service) as Box<dyn ErasedUtility>)
    })
}

fn create_v8<S: V8>(mut context: V8Context) -> ApplicationResult<Arc<dyn ErasedV8>> {
    let builder = v8::Builder::default().configuration(context.configuration().to_vec());
    let scheme = v8::app_scheme(context.application_configuration())
        .ok_or_else(|| ApplicationError::new("application scheme is unavailable"))?;
    let (configuration, entry) = S::setup(builder)?.finish(&scheme, context.name())?;
    context.configuration = configuration.into();
    let provider = S::new(context)?;
    Ok(Arc::new(ConfiguredV8 { provider, entry }))
}

impl std::fmt::Debug for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host").finish_non_exhaustive()
    }
}

/// Rust utility implementation and session contracts.
pub mod rust {
    pub use super::{Builder, Utility, UtilityContext, UtilityRequest, UtilityServiceRegistration};
    pub use crate::utility_host::{
        UtilitySessionClosed, UtilitySessionHandle, UtilitySessionHost, UtilitySessionHostError,
        UtilitySessionOptions,
    };
    pub use crate::utility_session::{
        DEFAULT_UTILITY_SESSION_CREDITS, MAX_UTILITY_SESSION_CREDITS,
        MAX_UTILITY_SESSION_EVENT_BYTES, UtilitySession, UtilitySessionCommand,
        UtilitySessionEmitter, UtilitySessionEvent, UtilitySessionFlowControl,
        UtilitySessionFlowError, UtilitySessionFuture, UtilitySessionRequest,
    };
}

/// Browser-side API for utilities executed by the native V8 host.
pub mod javascript {
    pub use crate::javascript_extension::{
        JavaScriptExtensionError, JavaScriptExtensionHost, JavaScriptExtensionPackage,
        JavaScriptExtensionSessionHandle,
    };
}

/// Metadata emitted by the utility attribute and consumed by the main macro.
#[doc(hidden)]
pub trait UtilityDefinition: Send + Sync + 'static {
    const REGISTRATION: UtilityServiceRegistration;
}

pub(crate) const fn is_reserved_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    let reserved = b"tuffite_js_extension";
    if bytes.len() != reserved.len() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != reserved[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Compile-time validation for the explicit utility list.
#[doc(hidden)]
pub const fn validate_utility_names(values: &[UtilityServiceRegistration]) {
    let mut i = 0;
    while i < values.len() {
        assert!(!is_reserved_name(values[i].name()), "reserved utility name");
        let name = values[i].name().as_bytes();
        assert!(
            !name.is_empty() && name.len() <= 128,
            "invalid utility name"
        );
        assert!(
            is_valid_service_name(values[i].name()),
            "invalid utility name"
        );
        let mut j = 0;
        while j < i {
            let other = values[j].name().as_bytes();
            let mut same = name.len() == other.len();
            let mut k = 0;
            while same && k < name.len() {
                same = name[k] == other[k];
                k += 1;
            }
            assert!(!same, "duplicate utility name in tuffite::main");
            j += 1;
        }
        i += 1;
    }
}

pub(crate) const fn is_valid_service_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > 128 {
        return false;
    }
    let mut start = true;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if !start && (b == b'.' || (b == b':' && i + 1 < bytes.len() && bytes[i + 1] == b':')) {
            if b == b':' {
                i += 1;
            }
            start = true;
        } else if b.is_ascii_alphabetic()
            || b == b'_'
            || b == b'$'
            || (!start && b.is_ascii_digit())
        {
            start = false;
        } else {
            return false;
        }
        i += 1;
    }
    !start
}

impl std::fmt::Debug for V8Providers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("V8Providers")
    }
}
