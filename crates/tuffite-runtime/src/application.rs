//! Application-wide registration and process entry selection.

use crate::{Browser, CommandError, UtilityServiceRegistration};
use tuffite_abi::{Application as AbiApplication, Header};

/// Failure while creating or shutting down an application.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("{message}")]
pub struct ApplicationError {
    message: String,
}

impl ApplicationError {
    /// Creates an application lifecycle error.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Result returned by application lifecycle methods.
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// Result returned by a binary `ShellAPI` command.
pub type CommandResult = Result<Vec<u8>, CommandError>;

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
            session: &UTILITY_SESSION_SERVICE,
        },
    }
}
