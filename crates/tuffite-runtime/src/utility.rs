//! Sandboxed utility implementations and browser-side clients.
//!
//! Rust and V8 definitions share the utility attribute. Rust implementations
//! use [`Utility`]; V8 definitions use [`V8`] and [`v8::ModuleSource`].
//! Both use supervised Chromium utility processes and session transport.

use crate::supervisor::CancellationToken;
use crate::utility_session::{UtilitySession, UtilitySessionRequest};
use crate::{ApplicationError, ApplicationResult, CommandResult};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

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

    /// Returns only this service's serialized startup configuration, not the app manifest.
    /// Missing configuration is an empty byte slice. Session init is independent.
    #[must_use]
    pub fn configuration(&self) -> &[u8] {
        &self.configuration
    }

    /// Reports service shutdown; retained context clones observe the same signal.
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

    /// Returns this service's serialized startup configuration.
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
    pub fn is_stopping(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}

type V8Factory = fn(V8Context) -> ApplicationResult<Arc<dyn V8>>;
type ProviderCell = OnceLock<ApplicationResult<Arc<dyn V8>>>;

pub(crate) struct V8Providers {
    configuration: Arc<[u8]>,
    executor: crate::ChromiumExecutor,
    state: crate::browser::SharedState,
    cancellation: CancellationToken,
    providers: Mutex<HashMap<String, Arc<ProviderCell>>>,
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
        }
    }

    pub(crate) fn get(&self, name: &str) -> ApplicationResult<Arc<dyn V8>> {
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

    pub(crate) fn stop(&self) {
        self.cancellation.cancel();
    }

    pub(crate) fn release(&self) {
        let providers = std::mem::take(
            &mut *self
                .providers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        // User destructors run without the registry lock after resolver workers are joined.
        drop(providers);
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

pub(crate) fn resolve_source(
    provider: &dyn V8,
    name: &str,
    url: &str,
) -> ApplicationResult<Option<Vec<u8>>> {
    v8::validate_url(name, url, None).map_err(|error| ApplicationError::new(error.to_string()))?;
    let Some(module) = provider
        .resolve(url)
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

/// Future returned by an asynchronous utility-service invocation.
pub type UtilityFuture<'a> = Pin<Box<dyn Future<Output = CommandResult> + Send + 'a>>;

/// Stateful Rust implementation hosted in a sandboxed Chromium process.
pub trait Utility: Send + Sync + 'static {
    /// Creates service state after Chromium launches the utility process.
    fn new(context: UtilityContext) -> ApplicationResult<Self>
    where
        Self: Sized;

    /// Handles one binary request without blocking the Chromium utility sequence.
    /// Implementations should observe `request.is_cancelled()` during long work.
    fn invoke(&self, request: UtilityRequest) -> UtilityFuture<'_>;

    /// Creates one long-lived session in this supervised Utility process.
    fn create_session(
        &self,
        _request: UtilitySessionRequest,
    ) -> ApplicationResult<Box<dyn UtilitySession>> {
        Err(ApplicationError::new("Utility sessions are not supported"))
    }

    /// Performs service-specific cleanup before the process becomes idle.
    fn shutdown(&self) -> ApplicationResult<()> {
        Ok(())
    }
}

type UtilityFactory = fn(UtilityContext) -> ApplicationResult<Box<dyn Utility>>;

/// Static registration for an application-defined utility service.
#[derive(Clone, Copy)]
pub struct UtilityServiceRegistration {
    name: &'static str,
    factory: UtilityFactory,
    v8: Option<V8Factory>,
}

impl UtilityServiceRegistration {
    /// Registers a concrete utility service type.
    #[must_use]
    pub const fn new<S: Utility>(name: &'static str) -> Self {
        Self {
            name,
            factory: create_utility::<S>,
            v8: None,
        }
    }

    /// Registers a V8 resource factory without creating a realm or process.
    #[must_use]
    pub const fn v8<S: V8>(name: &'static str) -> Self {
        Self {
            name,
            factory: native_utility,
            v8: Some(create_v8::<S>),
        }
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

    pub(crate) fn create(&self, context: UtilityContext) -> ApplicationResult<Box<dyn Utility>> {
        (self.factory)(context)
    }
}

fn native_utility(_: UtilityContext) -> ApplicationResult<Box<dyn Utility>> {
    Err(ApplicationError::new(
        "V8 utilities are created by the native backend",
    ))
}

/// Stateful Browser-side resource provider for the native V8 utility backend.
///
/// The browser obtains module sources only when a named session is requested. This trait
/// creates one reusable Rust provider per name and Browser runtime; native creates
/// an independent JS isolate for each session. Provider methods may run concurrently.
pub trait V8: Send + Sync + 'static {
    /// Creates Browser-side provider state on first use. A failure is retained for this runtime.
    fn new(context: V8Context) -> ApplicationResult<Self>
    where
        Self: Sized;

    /// Returns a relative entry path or an absolute HTTPS/file module URL.
    fn entry(&self) -> &str;

    /// Resolves a canonical `app-scheme://service.utility/path.mjs`, HTTPS or file URL in the browser's worker runtime.
    ///
    /// Application resources use the app scheme and `name.utility` host. HTTPS/file
    /// URLs still pass through this provider; it owns I/O and access policy.
    /// Return Some(source) to override app.assets, None to fall back, or Err to
    /// fail loading without fallback. The default uses app.assets exclusively.
    /// Called on demand for an entry or static import. Each URL is requested
    /// once per session; the utility caches compiled modules and never reads files.
    fn resolve(&self, _url: &str) -> Result<Option<v8::ModuleSource>, v8::Error> {
        Ok(None)
    }
}

/// Resource construction for utilities executed by the native V8 backend.
pub mod v8;

/// Browser client for explicitly registered Rust and V8 utilities.
#[derive(Clone)]
pub struct Host {
    sessions: crate::UtilitySessionHost,
    app_scheme: Option<String>,
    providers: Arc<V8Providers>,
}

impl Host {
    pub(crate) fn new(
        sessions: crate::UtilitySessionHost,
        app_scheme: Option<String>,
        providers: Arc<V8Providers>,
    ) -> Self {
        Self {
            sessions,
            app_scheme,
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
        if url::Url::parse(entry).is_err() {
            crate::javascript_extension::validate_module_name(entry)
                .map_err(|error| ApplicationError::new(error.to_string()))?;
        }
        let scheme = self
            .app_scheme
            .as_deref()
            .ok_or_else(|| ApplicationError::new("application scheme is unavailable"))?;
        let entry = v8::entry_url(scheme, name, entry)
            .map_err(|error| ApplicationError::new(error.to_string()))?;
        crate::JavaScriptExtensionHost::new(self.sessions.clone())
            .create_resolved(name, &entry, on_closed, on_created)
            .map_err(|error| ApplicationError::new(error.to_string()))
    }
}

fn create_utility<S: Utility>(context: UtilityContext) -> ApplicationResult<Box<dyn Utility>> {
    S::new(context).map(|service| Box::new(service) as Box<dyn Utility>)
}

fn create_v8<S: V8>(context: V8Context) -> ApplicationResult<Arc<dyn V8>> {
    S::new(context).map(|provider| Arc::new(provider) as Arc<dyn V8>)
}

impl std::fmt::Debug for Host {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Host")
            .field("app_scheme", &self.app_scheme)
            .finish_non_exhaustive()
    }
}

/// Rust utility implementation and session contracts.
pub mod rust {
    pub use super::{
        Utility, UtilityContext, UtilityFuture, UtilityRequest, UtilityServiceRegistration,
    };
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
        let mut k = 0;
        while k < name.len() {
            let b = name[k];
            assert!(
                b.is_ascii_alphabetic() || b == b'_' || b == b'$' || (k > 0 && b.is_ascii_digit()),
                "invalid utility name"
            );
            k += 1;
        }
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

impl std::fmt::Debug for V8Providers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("V8Providers")
    }
}
