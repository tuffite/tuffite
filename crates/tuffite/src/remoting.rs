//! Asynchronous native desktop remoting client. No application policy or transport.
use crate::__private::mojom::Struct;
use crate::ApplicationContext;
use crate::shell_api::remoting;
pub use crate::shell_api::{RemotingInputEvent, RemotingSignal, RemotingStartOptions};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tuffite_runtime::native_remoting::Host;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotingSession {
    pub id: u64,
    pub state: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotingSnapshot {
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub data_base64: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemotingNetworkStats {
    pub state: String,
    pub route: String,
    pub transport: String,
    pub latency_ms: f64,
    pub jitter_ms: f64,
    pub bandwidth_kbps: f64,
    pub packet_loss_percent: f64,
}

#[derive(Clone)]
pub struct Remoting {
    host: Host,
    supported: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl Remoting {
    #[must_use]
    pub fn new(context: &ApplicationContext) -> Self {
        Self {
            host: context.remoting_host(),
            supported: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    #[must_use]
    pub fn is_supported(&self) -> bool {
        self.host.is_available()
    }
    #[must_use]
    pub fn supports_control(&self) -> bool {
        self.supported.load(std::sync::atomic::Ordering::Acquire)
    }
    pub async fn refresh_capabilities(&self) {
        let supported = match self.invoke(remoting::Capabilities {}).await {
            Ok(capabilities) => {
                tracing::debug!(available = capabilities.available, protocol = %capabilities.protocol, "native desktop capabilities");
                if !capabilities.available || capabilities.protocol != "crd-webrtc" {
                    tracing::warn!(available = capabilities.available, protocol = %capabilities.protocol, "native desktop protocol is unavailable");
                }
                capabilities.available && capabilities.protocol == "crd-webrtc"
            }
            Err(error) => {
                tracing::warn!(%error, "native desktop capability discovery failed");
                false
            }
        };
        self.supported
            .store(supported, std::sync::atomic::Ordering::Release);
    }
    pub async fn start(&self, options: RemotingStartOptions) -> Result<RemotingSession, String> {
        let host = &self.host;
        let cleanup_api = host.clone();
        let (tx, rx) = oneshot::channel();
        remoting::Start { options }.submit(host, move |result| {
            if let Err(Ok(session)) = tx.send(result) {
                // A timed-out/cancelled creator must not strand a native session.
                let _ = remoting::Stop { id: session.id }.submit(&cleanup_api, |_| {});
            }
        })?;
        let session = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .map_err(|_| "native remoting startup timed out")?
            .map_err(|_| "native remoting startup was cancelled")??;
        let id = u64::try_from(session.id)
            .ok()
            .filter(|id| *id > 0)
            .ok_or_else(|| "native CRD returned an invalid session ID".to_owned())?;
        Ok(RemotingSession {
            id,
            state: session.state,
        })
    }
    pub async fn process_signal(&self, id: u64, signal: RemotingSignal) -> Result<(), String> {
        self.invoke(remoting::ProcessSignal {
            id: session_id(id)?,
            signal,
        })
        .await
    }
    pub async fn take_signals(
        &self,
        id: u64,
        after_sequence: u64,
    ) -> Result<Vec<RemotingSignal>, String> {
        self.invoke(remoting::TakeSignals {
            id: session_id(id)?,
            after_sequence: Some(
                i64::try_from(after_sequence).map_err(|_| "invalid signal sequence")?,
            ),
        })
        .await
    }
    pub async fn input(&self, id: u64, event: RemotingInputEvent) -> Result<(), String> {
        self.invoke(remoting::SendInput {
            id: session_id(id)?,
            event,
        })
        .await
    }
    pub async fn stop(&self, id: u64) -> Result<(), String> {
        self.invoke(remoting::Stop {
            id: session_id(id)?,
        })
        .await
    }
    pub async fn stats(&self, id: u64) -> Result<RemotingNetworkStats, String> {
        let stats = self
            .invoke(remoting::Stats {
                id: session_id(id)?,
            })
            .await?;
        Ok(RemotingNetworkStats {
            state: stats.state,
            route: stats.route,
            transport: stats.transport,
            latency_ms: stats.latency_ms,
            jitter_ms: stats.jitter_ms,
            bandwidth_kbps: stats.available_bandwidth_kbps,
            packet_loss_percent: stats.packet_loss_percent,
        })
    }
    pub async fn snapshot(&self, screen_id: Option<i64>) -> Result<RemotingSnapshot, String> {
        let snapshot = self.invoke(remoting::Capture { screen_id }).await?;
        Ok(RemotingSnapshot {
            width: dimension(snapshot.width)?,
            height: dimension(snapshot.height)?,
            mime_type: snapshot.mime_type,
            data_base64: base64::engine::general_purpose::STANDARD.encode(snapshot.data),
        })
    }
    pub async fn inject_input(&self, event: RemotingInputEvent) -> Result<(), String> {
        self.invoke(remoting::InjectInput { event }).await
    }
    async fn invoke<R: Request>(&self, request: R) -> Result<R::Response, String> {
        let (tx, rx) = oneshot::channel();
        request
            .submit(&self.host, move |result| {
                let _ = tx.send(result);
            })
            .map_err(|error| format!("native CRD operation failed: {error}"))?;
        tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .map_err(|_| "native remoting deadline exceeded".to_owned())?
            .map_err(|_| "native CRD reply channel closed".to_owned())?
            .map_err(|error| format!("native CRD operation failed: {error}"))
    }
}

fn session_id(id: u64) -> Result<i64, String> {
    i64::try_from(id)
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| "invalid session ID".to_owned())
}
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn dimension(value: f64) -> Result<u32, String> {
    if !value.is_finite() || value.fract() != 0.0 || value <= 0.0 || value > f64::from(u32::MAX) {
        return Err("native CRD returned an invalid snapshot dimension".to_owned());
    }
    Ok(value as u32)
}

// The request-to-operation mapping is private to the dedicated Remoting client.
// There is no public channel-name invocation API.
trait Request: Struct {
    type Response: Send + 'static;
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static;
}
impl Request for remoting::Capabilities {
    type Response = crate::shell_api::RemotingCapabilities;
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.capabilities(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingCapabilitiesResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(response.result)
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::Start {
    type Response = crate::shell_api::RemotingSession;
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.start(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingStartResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(response.result)
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::ProcessSignal {
    type Response = ();
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.process_signal(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response =
                    crate::shell_api::RemotingProcessSignalResponse::decode_mojom(&bytes)
                        .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(())
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::TakeSignals {
    type Response = Vec<RemotingSignal>;
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.take_signals(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingTakeSignalsResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(response.result)
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::SendInput {
    type Response = ();
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.send_input(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingSendInputResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(())
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::InjectInput {
    type Response = ();
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.inject_input(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingInjectInputResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(())
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::Stop {
    type Response = ();
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.stop(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingStopResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(())
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::Stats {
    type Response = crate::shell_api::RemotingStats;
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.stats(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingStatsResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(response.result)
            }));
        })
        .map_err(|e| e.to_string())
    }
}
impl Request for remoting::Capture {
    type Response = crate::shell_api::RemotingSnapshot;
    fn submit<F>(&self, host: &Host, reply: F) -> Result<(), String>
    where
        F: FnOnce(Result<Self::Response, String>) + Send + 'static,
    {
        let bytes = self
            .encode_mojom()
            .map_err(|_| "invalid remoting request")?;
        host.capture(&bytes, move |result| {
            reply(result.map_err(|e| e.to_string()).and_then(|bytes| {
                let response = crate::shell_api::RemotingCaptureResponse::decode_mojom(&bytes)
                    .map_err(|_| "invalid remoting response".to_owned())?;
                let _ = &response;
                Ok(response.result)
            }));
        })
        .map_err(|e| e.to_string())
    }
}
