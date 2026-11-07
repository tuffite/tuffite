//! Browser identity and counters, independent from application-wide state.
use super::PlaygroundBrowser;
use crate::shell_api;
use std::sync::atomic::Ordering;
use tuffite::{CommandError, ProfileId};

fn error(error: impl std::fmt::Display) -> CommandError {
    CommandError::Failed(error.to_string())
}

impl shell_api::Diagnostics for PlaygroundBrowser {
    fn process_id(&self) -> Result<u32, CommandError> {
        Ok(std::process::id())
    }
}

impl shell_api::BrowserDiagnostics for PlaygroundBrowser {
    fn identity(&self) -> Result<String, CommandError> {
        Ok(serde_json::json!({
            "profile": self.context.profile_id().map(ProfileId::as_str),
            "context": self.context.handle().map_err(error)?.as_u64().to_string(),
            "application": self.context.application()
                .and_then(|application| application.try_downcast::<crate::application::PlaygroundApplicationContext>())
                .map(|application| application.label.clone()),
        })
        .to_string())
    }
    fn increment(&self) -> Result<u32, CommandError> {
        Ok(self.profile_counter.fetch_add(1, Ordering::Relaxed) + 1)
    }
}
