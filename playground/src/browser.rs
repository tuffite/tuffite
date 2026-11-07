//! State and lifecycle belonging to one loaded Playground profile.
mod commands;
mod diagnostics;
mod extensions;
mod isolated_ui;
mod javascript_sessions;
mod utility_sessions;

use crate::{shell_api, utilities};
use std::sync::{Arc, Mutex};
use tuffite::{
    Browser, BrowserContext, ChromiumExecutor, EventEmitter, JavaScriptExtensionSessionHandle,
    Result, UtilitySessionHandle, UtilitySessionHost,
};

pub(crate) struct PlaygroundBrowserContext {
    pub base: BrowserContext,
    pub parser: Arc<utilities::ParserState>,
    pub resource_file: ResolverFile,
}
/// A private sample resource whose lifetime follows its business Context.
pub(crate) struct ResolverFile {
    pub path: std::path::PathBuf,
}
impl ResolverFile {
    fn new() -> Result<Self> {
        use std::io::Write;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "tuffite-resolver-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| tuffite::Error::new(e.to_string()))?
                .as_nanos(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        let builder = {
            use std::os::unix::fs::DirBuilderExt;
            let mut builder = builder;
            builder.mode(0o700);
            builder
        };
        builder.create(&directory)?;
        let resource = Self {
            path: directory.join("example.txt"),
        };
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&resource.path)?
            .write_all(b"Resolved from a business-owned file")?;
        Ok(resource)
    }
}
impl Drop for ResolverFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        if let Some(directory) = self.path.parent() {
            let _ = std::fs::remove_dir(directory);
        }
    }
}

pub(crate) struct PlaygroundBrowser {
    context: BrowserContext,
    profile_counter: std::sync::atomic::AtomicU32,
    isolated: tuffite::isolated::Host,
    events: EventEmitter,
    executor: ChromiumExecutor,
    utility_sessions: UtilitySessionHost,
    utility_session: Arc<Mutex<Option<UtilitySessionHandle>>>,
    utilities: tuffite::utility::Host,
    javascript_extension_session: Arc<Mutex<Option<JavaScriptExtensionSessionHandle>>>,
}

impl Browser for PlaygroundBrowser {
    fn setup(builder: tuffite::browser::Builder<Self>) -> Result<tuffite::browser::Builder<Self>> {
        Ok(builder
            .context(|base| {
                Ok(PlaygroundBrowserContext {
                    base,
                    resource_file: ResolverFile::new()?,
                    parser: Arc::new(utilities::ParserState {
                        fixture_id: "io.tuffite.playground".into(),
                    }),
                })
            })
            .interfaces(|registry| {
                registry
                    .register::<shell_api::interfaces::Commands>()
                    .register::<shell_api::interfaces::Diagnostics>()
                    .register::<shell_api::interfaces::BrowserDiagnostics>()
            }))
    }

    fn new(context: BrowserContext) -> Result<Self> {
        let business = context.downcast::<PlaygroundBrowserContext>()?;
        assert_eq!(business.base.profile_id(), context.profile_id());
        crate::logging::initialize();
        Ok(Self {
            context: context.clone(),
            profile_counter: std::sync::atomic::AtomicU32::new(0),
            isolated: context.isolated(),
            events: context.events(),
            executor: context.executor(),
            utility_sessions: context.utility_sessions(),
            utility_session: Arc::new(Mutex::new(None)),
            utilities: context.utilities(),
            javascript_extension_session: Arc::new(Mutex::new(None)),
        })
    }

    async fn initialize(&mut self) -> Result<()> {
        isolated_ui::register(&self.isolated).await?;
        tracing::info!("playground browser services ready");
        Ok(())
    }

    async fn shutdown(&mut self, _context: tuffite::ShutdownContext) -> Result<()> {
        tracing::info!("playground browser shutdown");
        Ok(())
    }

    async fn resolve(
        &mut self,
        url: &tuffite::Url,
        context: tuffite::ResolveContext,
    ) -> Result<tuffite::Resolution> {
        self.resolve_framework(url, context).await
    }
}
