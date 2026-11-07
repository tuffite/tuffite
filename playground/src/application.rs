//! Application-wide state, lifecycle and runtime declarations.
mod configuration;
mod profiles;

use crate::shell_api;
use std::sync::atomic::AtomicU32;
use tuffite::{Application, ApplicationContext, Result};

pub(crate) struct PlaygroundApplicationContext {
    pub base: ApplicationContext,
    pub label: String,
}
pub(crate) struct PlaygroundApplication {
    context: ApplicationContext,
    calls: AtomicU32,
}
impl Application for PlaygroundApplication {
    fn setup(builder: tuffite::app::Builder<Self>) -> Result<tuffite::app::Builder<Self>> {
        Ok(builder
            .context(|base| {
                Ok(PlaygroundApplicationContext {
                    base,
                    label: "playground application".into(),
                })
            })
            .configuration(configuration::build())
            .interfaces(
                tuffite::app::InterfaceRegistry::register::<shell_api::interfaces::Profiles>,
            ))
    }
    fn new(context: ApplicationContext) -> Result<Self> {
        crate::logging::initialize();
        let business = context.downcast::<PlaygroundApplicationContext>()?;
        assert_eq!(business.label, "playground application");
        let _ = business.base.configuration();
        Ok(Self {
            context,
            calls: AtomicU32::new(0),
        })
    }
    async fn initialize(&mut self) -> Result<()> {
        self.context
            .shortcuts()
            .application()
            .register("CommandOrControl+Shift+P")
            .await?;
        let events = self.context.events();
        self.context
            .shortcuts()
            .application()
            .on_shortcut("CommandOrControl+Shift+K", move |_| {
                let _ = events.emit("playground.framework.shortcut", br#"{"callback":true}"#);
            })
            .await?;
        tracing::info!("playground application ready; profiles remain lazy");
        Ok(())
    }
    async fn shutdown(&mut self, _: tuffite::ShutdownContext) -> Result<()> {
        tracing::info!("playground application shutdown after profiles");
        Ok(())
    }

    async fn shortcut(&mut self, event: tuffite::ShortcutEvent) -> Result<()> {
        self.context.events().emit(
            "playground.framework.shortcut",
            format!("{:?}: {}", event.scope, event.shortcut).as_bytes(),
        )
    }
}
