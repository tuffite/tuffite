//! Profiles API: compose profile views and application-owned windows.
use super::PlaygroundApplication;
use crate::shell_api;
use std::sync::atomic::Ordering;
use tuffite::{BrowserContextHandle, CommandError, ProfileId, ViewHandle, WindowId};

fn error(error: impl std::fmt::Display) -> CommandError {
    CommandError::Failed(error.to_string())
}
fn number(value: &str) -> Result<u64, CommandError> {
    value.parse().map_err(error)
}
impl shell_api::Profiles for PlaygroundApplication {
    fn list(&self) -> Result<String, CommandError> {
        let profiles = futures_executor::block_on(self.context.profiles().list()).map_err(error)?;
        serde_json::to_string(&profiles).map_err(error)
    }
    fn create(&self, id: String, name: String) -> Result<(), CommandError> {
        futures_executor::block_on(
            self.context
                .profiles()
                .create(ProfileId::new(id).map_err(error)?, name),
        )
        .map_err(error)
    }
    fn prepare(&self, id: String, url: String) -> Result<String, CommandError> {
        futures_executor::block_on(async {
            let context = self.context.profiles().open(&ProfileId::new(id)?).await?;
            let view = context.views().create(url).await?;
            Ok::<_, tuffite::Error>(view.as_u64().to_string())
        })
        .map_err(error)
    }
    fn attach(&self, window_id: i64, view: String) -> Result<String, CommandError> {
        let view = ViewHandle::from_u64(number(&view)?).map_err(error)?;
        futures_executor::block_on(
            self.context
                .windows()
                .set_content(WindowId(window_id), view),
        )
        .map(|v| v.map_or_else(String::new, |view| view.as_u64().to_string()))
        .map_err(error)
    }
    fn release_view(&self, view: String) -> Result<(), CommandError> {
        futures_executor::block_on(
            self.context
                .windows()
                .close_view(ViewHandle::from_u64(number(&view)?).map_err(error)?),
        )
        .map_err(error)
    }
    fn close(&self, context: String) -> Result<(), CommandError> {
        futures_executor::block_on(
            self.context
                .profiles()
                .close(BrowserContextHandle::from_u64(number(&context)?).map_err(error)?),
        )
        .map_err(error)
    }
    fn application_state(&self) -> Result<u32, CommandError> {
        Ok(self.calls.fetch_add(1, Ordering::Relaxed) + 1)
    }
    fn create_window(&self) -> Result<i64, CommandError> {
        futures_executor::block_on(
            self.context
                .windows()
                .create(tuffite::WindowOptions::default()),
        )
        .map(|id| id.0)
        .map_err(error)
    }
    fn close_window(&self, window_id: i64) -> Result<(), CommandError> {
        futures_executor::block_on(self.context.windows().close(WindowId(window_id))).map_err(error)
    }
}
