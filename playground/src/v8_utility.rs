//! Static and dynamically generated sources for a named V8 utility.
//!
//! Resource resolution runs on Browser workers when modules are requested. Native caches
//! each source per session and executes it in a sandboxed utility process.

use tuffite::utility::v8::{Error, ModuleSource};

pub struct Parser {
    context: tuffite::V8Context,
    shared: std::sync::Arc<ParserState>,
    fixture_id: String,
}

/// Installed by Browser and read directly by the Browser-side provider.
pub struct ParserState {
    pub fixture_id: String,
}

#[tuffite::utility(name = "parser")]
impl tuffite::utility::V8 for Parser {
    fn new(context: tuffite::V8Context) -> tuffite::ApplicationResult<Self> {
        let config: serde_json::Value = serde_json::from_slice(context.configuration())
            .map_err(|error| tuffite::ApplicationError::new(error.to_string()))?;
        let shared = context
            .state::<ParserState>()
            .ok_or_else(|| tuffite::ApplicationError::new("Browser shared state is missing"))?;
        let fixture_id = config["fixtureId"]
            .as_str()
            .ok_or_else(|| tuffite::ApplicationError::new("Parser configuration is missing"))?
            .to_owned();
        if context.name() != "parser" || shared.fixture_id != fixture_id {
            return Err(tuffite::ApplicationError::new(
                "invalid V8 provider context",
            ));
        }
        Ok(Self {
            context,
            shared,
            fixture_id,
        })
    }

    fn entry(&self) -> &'static str {
        "extension/entry.mjs"
    }

    fn resolve(&self, url: &str) -> Result<Option<ModuleSource>, Error> {
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
            _ => return Ok(None), // Entry/runtime/cycles use app.assets.
        };
        Ok(Some(ModuleSource::new(url.to_owned(), source)))
    }
}
