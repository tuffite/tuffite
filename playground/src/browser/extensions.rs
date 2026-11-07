//! Dynamic resources, consumed actions, cancellation and scoped shortcut examples.
use super::PlaygroundBrowser;
use tuffite::{Resolution, ResolveContext, Resource, Result, Url};

impl PlaygroundBrowser {
    pub(super) async fn resolve_framework(
        &mut self,
        url: &Url,
        context: ResolveContext,
    ) -> Result<Resolution> {
        let application = self
            .context
            .application()
            .ok_or_else(|| tuffite::Error::new("application required"))?;
        if !matches!(url.host_str(), Some("main" | "ui")) {
            return Ok(Resolution::Pass);
        }
        if url.path().starts_with("/framework/shortcuts/")
            && context.initiator() != Some("playground://main")
        {
            return Ok(Resolution::Discard);
        }
        match url.path() {
            "/framework/resource.json" => Ok(Resolution::Resource(Resource::bytes(
                serde_json::to_vec(&serde_json::json!({"context":application.downcast::<crate::application::PlaygroundApplicationContext>()?.label,"profile":context.profile(),"window":context.window().map(|w|w.0),"initiator":context.initiator()}))?
            ).mime_type("application/json"))),
            "/framework/image.svg" => Ok(Resolution::Resource(Resource::bytes(br##"<svg xmlns="http://www.w3.org/2000/svg" width="96" height="64"><rect width="96" height="64" rx="12" fill="#5267df"/><text x="48" y="39" text-anchor="middle" fill="white" font-size="20">Tuffite</text></svg>"##.to_vec()).mime_type("image/svg+xml"))),
            "/framework/file" => Ok(Resolution::Resource(Resource::file(
                self.context.downcast::<super::PlaygroundBrowserContext>()?.resource_file.path.clone()
            ).mime_type("text/plain"))),
            "/dialog/open" | "/framework/consume" => {
                if context.initiator() != Some("playground://main") { return Ok(Resolution::Discard); }
                self.context.events().emit("playground.framework.action", url.as_str().as_bytes())?;
                Ok(Resolution::Consumed)
            }
            "/framework/discard" => Ok(Resolution::Discard),
            "/framework/shortcuts/window" => {
                let window = context.window().ok_or_else(|| tuffite::Error::new("window required"))?;
                application.shortcuts().window(window).register("CommandOrControl+Shift+P").await?;
                Ok(Resolution::Consumed)
            }
            "/framework/shortcuts/system" => {
                match application.shortcuts().system().register("CommandOrControl+Alt+Space").await {
                    Ok(()) => Ok(Resolution::Consumed),
                    Err(error) => Ok(Resolution::Resource(Resource::bytes(error.to_string().into_bytes()).mime_type("text/plain").status(409))),
                }
            }
            _ => Ok(Resolution::Pass),
        }
    }
}
