//! Application composition. Registries stay at the application root; services
//! receive their construction dependencies and manage lifecycle explicitly.
//!
//! ```no_run
//! # #[cfg(feature = "ioc-registry")]
//! # mod example {
//! use tuffite::{BrowserContext, ioc::{self, FactoryContext, Factory, Service}};
//! use std::{path::PathBuf, sync::Arc};
//! struct AppContext { browser: BrowserContext, data_dir: PathBuf }
//! impl AsRef<BrowserContext> for AppContext {
//!     fn as_ref(&self) -> &BrowserContext { &self.browser }
//! }
//! struct DataDirectoryFactory;
//! impl Factory<AppContext> for DataDirectoryFactory {
//!     type Target = Service<PathBuf>;
//!     fn build(&self, build: &mut FactoryContext<'_, AppContext>) -> ioc::Result<Arc<PathBuf>> {
//!         Ok(Arc::new(build.context().data_dir.clone()))
//!     }
//! }
//! fn compose(context: AppContext) -> ioc::Result<ioc::Registry<AppContext>> {
//!     let mut registry = ioc::browser_registry(context)?;
//!     registry.register(DataDirectoryFactory)?;
//!     registry.freeze();
//!     Ok(registry)
//! }
//! # }
//! ```
pub use tuffite_ioc::{Factory, FactoryContext, KeyedService, Registrar, Result, Service, factory};

#[cfg(feature = "ioc-registry")]
pub use tuffite_ioc::Registry;

/// Creates a Browser-local registry retaining the complete application context.
/// Native capabilities are obtained through `AsRef`; no services are started.
#[cfg(feature = "ioc-registry")]
pub fn browser_registry<C>(context: C) -> Result<Registry<C>>
where
    C: AsRef<crate::BrowserContext> + Send + Sync + 'static,
{
    use std::sync::Arc;
    let context = Arc::new(context);
    let browser = context.as_ref().as_ref();
    let mut registry = Registry::new(Arc::clone(&context));
    registry.inject(Arc::new(browser.events()))?;
    registry.inject(Arc::new(browser.executor()))?;
    registry.inject(Arc::new(browser.shell_api()))?;
    registry.inject(Arc::new(browser.utility_sessions()))?;
    registry.inject(Arc::new(browser.utilities()))?;
    Ok(registry)
}
