//! Application protocol resolution, independent of HTTP wire encoding.
use crate::{Error, Result, StopSignal};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
pub use url::Url;
pub(crate) const URL_COMMAND: u64 = u64::MAX;
pub(crate) const SHORTCUT_COMMAND: u64 = u64::MAX - 1;

/// Request metadata. These values describe the caller and do not grant access.
#[derive(Clone, Debug)]
pub struct ResolveContext {
    pub(crate) request: Request,
    pub(crate) stop: StopSignal,
    pub(crate) session: Option<u64>,
    pub(crate) deadline: std::time::Instant,
}
impl ResolveContext {
    #[must_use]
    pub fn session_id(&self) -> Option<u64> {
        self.session
    }
    #[must_use]
    pub fn deadline(&self) -> std::time::Instant {
        self.deadline
    }
    pub(crate) fn module(
        url: String,
        session: u64,
        deadline: std::time::Instant,
        stop: StopSignal,
    ) -> Self {
        Self {
            request: Request {
                url,
                method: "GET".into(),
                initiator: None,
                window: None,
                profile: None,
                navigation: false,
            },
            stop,
            session: Some(session),
            deadline,
        }
    }

    #[must_use]
    pub fn initiator(&self) -> Option<&str> {
        self.request.initiator.as_deref()
    }
    #[must_use]
    pub fn method(&self) -> &str {
        &self.request.method
    }
    #[must_use]
    pub fn window(&self) -> Option<crate::WindowId> {
        self.request
            .window
            .as_deref()?
            .parse()
            .ok()
            .filter(|id| *id > 0)
            .map(crate::WindowId)
    }
    #[must_use]
    pub fn profile(&self) -> Option<&str> {
        self.request.profile.as_deref()
    }
    #[must_use]
    pub fn is_navigation(&self) -> bool {
        self.request.navigation
    }
    #[must_use]
    pub fn stop_signal(&self) -> StopSignal {
        self.stop.clone()
    }
}
#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Request {
    pub url: String,
    pub method: String,
    pub initiator: Option<String>,
    pub window: Option<String>,
    pub profile: Option<String>,
    #[serde(default)]
    pub navigation: bool,
}
/// Native loading supplies CSP, disables caching, and bounds file reads.
#[derive(Clone, Debug)]
pub struct Resource {
    status: u16,
    mime_type: Option<String>,
    bytes: Option<Vec<u8>>,
    file: Option<PathBuf>,
}
impl Resource {
    pub(crate) fn module_bytes(self) -> Result<Vec<u8>> {
        if self.status != 200 {
            return Err(Error::new("module resource must have status 200"));
        }
        let bytes = if let Some(bytes) = self.bytes {
            bytes
        } else {
            use std::io::Read;
            let path = self
                .file
                .ok_or_else(|| Error::new("module resource has no body"))?;
            if !path.is_absolute() || !std::fs::metadata(&path)?.is_file() {
                return Err(Error::new(
                    "module resource must be an absolute regular file",
                ));
            }
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(512 * 1024 + 1)
                .read_to_end(&mut bytes)?;
            bytes
        };
        if bytes.is_empty() || bytes.len() > 512 * 1024 || std::str::from_utf8(&bytes).is_err() {
            return Err(Error::new(
                "resolver returned an invalid UTF-8 module source",
            ));
        }
        Ok(bytes)
    }

    #[must_use]
    pub fn bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            status: 200,
            mime_type: None,
            bytes: Some(bytes.into()),
            file: None,
        }
    }
    /// File paths are absolute and supplied only by trusted business code.
    #[must_use]
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self {
            status: 200,
            mime_type: None,
            bytes: None,
            file: Some(path.into()),
        }
    }
    #[must_use]
    pub fn mime_type(mut self, mime: impl Into<String>) -> Self {
        self.mime_type = Some(mime.into());
        self
    }
    #[must_use]
    pub const fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }
}
#[derive(Clone, Debug)]
pub enum Resolution {
    Resource(Resource),
    /// An action completed. Fetch gets 204; navigation is cancelled in place.
    Consumed,
    /// Cancel the request, including fetch.
    Discard,
    /// Continue with a less specific static route, or return 404.
    Pass,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WireResolution {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<PathBuf>,
}
impl Resolution {
    pub(crate) fn encode(self) -> Result<Vec<u8>> {
        let mut wire = WireResolution {
            kind: "pass",
            status: None,
            mime_type: None,
            body: None,
            file: None,
        };
        match self {
            Self::Resource(resource) => {
                if !(200..=599).contains(&resource.status)
                    || (300..400).contains(&resource.status)
                    || resource
                        .mime_type
                        .as_ref()
                        .is_some_and(|mime| mime.is_empty() || mime.contains(['\r', '\n', '\0']))
                    || resource
                        .bytes
                        .as_ref()
                        .is_some_and(|body| body.len() > 8 * 1024 * 1024)
                    || resource.file.as_ref().is_some_and(|path| {
                        !path.is_absolute()
                            || std::fs::metadata(path).is_ok_and(|metadata| !metadata.is_file())
                    })
                {
                    return Err(Error::new("invalid resolved resource"));
                }
                wire.kind = "resource";
                wire.status = Some(resource.status);
                wire.mime_type = resource.mime_type;
                wire.body = resource
                    .bytes
                    .map(|body| base64::engine::general_purpose::STANDARD.encode(body));
                wire.file = resource.file;
            }
            Self::Consumed => wire.kind = "consumed",
            Self::Discard => wire.kind = "discard",
            Self::Pass => {}
        }
        serde_json::to_vec(&wire).map_err(|e| Error::new(e.to_string()))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn outcomes_and_binary_resources_have_distinct_encodings() {
        let encoded =
            Resolution::Resource(Resource::bytes(vec![0, 255, 128]).mime_type("image/png"))
                .encode()
                .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(value["body"], "AP+A");
        for (result, kind) in [
            (Resolution::Consumed, "consumed"),
            (Resolution::Discard, "discard"),
            (Resolution::Pass, "pass"),
        ] {
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&result.encode().unwrap()).unwrap()["kind"],
                kind
            );
        }
        assert!(
            Resolution::Resource(Resource::bytes(vec![]).mime_type("x\r\nInjected: yes"))
                .encode()
                .is_err()
        );
        assert!(
            Resolution::Resource(Resource::file("relative.png"))
                .encode()
                .is_err()
        );
        assert!(
            Resolution::Resource(Resource::file(std::env::temp_dir()))
                .encode()
                .is_err()
        );
    }
}
