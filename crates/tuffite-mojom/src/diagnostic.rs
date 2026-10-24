use miette::{Diagnostic, NamedSource, SourceSpan};
use std::path::Path;

/// A structured Mojom diagnostic. Render at the application boundary with miette;
/// library callers retain source text, labels and help for their own tooling.
#[derive(Debug, thiserror::Error, Diagnostic)]
#[error("{message}")]
#[diagnostic(code(tuffite::mojom))]
pub struct Error {
    message: String,
    declaration: Option<String>,
    #[source_code]
    src: Option<std::sync::Arc<NamedSource<String>>>,
    #[label("here")]
    span: Option<SourceSpan>,
    #[help]
    help: Option<String>,
}
impl Error {
    pub(crate) fn at(message: impl Into<String>, offset: usize, length: usize) -> Self {
        Self {
            span: Some((offset, length).into()),
            ..Self::from(message.into())
        }
    }
    pub(crate) fn with_source(mut self, path: &Path, source: &str) -> Self {
        self.src = Some(std::sync::Arc::new(NamedSource::new(
            path.display().to_string(),
            source.to_owned(),
        )));
        self
    }
    pub(crate) fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
    pub(crate) fn in_declaration(mut self, declaration: &str, span: (usize, usize)) -> Self {
        if self.declaration.is_none() {
            self.declaration = Some(declaration.to_owned());
            self.span = Some(span.into());
        }
        self
    }
    pub(crate) fn declaration(&self) -> Option<&str> {
        self.declaration.as_deref()
    }
    pub(crate) fn has_source(&self) -> bool {
        self.src.is_some()
    }
    /// A plain summary, useful for programmatic checks without rendering.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self {
            message,
            declaration: None,
            src: None,
            span: None,
            help: None,
        }
    }
}
impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::from(message.to_owned())
    }
}
