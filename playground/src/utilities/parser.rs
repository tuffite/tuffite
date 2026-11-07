//! Static and dynamically generated sources for a named V8 utility.
//!
//! Resource resolution runs on Browser workers when modules are requested. Native caches
//! each source per session and executes it in a sandboxed utility process.

use tuffite::{Resolution, Resource};

struct ParserContext {
    base: tuffite::V8Context,
    fixture: String,
}

pub struct Parser {
    context: tuffite::V8Context,
    shared: std::sync::Arc<ParserState>,
    fixture_id: String,
}

/// Installed by Browser and read directly by the Browser-side provider.
pub struct ParserState {
    pub fixture_id: String,
}

impl tuffite::utility::V8 for Parser {
    fn setup(
        builder: tuffite::utility::v8::Builder,
    ) -> tuffite::Result<tuffite::utility::v8::Builder> {
        Ok(builder
            .context(|base| {
                Ok(ParserContext {
                    base,
                    fixture: "parser-provider".into(),
                })
            })
            .name("parser")
            .entry("extension/entry.mjs")
            .configuration(br#"{"fixtureId":"io.tuffite.playground"}"#.to_vec()))
    }

    fn new(context: tuffite::V8Context) -> tuffite::Result<Self> {
        let business = context.downcast::<ParserContext>()?;
        assert_eq!(business.fixture, "parser-provider");
        assert_eq!(business.base.name(), context.name());
        let config: serde_json::Value = serde_json::from_slice(context.configuration())?;
        let shared = context
            .browser_context::<crate::browser::PlaygroundBrowserContext>()?
            .parser
            .clone();
        let fixture_id = config["fixtureId"]
            .as_str()
            .ok_or_else(|| tuffite::Error::new("Parser configuration is missing"))?
            .to_owned();
        if context.name() != "parser" || shared.fixture_id != fixture_id {
            return Err(tuffite::Error::new("invalid V8 provider context"));
        }
        Ok(Self {
            context,
            shared,
            fixture_id,
        })
    }

    async fn resolve(
        &self,
        url: &tuffite::Url,
        _context: tuffite::ResolveContext,
    ) -> tuffite::Result<Resolution> {
        let url = url.as_str();
        if self.context.is_stopping() {
            return Err(std::io::Error::from(std::io::ErrorKind::Interrupted).into());
        }
        let source = match url {
            "playground://parser.utility/sdk/generated.mjs" => {
                format!(
                    "export const fixtureId = {:?}; export const sharedState = {};",
                    self.fixture_id,
                    self.shared.fixture_id == self.fixture_id
                )
            }
            "https://fixtures.tuffite.invalid/sdk/remote.mjs" => {
                "export const remoteFixture = true;".to_owned()
            }
            "file:///tuffite-fixtures/sdk/local.mjs" => {
                "export const localFixture = true;".to_owned()
            }
            _ => return Ok(Resolution::Pass), // Entry/runtime/cycles use app.assets.
        };
        Ok(Resolution::Resource(
            Resource::bytes(source.into_bytes()).mime_type("text/javascript"),
        ))
    }
}
