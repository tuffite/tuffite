//! Playground service hosted in a Chromium utility process.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use tuffite::{
    ApplicationResult, CommandError, CommandResult, Utility, UtilityFuture, UtilityRequest,
    UtilitySession, UtilitySessionCommand, UtilitySessionEmitter, UtilitySessionFuture,
    UtilitySessionRequest,
};

const STATUS: u8 = 0;
const ECHO: u8 = 1;
const CRASH: u8 = 2;
const DELAY: u8 = 3;
const CANCELLABLE: u8 = 4;
const CONCURRENT: u8 = 5;
const RESPONSE_HEADER_BYTES: usize = 4 + 8 + 8 + 8;

static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(1);

/// Stateful service created inside a supervised Chromium utility process.
pub struct EchoUtility {
    context: tuffite::UtilityContext,
    process_id: u32,
    instance_id: u64,
    invocation_count: AtomicU64,
    concurrent_arrivals: AtomicU64,
    started_at: Instant,
}

struct EchoSession {
    emitter: UtilitySessionEmitter,
    command_count: u64,
}

impl UtilitySession for EchoSession {
    fn send(&mut self, command: UtilitySessionCommand) -> UtilitySessionFuture<'_> {
        Box::pin(async move {
            if command.is_cancelled() {
                return Err(CommandError::Failed("session command cancelled".to_owned()));
            }
            self.command_count += 1;
            self.emitter
                .emit(1, command.payload().to_vec())
                .map_err(|error| CommandError::Failed(error.to_string()))?;
            Ok(self.command_count.to_le_bytes().to_vec())
        })
    }

    fn close(&mut self, reason: u32) -> CommandResult {
        Ok(format!("commands={};reason={reason}", self.command_count).into_bytes())
    }
}

impl EchoUtility {
    fn snapshot(&self, message: &[u8]) -> Vec<u8> {
        let elapsed_ms = u64::try_from(self.started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
        let mut response = Vec::with_capacity(RESPONSE_HEADER_BYTES + message.len());
        response.extend_from_slice(&self.process_id.to_le_bytes());
        response.extend_from_slice(&self.instance_id.to_le_bytes());
        response.extend_from_slice(&self.invocation_count.load(Ordering::Relaxed).to_le_bytes());
        response.extend_from_slice(&elapsed_ms.to_le_bytes());
        response.extend_from_slice(message);
        response
    }
}

#[tuffite::utility(name = "echo")]
impl Utility for EchoUtility {
    fn new(context: tuffite::UtilityContext) -> ApplicationResult<Self> {
        let config: serde_json::Value = serde_json::from_slice(context.configuration())
            .map_err(|error| tuffite::ApplicationError::new(error.to_string()))?;
        if context.name() != "echo"
            || config["service"] != "echo"
            || config["fixtureId"] != "io.tuffite.playground"
        {
            return Err(tuffite::ApplicationError::new(
                "invalid Utility startup context",
            ));
        }
        crate::init_tracing();
        let service = Self {
            context,
            process_id: std::process::id(),
            instance_id: NEXT_INSTANCE_ID.fetch_add(1, Ordering::Relaxed),
            invocation_count: AtomicU64::new(0),
            concurrent_arrivals: AtomicU64::new(0),
            started_at: Instant::now(),
        };
        tracing::info!(
            pid = service.process_id,
            instance = service.instance_id,
            "utility started"
        );
        Ok(service)
    }

    fn invoke(&self, request: UtilityRequest) -> UtilityFuture<'_> {
        Box::pin(async move {
            if self.context.is_stopping() {
                return Err(CommandError::Failed("service is stopping".to_owned()));
            }
            let Some((&operation, payload)) = request.payload().split_first() else {
                return Err(CommandError::InvalidArgument(
                    "utility request is missing its operation byte".to_owned(),
                ));
            };
            self.invocation_count.fetch_add(1, Ordering::Relaxed);
            match operation {
                STATUS => Ok(self.snapshot(b"ready")),
                ECHO => Ok(self.snapshot(payload)),
                CRASH => {
                    // Only the sandboxed utility process exits. Tuffite fails this
                    // ambiguous request once and restarts lazily with backoff.
                    tracing::warn!(
                        pid = self.process_id,
                        instance = self.instance_id,
                        "intentional utility crash"
                    );
                    std::process::abort();
                }
                DELAY => {
                    std::thread::sleep(Duration::from_millis(150));
                    Ok(self.snapshot(b"async-complete"))
                }
                CONCURRENT => {
                    // Each pair must enter the same service before either can finish.
                    // A serialized host stalls the first call until its deadline.
                    let ticket = self.concurrent_arrivals.fetch_add(1, Ordering::AcqRel);
                    let required_arrivals = (ticket / 2 + 1) * 2;
                    while self.concurrent_arrivals.load(Ordering::Acquire) < required_arrivals {
                        if request.is_cancelled()
                            || self.context.is_stopping()
                            || Instant::now() >= request.deadline()
                        {
                            return Err(CommandError::Failed(
                                "concurrent peer did not arrive".to_owned(),
                            ));
                        }
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Ok(self.snapshot(b"concurrent-complete"))
                }
                CANCELLABLE => {
                    while !request.is_cancelled() && Instant::now() < request.deadline() {
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    if request.is_cancelled() {
                        Err(CommandError::Failed(
                            "utility invocation cancelled".to_owned(),
                        ))
                    } else {
                        Err(CommandError::Failed(
                            "utility invocation deadline reached".to_owned(),
                        ))
                    }
                }
                _ => Err(CommandError::InvalidArgument(format!(
                    "unknown utility operation: {operation}"
                ))),
            }
        })
    }

    fn create_session(
        &self,
        request: UtilitySessionRequest,
    ) -> ApplicationResult<Box<dyn UtilitySession>> {
        if request.init() != b"playground-session-v1" {
            return Err(tuffite::ApplicationError::new(
                "unexpected Utility session initialization",
            ));
        }
        Ok(Box::new(EchoSession {
            emitter: request.emitter(),
            command_count: 0,
        }))
    }

    fn shutdown(&self) -> ApplicationResult<()> {
        // A real service flushes indexes and durable checkpoints here.
        tracing::info!(
            pid = self.process_id,
            instance = self.instance_id,
            invocations = self.invocation_count.load(Ordering::Relaxed),
            "utility shutdown"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_is_binary_and_instance_state_is_reused() {
        let service = EchoUtility::new(tuffite::UtilityContext::new_for_test(
            "echo",
            br#"{"service":"echo","fixtureId":"io.tuffite.playground"}"#.to_vec(),
        ))
        .expect("service starts");
        let first = futures_executor::block_on(
            service.invoke(UtilityRequest::new_for_test(vec![ECHO, b'a'])),
        )
        .expect("first invocation");
        let second =
            futures_executor::block_on(service.invoke(UtilityRequest::new_for_test(vec![STATUS])))
                .expect("second invocation");
        assert_eq!(first.len(), RESPONSE_HEADER_BYTES + 1);
        assert_eq!(&first[RESPONSE_HEADER_BYTES..], b"a");
        assert_eq!(u64::from_le_bytes(first[12..20].try_into().unwrap()), 1);
        assert_eq!(u64::from_le_bytes(second[12..20].try_into().unwrap()), 2);
        assert_eq!(&first[..12], &second[..12]);
    }

    #[test]
    fn malformed_and_unknown_requests_are_rejected() {
        let service = EchoUtility::new(tuffite::UtilityContext::new_for_test(
            "echo",
            br#"{"service":"echo","fixtureId":"io.tuffite.playground"}"#.to_vec(),
        ))
        .expect("service starts");
        assert!(matches!(
            futures_executor::block_on(service.invoke(UtilityRequest::new_for_test(Vec::new()))),
            Err(CommandError::InvalidArgument(_))
        ));
        assert!(matches!(
            futures_executor::block_on(service.invoke(UtilityRequest::new_for_test(vec![99]))),
            Err(CommandError::InvalidArgument(_))
        ));
    }
}
