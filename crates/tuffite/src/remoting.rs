//! Asynchronous native desktop remoting client. No application policy or transport.
use crate::shell_api::remoting;
pub use crate::shell_api::{RemotingInputEvent, RemotingSignal, RemotingStartOptions};
use crate::{ShellApiHost, ShellApiRequest};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

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
    shell_api: Option<ShellApiHost>,
    supported: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl Remoting {
    #[must_use]
    pub fn new(shell_api: Option<ShellApiHost>) -> Self {
        Self {
            shell_api,
            supported: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    pub fn is_supported(&self) -> bool {
        self.shell_api
            .as_ref()
            .is_some_and(ShellApiHost::is_available)
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
        let shell_api = self
            .shell_api
            .as_ref()
            .ok_or("native Shell API is unavailable")?;
        let cleanup_api = shell_api.clone();
        let (tx, rx) = oneshot::channel();
        shell_api
            .invoke(&remoting::Start { options }, move |result| {
                if let Err(Ok(session)) = tx.send(result) {
                    // A timed-out/cancelled creator must not strand a native session.
                    let _ = cleanup_api.invoke(&remoting::Stop { id: session.id }, |_| {});
                }
            })
            .map_err(|error| error.to_string())?;
        let session = tokio::time::timeout(std::time::Duration::from_secs(5), rx)
            .await
            .map_err(|_| "native remoting startup timed out")?
            .map_err(|_| "native remoting startup was cancelled")?
            .map_err(|error| error.to_string())?;
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
            data_base64: base64::engine::general_purpose::STANDARD.encode(snapshot.data.0),
        })
    }
    pub async fn inject_input(&self, event: RemotingInputEvent) -> Result<(), String> {
        self.invoke(remoting::InjectInput { event }).await
    }
    async fn invoke<R: ShellApiRequest>(&self, request: R) -> Result<R::Response, String> {
        let shell_api = self
            .shell_api
            .as_ref()
            .ok_or_else(|| "native Shell API is unavailable".to_owned())?;
        let (tx, rx) = oneshot::channel();
        shell_api
            .invoke(&request, move |result| {
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
