//! State and lifecycle belonging to one loaded Playground profile.
mod commands;
mod diagnostics;
mod isolated_ui;
mod javascript_sessions;
mod utility_sessions;

use crate::{shell_api, utilities};
use std::sync::{Arc, Mutex};
use tuffite::{
    Browser, BrowserContext, ChromiumExecutor, EventEmitter, JavaScriptExtensionSessionHandle,
    Result, UtilitySessionHandle, UtilitySessionHost,
};

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
        Ok(builder.interfaces(|registry| {
            registry
                .register::<shell_api::interfaces::Commands>()
                .register::<shell_api::interfaces::Diagnostics>()
                .register::<shell_api::interfaces::BrowserDiagnostics>()
        }))
    }

    fn new(context: BrowserContext) -> Result<Self> {
        context.set_state(std::sync::Arc::new(utilities::ParserState {
            fixture_id: "io.tuffite.playground".to_owned(),
        }));
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
}
