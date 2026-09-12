//! Flow-control primitives and safe service API for long-lived Utility sessions.
//!
//! Transport adapters use this state machine on the producer side. It keeps
//! semantic session events ordered and bounded; renderer application events are
//! a separate best-effort projection and must not bypass these credits.

/// Default maximum number of unacknowledged events in one Utility session.
pub const DEFAULT_UTILITY_SESSION_CREDITS: u32 = 32;
/// Hard upper bound for a single semantic Utility session event.
pub const MAX_UTILITY_SESSION_EVENT_BYTES: usize = 256 * 1024;
/// Hard upper bound for a negotiated event-credit window.
pub const MAX_UTILITY_SESSION_CREDITS: u32 = 1_024;

use std::sync::{Arc, Mutex};

use crate::application::{ApplicationError, ApplicationResult, CommandResult};
use crate::supervisor::CancellationToken;

/// One ordered semantic event emitted by a long-lived Utility session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UtilitySessionEvent {
    sequence: u64,
    kind: u32,
    payload: Vec<u8>,
}

impl UtilitySessionEvent {
    pub(crate) const fn new(sequence: u64, kind: u32, payload: Vec<u8>) -> Self {
        Self {
            sequence,
            kind,
            payload,
        }
    }

    /// Returns the monotonically increasing per-session sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the application-defined, versioned event kind.
    #[must_use]
    pub const fn kind(&self) -> u32 {
        self.kind
    }

    /// Returns the bounded event payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// Protocol errors detected before an event reaches Mojo or application code.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum UtilitySessionFlowError {
    /// The negotiated credit window or payload limit is invalid.
    #[error("invalid Utility session limits")]
    InvalidLimits,
    /// Event kind zero is reserved for transport lifecycle messages.
    #[error("Utility session event kind is invalid")]
    InvalidEventKind,
    /// An event payload exceeded the negotiated or framework limit.
    #[error("Utility session event payload is too large")]
    EventTooLarge,
    /// The producer exhausted its event credits and must await an ACK.
    #[error("Utility session event credits are exhausted")]
    CreditsExhausted,
    /// An ACK moved backwards, exceeded the emitted sequence, or over-granted.
    #[error("Utility session acknowledgement is invalid")]
    InvalidAcknowledgement,
    /// The sequence space was exhausted.
    #[error("Utility session sequence is exhausted")]
    SequenceExhausted,
    /// No more events are accepted after close.
    #[error("Utility session is closed")]
    Closed,
}

/// Producer-side credit and sequence state for one Utility session.
#[derive(Debug)]
pub struct UtilitySessionFlowControl {
    available_credits: u32,
    max_credits: u32,
    max_event_bytes: usize,
    last_emitted_sequence: u64,
    committed_sequence: u64,
    closed: bool,
}

type UtilitySessionEventSink =
    dyn Fn(UtilitySessionEvent) -> ApplicationResult<()> + Send + Sync + 'static;

/// Cloneable, credit-controlled event emitter supplied to a Utility session.
#[derive(Clone)]
pub struct UtilitySessionEmitter {
    flow: Arc<Mutex<UtilitySessionFlowControl>>,
    sink: Arc<UtilitySessionEventSink>,
}

impl std::fmt::Debug for UtilitySessionEmitter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UtilitySessionEmitter")
            .finish_non_exhaustive()
    }
}

impl UtilitySessionEmitter {
    /// Creates a Host-wired emitter after its limits and event sink are valid.
    #[allow(dead_code)]
    pub(crate) fn new_for_host(
        initial_credits: u32,
        max_event_bytes: usize,
        sink: impl Fn(UtilitySessionEvent) -> ApplicationResult<()> + Send + Sync + 'static,
    ) -> Result<Self, UtilitySessionFlowError> {
        Ok(Self {
            flow: Arc::new(Mutex::new(UtilitySessionFlowControl::new(
                initial_credits,
                max_event_bytes,
            )?)),
            sink: Arc::new(sink),
        })
    }

    /// Emits one ordered semantic event if the consumer has granted credit.
    pub fn emit(&self, kind: u32, payload: impl Into<Vec<u8>>) -> ApplicationResult<u64> {
        // Hold the flow lock until the Host sink has copied/enqueued the event.
        // This preserves sequence order and guarantees close waits for any
        // in-flight callback. Host sinks must not synchronously re-enter this
        // emitter; acknowledgement arrives on the later Host delivery path.
        let mut flow = self
            .flow
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let event = flow
            .emit(kind, payload)
            .map_err(|error| ApplicationError::new(error.to_string()))?;
        let sequence = event.sequence();
        if let Err(error) = (self.sink)(event) {
            flow.close();
            return Err(error);
        }
        Ok(sequence)
    }

    /// Applies a Host acknowledgement after its durable commit boundary.
    #[allow(dead_code)]
    pub(crate) fn acknowledge_from_host(
        &self,
        committed_sequence: u64,
        credits: u32,
    ) -> Result<(), UtilitySessionFlowError> {
        self.flow
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .acknowledge(committed_sequence, credits)
    }

    /// Prevents further emission and returns the final emitted sequence.
    #[allow(dead_code)]
    pub(crate) fn close_from_host(&self) -> u64 {
        self.flow
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .close()
    }

    /// Returns currently available producer credits.
    #[must_use]
    pub fn available_credits(&self) -> u32 {
        self.flow
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .available_credits()
    }
}

/// Immutable initialization data supplied when creating one Utility session.
#[derive(Clone, Debug)]
pub struct UtilitySessionRequest {
    id: u64,
    init: Vec<u8>,
    emitter: UtilitySessionEmitter,
}

impl UtilitySessionRequest {
    /// Creates a request after the Host validates session limits and identity.
    #[allow(dead_code)]
    pub(crate) const fn new_for_host(
        id: u64,
        init: Vec<u8>,
        emitter: UtilitySessionEmitter,
    ) -> Self {
        Self { id, init, emitter }
    }

    /// Returns the caller-assigned logical session ID.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the versioned initialization payload.
    #[must_use]
    pub fn init(&self) -> &[u8] {
        &self.init
    }

    /// Returns a cloneable emitter for ordered semantic events.
    #[must_use]
    pub fn emitter(&self) -> UtilitySessionEmitter {
        self.emitter.clone()
    }
}

/// One command delivered to a live Utility session.
#[derive(Clone)]
pub struct UtilitySessionCommand {
    id: u64,
    payload: Vec<u8>,
    cancellation: CancellationToken,
}

impl std::fmt::Debug for UtilitySessionCommand {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UtilitySessionCommand")
            .field("id", &self.id)
            .field("payload_bytes", &self.payload.len())
            .field("cancelled", &self.cancellation.is_cancelled())
            .finish()
    }
}

impl UtilitySessionCommand {
    /// Creates a command after the Host validates its envelope and ownership.
    #[allow(dead_code)]
    pub(crate) const fn new_for_host(
        id: u64,
        payload: Vec<u8>,
        cancellation: CancellationToken,
    ) -> Self {
        Self {
            id,
            payload,
            cancellation,
        }
    }

    /// Returns the command identity, unique within the session.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the versioned command payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    /// Reports whether the trusted Host cancelled this command.
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancellation.is_cancelled()
    }
}

/// Future returned by one Utility session command.
pub type UtilitySessionFuture<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = CommandResult> + Send + 'a>>;

/// Stateful implementation hosted for one long-lived Utility session.
pub trait UtilitySession: Send + 'static {
    /// Handles one bounded command. Semantic output is emitted separately.
    fn send(&mut self, command: UtilitySessionCommand) -> UtilitySessionFuture<'_>;

    /// Performs graceful session-specific cleanup and returns a bounded summary.
    fn close(&mut self, _reason: u32) -> CommandResult {
        Ok(Vec::new())
    }
}

impl UtilitySessionFlowControl {
    /// Creates a session with an explicit credit window and event-size limit.
    pub const fn new(
        initial_credits: u32,
        max_event_bytes: usize,
    ) -> Result<Self, UtilitySessionFlowError> {
        if initial_credits == 0
            || initial_credits > MAX_UTILITY_SESSION_CREDITS
            || max_event_bytes == 0
            || max_event_bytes > MAX_UTILITY_SESSION_EVENT_BYTES
        {
            return Err(UtilitySessionFlowError::InvalidLimits);
        }
        Ok(Self {
            available_credits: initial_credits,
            max_credits: initial_credits,
            max_event_bytes,
            last_emitted_sequence: 0,
            committed_sequence: 0,
            closed: false,
        })
    }

    /// Reserves one credit and creates the next ordered event.
    pub fn emit(
        &mut self,
        kind: u32,
        payload: impl Into<Vec<u8>>,
    ) -> Result<UtilitySessionEvent, UtilitySessionFlowError> {
        if self.closed {
            return Err(UtilitySessionFlowError::Closed);
        }
        if kind == 0 {
            return Err(UtilitySessionFlowError::InvalidEventKind);
        }
        let payload = payload.into();
        if payload.len() > self.max_event_bytes {
            return Err(UtilitySessionFlowError::EventTooLarge);
        }
        if self.available_credits == 0 {
            return Err(UtilitySessionFlowError::CreditsExhausted);
        }
        let sequence = self
            .last_emitted_sequence
            .checked_add(1)
            .ok_or(UtilitySessionFlowError::SequenceExhausted)?;
        self.available_credits -= 1;
        self.last_emitted_sequence = sequence;
        Ok(UtilitySessionEvent {
            sequence,
            kind,
            payload,
        })
    }

    /// Acknowledges durably committed events and replenishes producer credits.
    ///
    /// Credits cannot exceed the number of newly acknowledged events. This
    /// prevents a consumer from accidentally expanding the negotiated window.
    pub fn acknowledge(
        &mut self,
        committed_sequence: u64,
        credits: u32,
    ) -> Result<(), UtilitySessionFlowError> {
        if self.closed {
            return Err(UtilitySessionFlowError::Closed);
        }
        if committed_sequence < self.committed_sequence
            || committed_sequence > self.last_emitted_sequence
        {
            return Err(UtilitySessionFlowError::InvalidAcknowledgement);
        }
        let outstanding = self.last_emitted_sequence - committed_sequence;
        let Some(window_usage) = u64::from(self.available_credits).checked_add(outstanding) else {
            return Err(UtilitySessionFlowError::InvalidAcknowledgement);
        };
        let available_capacity = u64::from(self.max_credits)
            .checked_sub(window_usage)
            .ok_or(UtilitySessionFlowError::InvalidAcknowledgement)?;
        if u64::from(credits) > available_capacity {
            return Err(UtilitySessionFlowError::InvalidAcknowledgement);
        }
        self.committed_sequence = committed_sequence;
        self.available_credits += credits;
        Ok(())
    }

    /// Closes the producer and returns the final emitted sequence.
    pub const fn close(&mut self) -> u64 {
        self.closed = true;
        self.last_emitted_sequence
    }

    /// Returns currently available event credits.
    #[must_use]
    pub const fn available_credits(&self) -> u32 {
        self.available_credits
    }

    /// Returns the highest sequence acknowledged as durably committed.
    #[must_use]
    pub const fn committed_sequence(&self) -> u64 {
        self.committed_sequence
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn events_stop_at_the_credit_boundary_and_resume_after_ack() {
        let mut flow = UtilitySessionFlowControl::new(2, 16).unwrap();
        assert_eq!(flow.emit(1, b"first".to_vec()).unwrap().sequence(), 1);
        assert_eq!(flow.emit(1, b"second".to_vec()).unwrap().sequence(), 2);
        assert_eq!(
            flow.emit(1, b"blocked".to_vec()),
            Err(UtilitySessionFlowError::CreditsExhausted)
        );

        flow.acknowledge(2, 2).unwrap();
        assert_eq!(flow.available_credits(), 2);
        assert_eq!(flow.committed_sequence(), 2);
        assert_eq!(flow.emit(7, b"third".to_vec()).unwrap().sequence(), 3);
    }

    #[test]
    fn acknowledgement_can_return_credits_after_commit() {
        let mut flow = UtilitySessionFlowControl::new(2, 16).unwrap();
        flow.emit(1, b"first".to_vec()).unwrap();
        flow.emit(1, b"second".to_vec()).unwrap();

        flow.acknowledge(2, 1).unwrap();
        assert_eq!(flow.available_credits(), 1);
        flow.acknowledge(2, 1).unwrap();
        assert_eq!(flow.available_credits(), 2);
        assert_eq!(
            flow.acknowledge(2, 1),
            Err(UtilitySessionFlowError::InvalidAcknowledgement)
        );
    }

    #[test]
    fn invalid_ack_payload_and_close_fail_closed() {
        assert!(UtilitySessionFlowControl::new(0, 1).is_err());
        assert!(UtilitySessionFlowControl::new(1, 0).is_err());
        let mut flow = UtilitySessionFlowControl::new(1, 4).unwrap();
        assert_eq!(
            flow.emit(0, Vec::new()),
            Err(UtilitySessionFlowError::InvalidEventKind)
        );
        assert_eq!(
            flow.emit(1, b"large".to_vec()),
            Err(UtilitySessionFlowError::EventTooLarge)
        );
        flow.emit(1, b"ok".to_vec()).unwrap();
        assert_eq!(
            flow.acknowledge(2, 1),
            Err(UtilitySessionFlowError::InvalidAcknowledgement)
        );
        assert_eq!(flow.close(), 1);
        assert_eq!(
            flow.emit(1, Vec::new()),
            Err(UtilitySessionFlowError::Closed)
        );
    }

    #[test]
    fn emitter_delivers_only_credited_events_to_the_host_sink() {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&captured);
        let emitter = UtilitySessionEmitter::new_for_host(1, 16, move |event| {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(event);
            Ok(())
        })
        .unwrap();

        assert_eq!(emitter.emit(9, b"one".to_vec()).unwrap(), 1);
        assert!(emitter.emit(9, b"blocked".to_vec()).is_err());
        assert_eq!(captured.lock().unwrap().len(), 1);

        emitter.acknowledge_from_host(1, 1).unwrap();
        assert_eq!(emitter.emit(9, b"two".to_vec()).unwrap(), 2);
        assert_eq!(emitter.close_from_host(), 2);
        assert!(emitter.emit(9, b"closed".to_vec()).is_err());
        assert_eq!(captured.lock().unwrap().len(), 2);
    }

    #[test]
    fn emitter_closes_when_the_host_sink_rejects_an_event() {
        let emitter = UtilitySessionEmitter::new_for_host(1, 16, |_| {
            Err(ApplicationError::new("host queue is closed"))
        })
        .unwrap();
        assert!(emitter.emit(1, b"lost".to_vec()).is_err());
        assert!(emitter.emit(1, b"after-close".to_vec()).is_err());
    }
}
