//! Application-wide state, lifecycle and runtime declarations.
mod configuration;
mod profiles;

use crate::shell_api;
use std::sync::{Arc, atomic::AtomicU32};
use tuffite::{Application, ApplicationContext, Result};

pub(crate) struct PlaygroundApplication {
    context: ApplicationContext,
    calls: AtomicU32,
}
impl Application for PlaygroundApplication {
    fn setup(builder: tuffite::app::Builder<Self>) -> Result<tuffite::app::Builder<Self>> {
        Ok(builder.configuration(configuration::build()).interfaces(
            tuffite::app::InterfaceRegistry::register::<shell_api::interfaces::Profiles>,
        ))
    }
    fn new(context: ApplicationContext) -> Result<Self> {
        crate::logging::initialize();
        context.set_state(Arc::new(String::from("playground application")));
        Ok(Self {
            context,
            calls: AtomicU32::new(0),
        })
    }
    async fn initialize(&mut self) -> Result<()> {
        tracing::info!("playground application ready; profiles remain lazy");
        Ok(())
    }
    async fn shutdown(&mut self, _: tuffite::ShutdownContext) -> Result<()> {
        tracing::info!("playground application shutdown after profiles");
        Ok(())
    }
}
