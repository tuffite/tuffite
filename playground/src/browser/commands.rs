//! Browser command handlers and the Chromium task-pool example.
use super::{PlaygroundBrowser, javascript_sessions, utility_sessions};
use crate::shell_api;
use tuffite::{CommandError, TaskPriority};

impl shell_api::Commands for PlaygroundBrowser {
    fn do_something(&self, message: String) -> Result<String, CommandError> {
        self.events
            .emit("playground:command", message.as_bytes())
            .map_err(|error| CommandError::Failed(error.to_string()))?;
        let events = self.events.clone();
        self.executor
            .post(TaskPriority::UserVisible, false, move || {
                let _ = events.emit("playground:thread-pool", b"chromium-thread-pool");
            })
            .map_err(|error| CommandError::Failed(error.to_string()))?;
        Ok(format!("Rust received: {message}"))
    }
    fn check_utility_session(&self) -> Result<String, CommandError> {
        utility_sessions::check(self)
    }
    fn check_javascript_extension(&self) -> Result<String, CommandError> {
        javascript_sessions::check(self)
    }
    fn echo_mojom(
        &self,
        value: shell_api::MojomSample,
    ) -> Result<shell_api::MojomSample, CommandError> {
        Ok(value)
    }
}
