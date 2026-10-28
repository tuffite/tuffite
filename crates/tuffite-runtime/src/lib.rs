//! Tuffite's Rust control plane and its stable C ABI entry points.

pub mod abi;
pub mod application;
mod embedding;
mod error;
pub use builder::{ProcessKind, Startup};
pub use embedding::Embedding;
pub use error::{Error, Result};
pub mod browser;
mod builder;
#[doc(hidden)]
pub mod codec;
pub mod configuration;
mod contexts;
mod dispatch;
mod executor;
mod handle;
pub mod isolated;
mod javascript_extension;
mod lifecycle;
#[doc(hidden)]
pub mod native_remoting;
pub use lifecycle::{ShutdownContext, ShutdownReason, StopSignal};
pub mod mojom;
pub mod protocol;
mod runtime;
mod supervisor;
pub mod utility;
mod utility_entry;
mod utility_host;
mod utility_session;

pub use application::{Application, ApplicationContext, CommandResult};
pub use browser::{Browser, BrowserContext, Command, EventEmitter};
pub use builder::CommandDescriptor;
pub use contexts::{
    BrowserContextHandle, ProfileId, ProfileInfo, ProfileManager, ViewHandle, Views, WindowId,
    WindowOptions, Windows,
};
/// Application startup configuration and typed interface registration.
pub mod app {
    pub use crate::application::builder::{Builder, InterfaceFor, InterfaceRegistry};
    pub use crate::builder::ProcessKind;
    pub use crate::configuration::Configuration;
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
pub use utility::{Utility, UtilityContext, UtilityRequest, UtilityServiceRegistration, V8Context};
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
