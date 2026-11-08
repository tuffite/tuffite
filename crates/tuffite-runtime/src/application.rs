//! Application context, lifecycle and startup configuration.

pub(crate) mod builder;

use crate::{Browser, CommandError, Result, UtilityServiceRegistration};
use tuffite_abi::{Application as AbiApplication, Header};

/// Application-wide capabilities. Clones are handles, not native ownership.
#[derive(Clone, Debug)]
pub struct ApplicationContext {
    context: std::sync::Arc<crate::BrowserContext>,
}
impl AsRef<Self> for ApplicationContext {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl ApplicationContext {
    #[doc(hidden)]
    #[must_use]
    pub fn remoting_host(&self) -> crate::native_remoting::Host {
        self.context.remoting_host()
    }
    pub(crate) fn new(context: crate::BrowserContext) -> Self {
        Self {
            context: std::sync::Arc::new(context),
        }
    }
    #[must_use]
    pub fn configuration(&self) -> &[u8] {
        self.context.configuration()
    }
    #[must_use]
    pub fn executor(&self) -> crate::ChromiumExecutor {
        self.context.executor()
    }
    #[must_use]
    pub fn stop_signal(&self) -> crate::StopSignal {
        self.context.stop_signal()
    }
    pub fn downcast<T: std::any::Any + Send + Sync>(&self) -> Result<std::sync::Arc<T>> {
        self.context.downcast()
    }
    #[must_use]
    pub fn try_downcast<T: std::any::Any + Send + Sync>(&self) -> Option<std::sync::Arc<T>> {
        self.context.try_downcast()
    }
    #[must_use]
    pub fn profiles(&self) -> crate::ProfileManager {
        crate::ProfileManager::new(self.context.context_client())
    }
    #[must_use]
    pub fn shortcuts(&self) -> crate::Shortcuts {
        crate::Shortcuts::new(
            self.context.context_client(),
            self.context.shortcut_callbacks.clone(),
        )
    }
    #[must_use]
    pub fn windows(&self) -> crate::Windows {
        crate::Windows::new(self.context.context_client())
    }
    #[must_use]
    pub fn events(&self) -> crate::EventEmitter {
        self.context.events()
    }
    #[must_use]
    pub fn utilities(&self) -> crate::utility::Host {
        self.context.utilities()
    }
    #[must_use]
    pub fn utility_sessions(&self) -> crate::UtilitySessionHost {
        self.context.utility_sessions()
    }
    #[must_use]
    pub fn javascript_extensions(&self) -> crate::JavaScriptExtensionHost {
        crate::JavaScriptExtensionHost::new(self.utility_sessions())
    }
}

/// One application backend, shared by all of its loaded profiles.
pub trait Application: Send + Sized + 'static {
    /// Collects immutable declarations before Chromium starts. No native
    /// context exists yet; this must only build definitions in each process.
    fn setup(builder: crate::app::Builder<Self>) -> Result<crate::app::Builder<Self>> {
        Ok(builder)
    }
    fn new(context: ApplicationContext) -> Result<Self>;
    /// Receives activations without a shortcut-specific callback.
    fn shortcut(
        &mut self,
        _event: crate::ShortcutEvent,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
    fn initialize(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
    fn shutdown(
        &mut self,
        _context: crate::ShutdownContext,
    ) -> impl std::future::Future<Output = Result<()>> + Send {
        async { Ok(()) }
    }
}
/// Result returned by a binary `ShellAPI` command.
pub type CommandResult = std::result::Result<Vec<u8>, CommandError>;

/// Internal frozen registration view used by ABI adapters.
#[doc(hidden)]
pub trait Registrations: 'static {
    #[must_use]
    fn utility_services() -> &'static [UtilityServiceRegistration];
}

static UTILITY_SESSION_SERVICE: tuffite_abi::UtilitySessionService =
    tuffite_abi::UtilitySessionService {
        header: Header::for_type::<tuffite_abi::UtilitySessionService>(),
        create: Some(crate::utility_entry::create_utility_session),
        send: Some(crate::utility_entry::send_utility_session),
        acknowledge: Some(crate::utility_entry::acknowledge_utility_session),
        cancel: Some(crate::utility_entry::cancel_utility_session),
        close: Some(crate::utility_entry::close_utility_session),
    };

/// Builds the versioned function table exported by an application library.
#[doc(hidden)]
pub const fn for_application<B: Browser, R: Registrations>() -> AbiApplication {
    AbiApplication {
        header: Header::for_type::<AbiApplication>(),
        start_profile: None,
        command_scope: None,
        configuration: None,
        resolve_mojom_method: None,
        dispatch_mojom: Some(crate::abi::dispatch_browser_mojom),
        invoke_utility_mojom: Some(crate::utility_entry::invoke_utility_mojom),
        setup_browser: Some(crate::abi::setup_browser::<B>),
        initialize_browser: Some(crate::abi::initialize_browser),
        cancel_v8_session: Some(crate::abi::cancel_v8_session),
        prepare_browser: None,
        prepare_process: None,
        utility_runtime: Some(crate::utility_entry::utility_runtime::<R>),
        resolve_v8: Some(crate::abi::resolve_v8),
        utility_start_with_context: Some(crate::utility_entry::start_utility_with_context::<R>),
        resolve_utility_route: Some(crate::utility_entry::resolve_utility_route::<R>),
        start: Some(crate::abi::start_browser::<B>),
        dispatch: Some(crate::abi::dispatch_browser),
        stop: Some(crate::abi::stop_browser),
        utility: tuffite_abi::UtilityService {
            is_registered: Some(crate::utility_entry::is_utility_registered::<R>),
            start: Some(crate::utility_entry::start_utility::<R>),
            invoke: Some(crate::utility_entry::invoke_utility),
            cancel: Some(crate::utility_entry::cancel_utility),
            stop: Some(crate::utility_entry::stop_utility),
            session: &raw const UTILITY_SESSION_SERVICE,
        },
    }
}
