//! Tuffite's Rust control plane and its stable C ABI entry points.

pub mod abi;
mod application;
pub mod browser;
mod builder;
#[doc(hidden)]
pub mod codec;
mod dispatch;
mod executor;
mod handle;
mod javascript_extension;
pub mod protocol;
mod runtime;
mod shell_api;
mod supervisor;
pub mod utility;
mod utility_entry;
mod utility_host;
mod utility_session;

pub use application::{ApplicationError, ApplicationResult, CommandResult};
pub use browser::{Browser, BrowserContext, Command, EventEmitter};
pub use builder::CommandDescriptor;
/// Application entry assembly and browser launch hooks.
pub mod app {
    pub use crate::builder::{Builder, ProcessKind, ProcessLaunch};
}
#[doc(hidden)]
pub use builder::{CommandHandler, CommandRegistry};
pub use codec::{Bytes, Timestamp};
pub use executor::{ChromiumExecutor, ExecutorError, TaskPriority};
#[cfg(feature = "async")]
pub use executor::{ChromiumTask, ChromiumTaskError};
pub use handle::{HandleKind, RawHandle};
pub use javascript_extension::{
    JavaScriptExtensionError, JavaScriptExtensionHost, JavaScriptExtensionPackage,
    JavaScriptExtensionSessionHandle,
};
#[doc(hidden)]
pub use protocol::CommandError;
pub use runtime::{Runtime, RuntimeError, RuntimeState, TaskContext, TaskHandle};
#[doc(hidden)]
pub use serde;
pub use shell_api::{ShellApiError, ShellApiHost, ShellApiRequest};
pub use utility::{
    Utility, UtilityContext, UtilityFuture, UtilityRequest, UtilityServiceRegistration, V8Context,
};
pub use utility_host::{
    UtilitySessionClosed, UtilitySessionHandle, UtilitySessionHost, UtilitySessionHostError,
    UtilitySessionOptions,
};
pub use utility_session::{
    DEFAULT_UTILITY_SESSION_CREDITS, MAX_UTILITY_SESSION_CREDITS, MAX_UTILITY_SESSION_EVENT_BYTES,
    UtilitySession, UtilitySessionCommand, UtilitySessionEmitter, UtilitySessionEvent,
    UtilitySessionFlowControl, UtilitySessionFlowError, UtilitySessionFuture,
    UtilitySessionRequest,
};
