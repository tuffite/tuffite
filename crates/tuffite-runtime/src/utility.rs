//! Sandboxed utility implementations and browser-side clients.
//!
//! Rust and V8 definitions share the utility attribute. Rust implementations
//! use [`Utility`]; V8 definitions use [`V8`] and [`crate::Resource`].
//! Both use supervised Chromium utility processes and session transport.

use crate::lifecycle::BoxFuture;
use crate::supervisor::CancellationToken;
use crate::utility_session::{UtilitySession, UtilitySessionRequest};
use crate::{CommandResult, Error, Result};
use crate::{ShutdownContext, ShutdownReason, StopSignal};
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Instant;

mod builder;
pub use builder::{Builder, Interface, InterfaceFor, InterfaceRegistry};

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
    context_slot: crate::context_data::Slot,
}

impl UtilityContext {
    pub fn downcast<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.context_slot.require()
    }
    #[must_use]
    pub fn try_downcast<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.context_slot.get()
    }

    pub(crate) fn new(name: &str, configuration: Vec<u8>, cancellation: CancellationToken) -> Self {
        Self {
            name: name.into(),
            configuration: configuration.into(),
            cancellation,
            context_slot: crate::context_data::Slot::default(),
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
    /// Setup starts with empty startup data. Session init is independent.
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
    state: crate::context_data::Slot,
    cancellation: CancellationToken,
    context_slot: crate::context_data::Slot,
}

impl V8Context {
    /// Returns the registered service name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns this service's frozen startup data after `setup`.
    /// Setup starts with empty startup data.
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

    /// Restores the owning Browser's business context. No Rust state enters the JS isolate.
    pub fn browser_context<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.state.require()
    }
    pub fn downcast<T: Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.context_slot.require()
    }
    #[must_use]
    pub fn try_downcast<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.context_slot.get()
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

type V8Factory = fn(V8Context) -> Result<Arc<dyn ErasedV8>>;
type ProviderCell = OnceLock<Result<Arc<dyn ErasedV8>>>;

pub(crate) struct V8Providers {
    configuration: Arc<[u8]>,
    executor: crate::ChromiumExecutor,
    state: crate::context_data::Slot,
    cancellation: CancellationToken,
    providers: Mutex<HashMap<String, Arc<ProviderCell>>>,
    sessions: Mutex<HashMap<(String, u64), CancellationToken>>,
    construction: RwLock<()>,
}

impl V8Providers {
    pub(crate) fn new(
        configuration: Arc<[u8]>,
        executor: crate::ChromiumExecutor,
        state: crate::context_data::Slot,
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

    pub(crate) fn get(&self, name: &str) -> Result<Arc<dyn ErasedV8>> {
        // Fence constructors started through Host clones as well as resolver workers.
        let _construction = self
            .construction
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.cancellation.is_cancelled() {
            return Err(Error::new("Browser is stopping"));
        }
        let registration = crate::builder::utility_definition(name)
            .ok_or_else(|| Error::new("unknown utility service"))?;
        let factory = registration
            .v8
            .ok_or_else(|| Error::new("utility is not a V8 service"))?;
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
                configuration: Arc::from([]),
                application_configuration: self.configuration.clone(),
                executor: self.executor.clone(),
                state: self.state.clone(),
                context_slot: crate::context_data::Slot::default(),
                cancellation: self.cancellation.clone(),
            };
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| factory(context)))
                .unwrap_or_else(|_| Err(Error::new("V8 provider constructor panicked")))
        });
        result
            .as_ref()
            .map(Arc::clone)
            .map_err(|error| Error::new(error.to_string()))
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

    pub(crate) fn release(&self) -> Result<()> {
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
            Err(Error::new("V8 provider cleanup failed"))
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn resolve_source(
    provider: &dyn ErasedV8,
    name: &str,
    request: v8::ModuleRequest,
) -> Result<Option<Vec<u8>>> {
    let url = request.url().to_owned();
    v8::validate_url(name, &url, None).map_err(|error| Error::new(error.to_string()))?;
    let parsed = crate::Url::parse(&url)?;
    let context = crate::ResolveContext::module(
        url,
        request.session_id(),
        request.deadline(),
        request.stop_signal(),
    );
    match provider.resolve(&parsed, context).await? {
        crate::Resolution::Pass => Ok(None),
        crate::Resolution::Resource(resource) => resource.module_bytes().map(Some),
        crate::Resolution::Consumed | crate::Resolution::Discard => Err(Error::new(
            "V8 module resolution requires a Resource or Pass outcome",
        )),
    }
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
    /// Configures this child before state construction. Framework supplies service startup data.
    fn setup(builder: Builder<Self>) -> Result<Builder<Self>>
    where
        Self: Sized,
    {
        Ok(builder)
    }

    /// Asynchronously creates state inside the launched sandboxed child.
    /// Ready is reported only after this future succeeds; failure owns its partial rollback.
    fn new(context: UtilityContext) -> impl Future<Output = Result<Self>> + Send
    where
        Self: Sized;

    /// Handles one binary request without blocking the Chromium utility sequence.
    /// Implementations should observe `request.is_cancelled()` during long work.
    fn invoke(&self, _request: UtilityRequest) -> impl Future<Output = CommandResult> + Send {
        async { Err(crate::CommandError::UnknownCommand) }
    }

    /// Creates one long-lived session in this supervised Utility process.
    fn create_session(&self, _request: UtilitySessionRequest) -> Result<Box<dyn UtilitySession>> {
        Err(Error::new("Utility sessions are not supported"))
    }

    /// Performs service-specific cleanup before the process becomes idle.
    fn shutdown(&self, _context: ShutdownContext) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

pub(crate) trait ErasedUtility: Send + Sync {
    fn invoke(&self, request: UtilityRequest) -> BoxFuture<'_, CommandResult>;
    fn create_session(&self, request: UtilitySessionRequest) -> Result<Box<dyn UtilitySession>>;
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, Result<()>>;
}
pub(crate) struct ConfiguredUtility<S> {
    pub(crate) implementation: S,
    pub(crate) interfaces: InterfaceRegistry<S>,
    pub(crate) _business_context: crate::context_data::Value,
}
impl<S: Utility> ErasedUtility for ConfiguredUtility<S> {
    fn invoke(&self, request: UtilityRequest) -> BoxFuture<'_, CommandResult> {
        Box::pin(async move {
            if self.interfaces.interfaces.is_empty() {
                return self.implementation.invoke(request).await;
            }
            Ok(dispatch_typed(&request, |command| {
                self.interfaces
                    .interfaces
                    .iter()
                    .find(|interface| interface.descriptors.iter().any(|d| d.id() == command.id()))
                    .map_or_else(
                        || Err(crate::CommandError::UnknownCommand),
                        |interface| (interface.dispatch)(&self.implementation, command),
                    )
            }))
        })
    }
    fn create_session(&self, request: UtilitySessionRequest) -> Result<Box<dyn UtilitySession>> {
        Utility::create_session(&self.implementation, request)
    }
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, Result<()>> {
        Box::pin(Utility::shutdown(&self.implementation, context))
    }
}

type UtilityFactory = fn(UtilityContext) -> BoxFuture<'static, Result<Box<dyn ErasedUtility>>>;

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
    ) -> BoxFuture<'static, Result<Box<dyn ErasedUtility>>> {
        (self.factory)(context)
    }
}

fn native_utility(_: UtilityContext) -> BoxFuture<'static, Result<Box<dyn ErasedUtility>>> {
    Box::pin(async { Err(Error::new("V8 utilities are created by the native backend")) })
}

/// Stateful Browser-side resource provider for the native V8 utility backend.
///
/// The browser obtains module sources only when a named session is requested. This trait
/// creates one reusable Rust provider per name and Browser runtime; native creates
/// an independent JS isolate for each session. Provider methods may run concurrently.
pub trait V8: Send + Sync + 'static {
    /// Configures a lazy Browser-side provider before construction.
    /// Framework supplies entry.mjs and empty startup data.
    fn setup(builder: v8::Builder) -> Result<v8::Builder>
    where
        Self: Sized,
    {
        Ok(builder)
    }

    /// Creates Browser-side provider state on first use. A failure is retained for this runtime.
    fn new(context: V8Context) -> Result<Self>
    where
        Self: Sized;

    /// Resolves a canonical `app-scheme://service.utility/path.mjs`, HTTPS or file URL in the browser's worker runtime.
    ///
    /// Application resources use the app scheme and `name.utility` host. HTTPS/file
    /// URLs still pass through this provider; it owns I/O and access policy.
    /// Return Resource to supply UTF-8 ECMAScript bytes/a file, Pass for packaged
    /// assets, or an error to fail without fallback. Consumed/Discard reject a module.
    fn resolve(
        &self,
        _url: &crate::Url,
        _context: crate::ResolveContext,
    ) -> impl Future<Output = Result<crate::Resolution>> + Send {
        async { Ok(crate::Resolution::Pass) }
    }

    /// Releases shared Browser-side resources after module workers have drained.
    fn shutdown(&self, _context: ShutdownContext) -> impl Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}

pub(crate) trait ErasedV8: Send + Sync {
    fn entry(&self) -> &str;
    fn resolve<'a>(
        &'a self,
        url: &'a crate::Url,
        context: crate::ResolveContext,
    ) -> BoxFuture<'a, Result<crate::Resolution>>;
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, Result<()>>;
}
struct ConfiguredV8<S> {
    provider: S,
    entry: String,
    pub(crate) _business_context: crate::context_data::Value,
}

impl<S: V8> ErasedV8 for ConfiguredV8<S> {
    fn entry(&self) -> &str {
        &self.entry
    }
    fn resolve<'a>(
        &'a self,
        url: &'a crate::Url,
        context: crate::ResolveContext,
    ) -> BoxFuture<'a, Result<crate::Resolution>> {
        Box::pin(self.provider.resolve(url, context))
    }
    fn shutdown(&self, context: ShutdownContext) -> BoxFuture<'_, Result<()>> {
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
    ) -> Result<crate::JavaScriptExtensionSessionHandle>
    where
        C: FnOnce(crate::UtilitySessionClosed) + Send + 'static,
        O: FnOnce(Result<(), crate::UtilitySessionHostError>) + Send + 'static,
    {
        let provider = self.providers.get(name)?;
        let entry = provider.entry();
        crate::JavaScriptExtensionHost::new(self.sessions.clone())
            .create_resolved(name, entry, on_closed, on_created)
            .map_err(|error| Error::new(error.to_string()))
    }
}

fn create_utility<S: Utility>(
    mut context: UtilityContext,
) -> BoxFuture<'static, Result<Box<dyn ErasedUtility>>> {
    Box::pin(async move {
        let builder = Builder::default()
            .configuration(context.configuration().to_vec())
            .named(context.name());
        let builder = S::setup(builder)?;
        let interfaces = builder.interfaces;
        let configuration = builder.configuration;
        if configuration.len() > 64 * 1024 {
            return Err(Error::new("Utility configuration exceeds 64 KiB"));
        }
        if let Some(expected) = crate::builder::utility_definition(context.name()) {
            let names: Vec<_> = interfaces
                .interfaces
                .iter()
                .map(|interface| interface.name)
                .collect();
            if builder.name.as_deref() != Some(expected.name()) || names != expected.interfaces() {
                return Err(Error::new("Utility setup changed frozen registration"));
            }
        }
        context.configuration = configuration.into();
        let value = builder.context_factory.build(context.clone())?;
        context.context_slot.install(&value)?;
        S::new(context).await.map(|implementation| {
            Box::new(ConfiguredUtility {
                implementation,
                _business_context: value,
                interfaces,
            }) as Box<dyn ErasedUtility>
        })
    })
}

fn create_v8<S: V8>(mut context: V8Context) -> Result<Arc<dyn ErasedV8>> {
    let builder = v8::Builder::default()
        .configuration(context.configuration().to_vec())
        .named(context.name());
    let schemes = v8::app_schemes(context.application_configuration())
        .ok_or_else(|| Error::new("application scheme is unavailable"))?;
    let builder = S::setup(builder)?;
    if builder.service_name() != Some(context.name()) {
        return Err(Error::new("V8 setup changed the frozen service name"));
    }
    let context_factory = builder.context_factory.clone();
    let (configuration, entry) = builder.finish(&schemes, context.name())?;
    context.configuration = configuration.into();
    let value = context_factory.build(context.clone())?;
    context.context_slot.install(&value)?;
    let provider = S::new(context)?;
    Ok(Arc::new(ConfiguredV8 {
        provider,
        entry,
        _business_context: value,
    }))
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

pub(crate) fn register<S: Utility>(startup: crate::Startup) -> Result<UtilityServiceRegistration> {
    let mut input = Builder::<S>::default().name(std::any::type_name::<S>());
    input.startup = startup;
    let builder = S::setup(input)?;
    if builder.configuration.len() > 64 * 1024 {
        return Err(Error::new("Utility configuration exceeds 64 KiB"));
    }
    let name = Box::leak(
        builder
            .name
            .ok_or_else(|| Error::new("Utility name is required"))?
            .into_boxed_str(),
    );
    let interfaces = builder
        .interfaces
        .interfaces
        .iter()
        .map(|interface| interface.name)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Ok(UtilityServiceRegistration::typed::<S>(
        name,
        Box::leak(interfaces),
    ))
}
pub(crate) fn register_v8<S: V8>(startup: crate::Startup) -> Result<UtilityServiceRegistration> {
    let mut input = v8::Builder::default().name(std::any::type_name::<S>());
    input.configuration.startup = startup;
    let builder = S::setup(input)?;
    let name = Box::leak(
        builder
            .configuration
            .name
            .ok_or_else(|| Error::new("V8 Utility name is required"))?
            .into_boxed_str(),
    );
    Ok(UtilityServiceRegistration::v8::<S>(name))
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

#[cfg(test)]
mod resolution_tests {
    use super::*;
    struct Provider;
    impl V8 for Provider {
        fn new(_: V8Context) -> Result<Self> {
            Ok(Self)
        }
        fn resolve(
            &self,
            url: &crate::Url,
            context: crate::ResolveContext,
        ) -> impl Future<Output = Result<crate::Resolution>> + Send {
            assert_eq!(context.session_id(), Some(41));
            assert!(context.deadline() > Instant::now());
            std::future::ready(Ok(match url.path() {
                "/valid.mjs" => crate::Resolution::Resource(crate::Resource::bytes(
                    b"export default 1;".to_vec(),
                )),
                "/binary.mjs" => crate::Resolution::Resource(crate::Resource::bytes(vec![255])),
                "/consume.mjs" => crate::Resolution::Consumed,
                "/discard.mjs" => crate::Resolution::Discard,
                _ => crate::Resolution::Pass,
            }))
        }
    }
    #[test]
    fn shared_resolver_contract_enforces_module_specific_outcomes() {
        let provider = ConfiguredV8 {
            provider: Provider,
            entry: String::new(),
            _business_context: Arc::new(()),
        };
        let resolve = |path| {
            futures_executor::block_on(resolve_source(
                &provider,
                "parser",
                v8::ModuleRequest::new(
                    format!("myapp://parser.utility/{path}"),
                    41,
                    StopSignal(CancellationToken::new()),
                ),
            ))
        };
        assert_eq!(resolve("valid.mjs").unwrap().unwrap(), b"export default 1;");
        assert!(resolve("fallback.mjs").unwrap().is_none());
        for path in ["binary.mjs", "consume.mjs", "discard.mjs"] {
            assert!(resolve(path).is_err(), "{path}");
        }
    }
}
