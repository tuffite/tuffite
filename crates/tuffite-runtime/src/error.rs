//! Common lifecycle and application composition errors.
use std::{fmt, sync::Arc};

/// An application failure with an optional typed cause.
#[derive(Clone, Debug)]
pub struct Error {
    message: String,
    source: Option<Arc<dyn std::error::Error + Send + Sync>>,
}
impl Error {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }
    /// Preserves a domain error for diagnostics and downcasting through `source()`.
    pub fn from_source(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            message: source.to_string(),
            source: Some(Arc::new(source)),
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|source| source.as_ref() as _)
    }
}
/// General application result. The error parameter can be specialized for domain operations.
pub type Result<T = (), E = Error> = std::result::Result<T, E>;

macro_rules! from_errors {
    ($($error:ty),* $(,)?) => {$(impl From<$error> for Error {
        fn from(error: $error) -> Self { Self::from_source(error) }
    })*};
}
from_errors!(
    std::io::Error,
    serde_json::Error,
    url::ParseError,
    crate::CommandError,
    crate::executor::ExecutorError,
    crate::isolated::Error,
    crate::utility::v8::Error,
    crate::utility_host::UtilitySessionHostError,
    crate::javascript_extension::JavaScriptExtensionError,
    crate::utility_session::UtilitySessionFlowError,
    crate::runtime::RuntimeError,
    crate::handle::HandleError,
    crate::codec::CodecError
);
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}
impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::new(message)
    }
}
impl From<Error> for crate::CommandError {
    fn from(error: Error) -> Self {
        error
            .source
            .as_ref()
            .and_then(|source| source.downcast_ref::<crate::CommandError>())
            .cloned()
            .unwrap_or_else(|| Self::Failed(error.to_string()))
    }
}

#[cfg(feature = "async")]
from_errors!(crate::executor::ChromiumTaskError);

impl From<Box<dyn std::error::Error + Send + Sync>> for Error {
    fn from(source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self {
            message: source.to_string(),
            source: Some(Arc::from(source)),
        }
    }
}
macro_rules! command_errors {
    ($($error:ty),* $(,)?) => {$(impl From<$error> for crate::CommandError {
        fn from(error: $error) -> Self { Self::Failed(error.to_string()) }
    })*};
}
command_errors!(std::io::Error, serde_json::Error, url::ParseError);

#[cfg(test)]
mod tests {
    #[test]
    fn command_conversion_preserves_protocol_variants() {
        let command = crate::CommandError::InvalidArgument("field".into());
        let error = super::Error::from(command.clone());
        assert_eq!(crate::CommandError::from(error), command);
    }
    #[test]
    fn question_mark_preserves_the_domain_error() {
        fn operation() -> super::Result {
            Err(crate::isolated::Error::Unavailable)?
        }
        let error = operation().unwrap_err();
        assert_eq!(
            std::error::Error::source(&error)
                .unwrap()
                .downcast_ref::<crate::isolated::Error>(),
            Some(&crate::isolated::Error::Unavailable)
        );
    }
}
