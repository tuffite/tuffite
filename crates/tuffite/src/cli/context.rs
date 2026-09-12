//! Invocation-scoped services shared by CLI workflows.
use crate::build::framework::{Distribution, Resolver};

pub(super) struct Context {
    refresh_framework: bool,
    framework: Option<Distribution>,
}

impl Context {
    pub(super) const fn new(refresh_framework: bool) -> Self {
        Self {
            refresh_framework,
            framework: None,
        }
    }

    /// Resolve once, retaining the exact selection for source-triggered rebuilds.
    /// An unsuccessful resolution remains retryable.
    pub(super) fn framework(&mut self) -> Result<&Distribution, String> {
        if self.framework.is_none() {
            let mut resolver = Resolver::from_env()?;
            if self.refresh_framework {
                tracing::info!("forcing a fresh Tuffite Framework Release download");
                resolver = resolver.managed().auto_download(true).force_download(true);
            }
            self.framework = Some(resolver.resolve()?);
        }
        Ok(self.framework.as_ref().expect("resolved Framework"))
    }
}
