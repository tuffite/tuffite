//! Safe main-Runtime access to supervised long-lived Utility sessions.

use std::ffi::c_void;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::slice;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

use tuffite_abi::{ByteSlice, Status};

use crate::utility_session::{
    DEFAULT_UTILITY_SESSION_CREDITS, MAX_UTILITY_SESSION_CREDITS, MAX_UTILITY_SESSION_EVENT_BYTES,
    UtilitySessionEvent,
};

const MAX_SERVICE_NAME_BYTES: usize = 128;
const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
const STATE_OPENING: u8 = 0;
const STATE_ACTIVE: u8 = 1;
const STATE_CLOSING: u8 = 2;
const STATE_CLOSED: u8 = 3;

/// Host-side error for one Utility session operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum UtilitySessionHostError {
    #[error("Utility sessions are unavailable in this Framework")]
    Unavailable,
    #[error("Utility session input is invalid")]
    InvalidArgument,
    #[error("Utility session handle or state is invalid")]
    InvalidState,
    #[error("Utility session is resource constrained")]
    ResourceExhausted,
    #[error("Utility session failed internally")]
    Internal,
}

/// Session limits negotiated before the Utility process receives extension code.
#[derive(Clone, Debug)]
pub struct UtilitySessionOptions {
    pub init: Vec<u8>,
    pub initial_event_credits: u32,
    pub max_event_bytes: usize,
}

impl Default for UtilitySessionOptions {
    fn default() -> Self {
        Self {
            init: Vec::new(),
            initial_event_credits: DEFAULT_UTILITY_SESSION_CREDITS,
            max_event_bytes: MAX_UTILITY_SESSION_EVENT_BYTES,
        }
    }
}

/// Terminal state delivered exactly once when a Utility session closes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UtilitySessionClosed {
    pub status: Result<(), UtilitySessionHostError>,
    pub last_sequence: u64,
    pub summary: Vec<u8>,
}

/// Only this validated capability's callable values and own native context.
#[derive(Clone, Copy, Debug)]
struct SessionClient(tuffite_abi::UtilitySessionClient);

// SAFETY: The accepted capability contract keeps callbacks and their own context
// alive and thread-safe through application stop. No provider table pointer is held.
unsafe impl Send for SessionClient {}
// SAFETY: The same accepted contract permits concurrent session operations.
unsafe impl Sync for SessionClient {}

/// Cloneable entrypoint for creating long-lived Utility sessions.
#[derive(Clone, Debug)]
pub struct UtilitySessionHost {
    callbacks: SessionClient,
    next_session_id: Arc<AtomicU64>,
}

impl UtilitySessionHost {
    pub(crate) fn new(callbacks: tuffite_abi::UtilitySessionClient) -> Self {
        Self {
            callbacks: SessionClient(callbacks),
            next_session_id: Arc::new(AtomicU64::new(1)),
        }
    }

    /// Creates one session. Commands may be sent after `on_created` reports success.
    /// Event and close callbacks run on Chromium's browser sequence and must
    /// only validate/copy/enqueue bounded data before returning. Persistence
    /// and other blocking work belongs on the application's worker runtime; it
    /// grants more credits only after that work reaches its commit boundary.
    pub fn create<E, C, O>(
        &self,
        service: &str,
        options: &UtilitySessionOptions,
        on_event: E,
        on_closed: C,
        on_created: O,
    ) -> Result<UtilitySessionHandle, UtilitySessionHostError>
    where
        E: Fn(UtilitySessionEvent) -> bool + Send + Sync + 'static,
        C: FnOnce(UtilitySessionClosed) + Send + 'static,
        O: FnOnce(Result<(), UtilitySessionHostError>) + Send + 'static,
    {
        validate_name(service)?;
        if options.init.len() > MAX_MESSAGE_BYTES
            || options.initial_event_credits == 0
            || options.initial_event_credits > MAX_UTILITY_SESSION_CREDITS
            || options.max_event_bytes == 0
            || options.max_event_bytes > MAX_UTILITY_SESSION_EVENT_BYTES
        {
            return Err(UtilitySessionHostError::InvalidArgument);
        }
        let Some(create) = self.callbacks.0.create else {
            return Err(UtilitySessionHostError::Unavailable);
        };
        let session_id = self.next_session_id.fetch_add(1, Ordering::Relaxed);
        if session_id == 0 {
            return Err(UtilitySessionHostError::ResourceExhausted);
        }
        let state = Arc::new(AtomicU8::new(STATE_OPENING));
        let session_context = Box::new(SessionContext {
            state: Arc::clone(&state),
            on_event: Box::new(on_event),
            on_closed: Some(Box::new(on_closed)),
        });
        let create_context = Box::new(CreateContext {
            state: Arc::clone(&state),
            callback: Some(Box::new(on_created)),
        });
        let session_data = Box::into_raw(session_context).cast::<c_void>();
        let create_data = Box::into_raw(create_context).cast::<c_void>();
        // SAFETY: Callback contexts transfer only when Framework accepts the
        // operation. Framework copies service and init bytes before return.
        let status = unsafe {
            create(
                self.callbacks.0.user_data,
                service.as_ptr(),
                service.len(),
                session_id,
                options.init.as_ptr(),
                options.init.len(),
                options.initial_event_credits,
                options.max_event_bytes,
                session_data,
                Some(on_session_event),
                Some(on_session_closed),
                create_data,
                Some(on_session_created),
            )
        };
        if status != Status::Ok {
            // SAFETY: A rejected operation does not consume callback contexts.
            drop(unsafe { Box::from_raw(session_data.cast::<SessionContext>()) });
            drop(unsafe { Box::from_raw(create_data.cast::<CreateContext>()) });
            return Err(map_status(status));
        }
        Ok(UtilitySessionHandle {
            callbacks: self.callbacks,
            service: service.to_owned(),
            session_id,
            next_command_id: Arc::new(AtomicU64::new(1)),
            state,
        })
    }
}

/// Opaque main-Runtime handle for one supervised Utility session.
#[derive(Clone, Debug)]
pub struct UtilitySessionHandle {
    callbacks: SessionClient,
    service: String,
    session_id: u64,
    next_command_id: Arc<AtomicU64>,
    state: Arc<AtomicU8>,
}

impl UtilitySessionHandle {
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.session_id
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state.load(Ordering::Acquire) == STATE_ACTIVE
    }

    /// Sends one bounded command and reports its one-shot response.
    pub fn send<F>(&self, command: &[u8], reply: F) -> Result<u64, UtilitySessionHostError>
    where
        F: FnOnce(Result<Vec<u8>, UtilitySessionHostError>) + Send + 'static,
    {
        if !self.is_active() || command.len() > MAX_MESSAGE_BYTES {
            return Err(UtilitySessionHostError::InvalidState);
        }
        let Some(send) = self.callbacks.0.send else {
            return Err(UtilitySessionHostError::Unavailable);
        };
        let command_id = self.next_command_id.fetch_add(1, Ordering::Relaxed);
        if command_id == 0 {
            return Err(UtilitySessionHostError::ResourceExhausted);
        }
        let context = Box::into_raw(Box::new(ReplyContext {
            callback: Some(Box::new(reply)),
        }))
        .cast::<c_void>();
        // SAFETY: Callback ownership transfers only on success; input bytes are
        // copied before this call returns.
        let status = unsafe {
            send(
                self.callbacks.0.user_data,
                self.service.as_ptr(),
                self.service.len(),
                self.session_id,
                command_id,
                command.as_ptr(),
                command.len(),
                context,
                Some(on_reply),
            )
        };
        if status != Status::Ok {
            // SAFETY: Rejected calls retain callback ownership.
            drop(unsafe { Box::from_raw(context.cast::<ReplyContext>()) });
            return Err(map_status(status));
        }
        Ok(command_id)
    }

    /// Acknowledges events only after the application's durable commit.
    pub fn acknowledge(
        &self,
        committed_sequence: u64,
        credits: u32,
    ) -> Result<(), UtilitySessionHostError> {
        if !self.is_active() {
            return Err(UtilitySessionHostError::InvalidState);
        }
        let Some(acknowledge) = self.callbacks.0.acknowledge else {
            return Err(UtilitySessionHostError::Unavailable);
        };
        // SAFETY: Framework copies the service name before return.
        status_result(unsafe {
            acknowledge(
                self.callbacks.0.user_data,
                self.service.as_ptr(),
                self.service.len(),
                self.session_id,
                committed_sequence,
                credits,
            )
        })
    }

    /// Requests cooperative cancellation of one active command.
    pub fn cancel(&self, command_id: u64) -> Result<(), UtilitySessionHostError> {
        if !self.is_active() || command_id == 0 {
            return Err(UtilitySessionHostError::InvalidState);
        }
        let Some(cancel) = self.callbacks.0.cancel else {
            return Err(UtilitySessionHostError::Unavailable);
        };
        // SAFETY: Framework copies the service name before return.
        status_result(unsafe {
            cancel(
                self.callbacks.0.user_data,
                self.service.as_ptr(),
                self.service.len(),
                self.session_id,
                command_id,
            )
        })
    }

    /// Starts graceful close. The callback receives the bounded close summary.
    pub fn close<F>(&self, reason: u32, reply: F) -> Result<(), UtilitySessionHostError>
    where
        F: FnOnce(Result<Vec<u8>, UtilitySessionHostError>) + Send + 'static,
    {
        if self
            .state
            .compare_exchange(
                STATE_ACTIVE,
                STATE_CLOSING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_err()
        {
            return Err(UtilitySessionHostError::InvalidState);
        }
        let Some(close) = self.callbacks.0.close else {
            self.state.store(STATE_ACTIVE, Ordering::Release);
            return Err(UtilitySessionHostError::Unavailable);
        };
        let context = Box::into_raw(Box::new(ReplyContext {
            callback: Some(Box::new(reply)),
        }))
        .cast::<c_void>();
        // SAFETY: Callback ownership transfers only on success; Framework copies
        // the service name before return.
        let status = unsafe {
            close(
                self.callbacks.0.user_data,
                self.service.as_ptr(),
                self.service.len(),
                self.session_id,
                reason,
                context,
                Some(on_reply),
            )
        };
        if status != Status::Ok {
            self.state.store(STATE_ACTIVE, Ordering::Release);
            // SAFETY: Rejected calls retain callback ownership.
            drop(unsafe { Box::from_raw(context.cast::<ReplyContext>()) });
            return Err(map_status(status));
        }
        Ok(())
    }
}

struct SessionContext {
    state: Arc<AtomicU8>,
    on_event: Box<dyn Fn(UtilitySessionEvent) -> bool + Send + Sync>,
    on_closed: Option<ClosedCallback>,
}

struct CreateContext {
    state: Arc<AtomicU8>,
    callback: Option<OpenCallback>,
}

struct ReplyContext {
    callback: Option<ReplyCallback>,
}

type ClosedCallback = Box<dyn FnOnce(UtilitySessionClosed) + Send>;
type OpenCallback = Box<dyn FnOnce(Result<(), UtilitySessionHostError>) + Send>;
type ReplyCallback = Box<dyn FnOnce(Result<Vec<u8>, UtilitySessionHostError>) + Send>;

unsafe extern "C" fn on_session_event(
    user_data: *mut c_void,
    sequence: u64,
    kind: u32,
    payload: ByteSlice,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Framework retains the session context until `on_session_closed`.
        let Some(context) = (unsafe { user_data.cast::<SessionContext>().as_ref() }) else {
            return Status::InvalidArgument;
        };
        let Some(payload) = copy_bytes(payload) else {
            return Status::InvalidArgument;
        };
        if (context.on_event)(UtilitySessionEvent::new(sequence, kind, payload)) {
            Status::Ok
        } else {
            Status::InvalidState
        }
    }))
    .unwrap_or(Status::Internal)
}

unsafe extern "C" fn on_session_closed(
    user_data: *mut c_void,
    status: Status,
    last_sequence: u64,
    summary: ByteSlice,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if user_data.is_null() {
            return;
        }
        // SAFETY: Framework invokes closed exactly once after disabling events.
        let mut context = unsafe { Box::from_raw(user_data.cast::<SessionContext>()) };
        context.state.store(STATE_CLOSED, Ordering::Release);
        let summary = copy_bytes(summary).unwrap_or_default();
        // Terminal closure uniquely owns the allocation after events are disabled.
        // Taking this FnOnce needs no shared lock and never calls user code under one.
        if let Some(callback) = context.on_closed.take() {
            callback(UtilitySessionClosed {
                status: status_result(status),
                last_sequence,
                summary,
            });
        }
    }));
}

unsafe extern "C" fn on_session_created(user_data: *mut c_void, status: Status) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if user_data.is_null() {
            return;
        }
        // SAFETY: Framework invokes this one-shot callback exactly once.
        let mut context = unsafe { Box::from_raw(user_data.cast::<CreateContext>()) };
        context.state.store(
            if status == Status::Ok {
                STATE_ACTIVE
            } else {
                STATE_CLOSED
            },
            Ordering::Release,
        );
        if let Some(callback) = context.callback.take() {
            callback(status_result(status));
        }
    }));
}

unsafe extern "C" fn on_reply(
    user_data: *mut c_void,
    status: Status,
    payload: ByteSlice,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        if user_data.is_null() {
            return Status::InvalidArgument;
        }
        // SAFETY: Framework invokes this one-shot callback exactly once.
        let mut context = unsafe { Box::from_raw(user_data.cast::<ReplyContext>()) };
        let result = status_result(status)
            .and_then(|()| copy_bytes(payload).ok_or(UtilitySessionHostError::InvalidArgument));
        if let Some(callback) = context.callback.take() {
            callback(result);
        }
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

fn copy_bytes(payload: ByteSlice) -> Option<Vec<u8>> {
    if payload.len == 0 {
        return Some(Vec::new());
    }
    if payload.data.is_null() || payload.len > MAX_MESSAGE_BYTES {
        return None;
    }
    // SAFETY: ABI callback guarantees a readable slice for this invocation.
    Some(unsafe { slice::from_raw_parts(payload.data, payload.len) }.to_vec())
}

fn validate_name(name: &str) -> Result<(), UtilitySessionHostError> {
    let bytes = name.as_bytes();
    let valid = !bytes.is_empty()
        && bytes.len() <= MAX_SERVICE_NAME_BYTES
        && (bytes[0].is_ascii_alphabetic() || matches!(bytes[0], b'_' | b'$'))
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'));
    valid
        .then_some(())
        .ok_or(UtilitySessionHostError::InvalidArgument)
}

fn status_result(status: Status) -> Result<(), UtilitySessionHostError> {
    if status == Status::Ok {
        Ok(())
    } else {
        Err(map_status(status))
    }
}

fn map_status(status: Status) -> UtilitySessionHostError {
    match status {
        Status::InvalidArgument | Status::IncompatibleAbi | Status::NotFound => {
            UtilitySessionHostError::InvalidArgument
        }
        Status::InvalidHandle | Status::InvalidState => UtilitySessionHostError::InvalidState,
        Status::ResourceExhausted => UtilitySessionHostError::ResourceExhausted,
        Status::Ok | Status::Internal => UtilitySessionHostError::Internal,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::{Arc, Mutex};

    use super::*;

    #[test]
    fn callback_panics_are_contained_at_the_utility_abi_boundary() {
        let state = Arc::new(AtomicU8::new(STATE_ACTIVE));
        let session = Box::into_raw(Box::new(SessionContext {
            state: state.clone(),
            on_event: Box::new(|_| panic!("event callback")),
            on_closed: Some(Box::new(|_| panic!("closed callback"))),
        }))
        .cast::<c_void>();
        // SAFETY: Live callback allocations obey single terminal ownership.
        unsafe {
            assert_eq!(
                on_session_event(session, 1, 1, ByteSlice::from_slice(&[])),
                Status::Internal
            );
            on_session_closed(session, Status::Ok, 1, ByteSlice::from_slice(&[]));
        }
        assert_eq!(state.load(Ordering::Acquire), STATE_CLOSED);
        let reply = Box::into_raw(Box::new(ReplyContext {
            callback: Some(Box::new(|_| panic!("reply callback"))),
        }))
        .cast::<c_void>();
        // SAFETY: This one-shot reply consumes its sole allocation.
        assert_eq!(
            unsafe { on_reply(reply, Status::Ok, ByteSlice::from_slice(&[])) },
            Status::Internal
        );
        let created = Box::into_raw(Box::new(CreateContext {
            state: state.clone(),
            callback: Some(Box::new(|_| panic!("create callback"))),
        }))
        .cast::<c_void>();
        // SAFETY: This one-shot creation callback consumes its sole allocation.
        unsafe {
            on_session_created(created, Status::Ok);
        }
        assert_eq!(state.load(Ordering::Acquire), STATE_ACTIVE);
    }

    static SESSION_DATA: AtomicU64 = AtomicU64::new(0);
    static EVENT_CALLBACK: Mutex<Option<tuffite_abi::UtilityHostEventFn>> = Mutex::new(None);
    static CLOSED_CALLBACK: Mutex<Option<tuffite_abi::UtilityHostClosedFn>> = Mutex::new(None);
    static ACK: Mutex<Option<(u64, u32)>> = Mutex::new(None);
    static CANCELLED: AtomicU64 = AtomicU64::new(0);

    unsafe extern "C" fn create(
        _host_data: *mut c_void,
        _name: *const u8,
        _name_len: usize,
        _session_id: u64,
        _init: *const u8,
        _init_len: usize,
        _initial_event_credits: u32,
        _max_event_bytes: usize,
        session_data: *mut c_void,
        event: Option<tuffite_abi::UtilityHostEventFn>,
        closed: Option<tuffite_abi::UtilityHostClosedFn>,
        create_data: *mut c_void,
        created: Option<tuffite_abi::UtilityHostCreateReplyFn>,
    ) -> Status {
        SESSION_DATA.store(session_data as usize as u64, Ordering::Release);
        *EVENT_CALLBACK.lock().unwrap() = event;
        *CLOSED_CALLBACK.lock().unwrap() = closed;
        if let Some(created) = created {
            // SAFETY: Test caller supplied this one-shot context.
            unsafe { created(create_data, Status::Ok) };
        }
        Status::Ok
    }

    unsafe extern "C" fn send(
        _host_data: *mut c_void,
        _name: *const u8,
        _name_len: usize,
        _session_id: u64,
        command_id: u64,
        _command: *const u8,
        _command_len: usize,
        reply_data: *mut c_void,
        reply: Option<tuffite_abi::UtilityReplyFn>,
    ) -> Status {
        let response = format!("reply:{command_id}").into_bytes();
        if let Some(reply) = reply {
            // SAFETY: Test caller supplied this one-shot context.
            unsafe { reply(reply_data, Status::Ok, ByteSlice::from_slice(&response)) };
        }
        Status::Ok
    }

    unsafe extern "C" fn acknowledge(
        _host_data: *mut c_void,
        _name: *const u8,
        _name_len: usize,
        _session_id: u64,
        committed_sequence: u64,
        credits: u32,
    ) -> Status {
        *ACK.lock().unwrap() = Some((committed_sequence, credits));
        Status::Ok
    }

    unsafe extern "C" fn cancel(
        _host_data: *mut c_void,
        _name: *const u8,
        _name_len: usize,
        _session_id: u64,
        command_id: u64,
    ) -> Status {
        CANCELLED.store(command_id, Ordering::Release);
        Status::Ok
    }

    unsafe extern "C" fn close(
        _host_data: *mut c_void,
        _name: *const u8,
        _name_len: usize,
        _session_id: u64,
        _reason: u32,
        reply_data: *mut c_void,
        reply: Option<tuffite_abi::UtilityReplyFn>,
    ) -> Status {
        let session_data =
            usize::try_from(SESSION_DATA.swap(0, Ordering::AcqRel)).unwrap() as *mut c_void;
        if let Some(closed) = CLOSED_CALLBACK.lock().unwrap().take() {
            // SAFETY: Test open stored this live session context.
            unsafe {
                closed(
                    session_data,
                    Status::Ok,
                    1,
                    ByteSlice::from_slice(b"closed"),
                );
            };
        }
        if let Some(reply) = reply {
            // SAFETY: Test caller supplied this one-shot context.
            unsafe { reply(reply_data, Status::Ok, ByteSlice::from_slice(b"summary")) };
        }
        Status::Ok
    }

    fn host() -> UtilitySessionHost {
        UtilitySessionHost::new(tuffite_abi::UtilitySessionClient {
            header: tuffite_abi::Header::for_type::<tuffite_abi::UtilitySessionClient>(),
            user_data: std::ptr::null_mut(),
            create: Some(create),
            send: Some(send),
            acknowledge: Some(acknowledge),
            cancel: Some(cancel),
            close: Some(close),
        })
    }

    #[test]
    fn safe_host_api_owns_open_event_command_ack_cancel_and_close_callbacks() {
        *ACK.lock().unwrap() = None;
        CANCELLED.store(0, Ordering::Release);
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured_events = Arc::clone(&events);
        let created = Arc::new(Mutex::new(None));
        let captured_created = Arc::clone(&created);
        let terminal = Arc::new(Mutex::new(None));
        let captured_terminal = Arc::clone(&terminal);
        let session = host()
            .create(
                "echo",
                &UtilitySessionOptions {
                    init: b"init".to_vec(),
                    initial_event_credits: 1,
                    max_event_bytes: 16,
                },
                move |event| {
                    captured_events.lock().unwrap().push(event);
                    true
                },
                move |closed| *captured_terminal.lock().unwrap() = Some(closed),
                move |result| *captured_created.lock().unwrap() = Some(result),
            )
            .unwrap();
        assert!(session.is_active());
        assert_eq!(*created.lock().unwrap(), Some(Ok(())));

        let event = EVENT_CALLBACK.lock().unwrap().expect("event callback");
        let session_data =
            usize::try_from(SESSION_DATA.load(Ordering::Acquire)).unwrap() as *mut c_void;
        // SAFETY: Fake Framework retained the live session context.
        assert_eq!(
            unsafe { event(session_data, 1, 7, ByteSlice::from_slice(b"event")) },
            Status::Ok
        );
        assert_eq!(events.lock().unwrap()[0].payload(), b"event");

        let response = Arc::new(Mutex::new(None));
        let captured_response = Arc::clone(&response);
        let command_id = session
            .send(b"command", move |value| {
                *captured_response.lock().unwrap() = Some(value);
            })
            .unwrap();
        assert_eq!(command_id, 1);
        assert_eq!(
            response.lock().unwrap().as_ref(),
            Some(&Ok(b"reply:1".to_vec()))
        );
        session.acknowledge(1, 1).unwrap();
        assert_eq!(*ACK.lock().unwrap(), Some((1, 1)));
        session.cancel(command_id).unwrap();
        assert_eq!(CANCELLED.load(Ordering::Acquire), 1);

        let close_reply = Arc::new(Mutex::new(None));
        let captured_reply = Arc::clone(&close_reply);
        session
            .close(9, move |value| {
                *captured_reply.lock().unwrap() = Some(value);
            })
            .unwrap();
        assert!(!session.is_active());
        assert_eq!(
            close_reply.lock().unwrap().as_ref(),
            Some(&Ok(b"summary".to_vec()))
        );
        assert_eq!(terminal.lock().unwrap().as_ref().unwrap().last_sequence, 1);
    }
}
