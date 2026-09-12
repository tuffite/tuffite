use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::slice;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use tuffite_abi::{ByteSlice, Handle as FfiHandle, Status, UtilityReplyFn, UtilitySessionEventFn};

use crate::application::Registrations;
use crate::handle::{Handle, HandleError, HandleKind, HandleTable, HandleType, RawHandle};
use crate::supervisor::CancellationToken;
use crate::utility_session::{
    UtilitySession, UtilitySessionCommand, UtilitySessionEmitter, UtilitySessionRequest,
};
use crate::{Utility, UtilityRequest};

enum UtilityTag {}

impl HandleType for UtilityTag {
    const KIND: HandleKind = HandleKind::UtilityService;
}

type UtilityHandle = Handle<UtilityTag>;
// The service is already Sync. This gate protects only operation lifetime:
// stop closes admission, cancels work, and drains admitted readers before cleanup.
struct UtilityInstance {
    implementation: Box<dyn Utility>,
    activity: RwLock<()>,
    closing: AtomicBool,
    cancellation: CancellationToken,
}
type SharedUtility = Arc<UtilityInstance>;
type UtilityTable = HandleTable<SharedUtility, UtilityTag>;

enum UtilitySessionTag {}

impl HandleType for UtilitySessionTag {
    const KIND: HandleKind = HandleKind::UtilitySession;
}

struct ActiveUtilitySession {
    service: FfiHandle,
    logical_id: u64,
    implementation: Arc<Mutex<Box<dyn UtilitySession>>>,
    emitter: UtilitySessionEmitter,
}

type UtilitySessionHandle = Handle<UtilitySessionTag>;
type UtilitySessionTable = HandleTable<ActiveUtilitySession, UtilitySessionTag>;

const MAX_SERVICE_NAME_BYTES: usize = 128;
const MAX_UTILITY_RESPONSE_BYTES: usize = 1024 * 1024;
const MAX_UTILITY_SESSION_COMMAND_BYTES: usize = 1024 * 1024;
const UTILITY_DEADLINE: Duration = Duration::from_secs(30);

fn services() -> &'static Mutex<UtilityTable> {
    static SERVICES: OnceLock<Mutex<UtilityTable>> = OnceLock::new();
    SERVICES.get_or_init(|| Mutex::new(UtilityTable::new()))
}

fn invocations() -> &'static Mutex<HashMap<(u64, u64), CancellationToken>> {
    static INVOCATIONS: OnceLock<Mutex<HashMap<(u64, u64), CancellationToken>>> = OnceLock::new();
    INVOCATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn sessions() -> &'static Mutex<UtilitySessionTable> {
    static SESSIONS: OnceLock<Mutex<UtilitySessionTable>> = OnceLock::new();
    SESSIONS.get_or_init(|| Mutex::new(UtilitySessionTable::new()))
}

fn session_commands() -> &'static Mutex<HashMap<(u64, u64), CancellationToken>> {
    static COMMANDS: OnceLock<Mutex<HashMap<(u64, u64), CancellationToken>>> = OnceLock::new();
    COMMANDS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
fn validate_registrations<A: Registrations>() -> Result<(), String> {
    let registrations = A::utility_services();
    for (index, registration) in registrations.iter().enumerate() {
        let name = registration.name();
        if !is_identifier(name) {
            return Err(format!("invalid utility service name: {name:?}"));
        }
        if registrations[..index]
            .iter()
            .any(|existing| existing.name() == name)
        {
            return Err(format!("duplicate utility service name: {name}"));
        }
    }
    Ok(())
}

pub(crate) fn validate_registration(
    existing: &[crate::UtilityServiceRegistration],
    registration: crate::UtilityServiceRegistration,
) -> Result<(), String> {
    let name = registration.name();
    if !is_identifier(name) || crate::utility::is_reserved_name(name) {
        return Err(format!(
            "invalid or reserved utility service name: {name:?}"
        ));
    }
    if existing.iter().any(|item| {
        item.name() == name
            || (registration.runtime() == 1
                && item.runtime() == 1
                && item.name().eq_ignore_ascii_case(name))
    }) {
        return Err(format!(
            "duplicate utility service name or V8 resource origin: {name}"
        ));
    }
    Ok(())
}

fn is_identifier(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_SERVICE_NAME_BYTES {
        return false;
    }
    let is_start = |byte: u8| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$');
    let is_continue = |byte: u8| is_start(byte) || byte.is_ascii_digit();
    is_start(bytes[0]) && bytes[1..].iter().copied().all(is_continue)
}

fn parse_handle(value: FfiHandle) -> Result<UtilityHandle, Status> {
    let raw = RawHandle::from_u64(value).map_err(map_handle_error)?;
    Handle::from_raw(raw).map_err(map_handle_error)
}

fn parse_session_handle(value: FfiHandle) -> Result<UtilitySessionHandle, Status> {
    let raw = RawHandle::from_u64(value).map_err(map_handle_error)?;
    Handle::from_raw(raw).map_err(map_handle_error)
}

const fn map_handle_error(_error: HandleError) -> Status {
    Status::InvalidHandle
}

unsafe fn borrowed_bytes<'a>(data: *const u8, len: usize) -> Result<&'a [u8], Status> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() || len > isize::MAX as usize {
        return Err(Status::InvalidArgument);
    }
    // SAFETY: Validity is guaranteed by the ABI caller for this call.
    Ok(unsafe { slice::from_raw_parts(data, len) })
}

pub(crate) unsafe extern "C" fn utility_runtime<A: Registrations>(
    name: *const u8,
    name_len: usize,
    runtime: *mut u32,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        if runtime.is_null() {
            return Status::InvalidArgument;
        }
        // SAFETY: Name bytes are readable by the caller's contract.
        let Ok(name) = (unsafe { borrowed_bytes(name, name_len) })
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| Status::InvalidArgument))
        else {
            return Status::InvalidArgument;
        };
        if name == "tuffite_js_extension" {
            // SAFETY: Writable output follows the caller contract.
            unsafe {
                *runtime = 1;
            }
            return Status::Ok;
        }
        let Some(registration) = A::utility_services()
            .iter()
            .find(|item| item.name() == name)
        else {
            return Status::InvalidArgument;
        };
        // SAFETY: The caller supplies a writable discriminator output.
        unsafe {
            *runtime = registration.runtime();
        }
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub unsafe extern "C" fn is_utility_registered<A: Registrations>(
    name: *const u8,
    name_len: usize,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: Forwarded from this function's pointer contract.
        let name = match unsafe { borrowed_bytes(name, name_len) }
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| Status::InvalidArgument))
        {
            Ok(name) if is_identifier(name) => name,
            Ok(_) => return Status::InvalidArgument,
            Err(status) => return status,
        };
        if A::utility_services()
            .iter()
            .any(|registration| registration.name() == name)
        {
            Status::Ok
        } else {
            Status::InvalidArgument
        }
    }))
    .unwrap_or(Status::Internal)
}

pub unsafe extern "C" fn start_utility<A: Registrations>(
    name: *const u8,
    name_len: usize,
    service_out: *mut FfiHandle,
) -> Status {
    // SAFETY: Legacy entry forwards the same name/output contract with empty configuration.
    unsafe {
        start_utility_with_context::<A>(name, name_len, ByteSlice::from_slice(&[]), service_out)
    }
}

/// ABI 2.5 service construction with owned startup configuration.
pub unsafe extern "C" fn start_utility_with_context<A: Registrations>(
    name: *const u8,
    name_len: usize,
    configuration: ByteSlice,
    service_out: *mut FfiHandle,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        if service_out.is_null() {
            return Status::InvalidArgument;
        }
        // SAFETY: `service_out` is writable by contract.
        unsafe { service_out.write(0) };
        // SAFETY: Forwarded from this function's pointer contract.
        let name = match unsafe { borrowed_bytes(name, name_len) }
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| Status::InvalidArgument))
        {
            Ok(name) => name,
            Err(status) => return status,
        };
        let Some(registration) = A::utility_services()
            .iter()
            .find(|registration| registration.name() == name)
        else {
            return Status::InvalidArgument;
        };
        if configuration.len > 64 * 1024 {
            return Status::InvalidArgument;
        }
        // SAFETY: Startup borrows configuration only during this call.
        let configuration = match unsafe { borrowed_bytes(configuration.data, configuration.len) } {
            Ok(bytes) => bytes.to_vec(),
            Err(status) => return status,
        };
        let cancellation = CancellationToken::new();
        let context =
            crate::utility::UtilityContext::new(name, configuration, cancellation.clone());
        let service = match catch_unwind(AssertUnwindSafe(|| registration.create(context))) {
            Ok(Ok(service)) => service,
            Ok(Err(_)) => {
                cancellation.cancel();
                return Status::InvalidState;
            }
            Err(_) => {
                cancellation.cancel();
                return Status::Internal;
            }
        };
        let Ok(handle) = services()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(Arc::new(UtilityInstance {
                implementation: service,
                activity: RwLock::new(()),
                closing: AtomicBool::new(false),
                cancellation: cancellation.clone(),
            }))
        else {
            cancellation.cancel();
            return Status::ResourceExhausted;
        };
        // SAFETY: `service_out` is writable by contract.
        unsafe { service_out.write(handle.raw().as_u64()) };
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub unsafe extern "C" fn invoke_utility(
    service: FfiHandle,
    request_id: u64,
    request: *const u8,
    request_len: usize,
    user_data: *mut std::ffi::c_void,
    reply: Option<UtilityReplyFn>,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(reply) = reply else {
            return Status::InvalidArgument;
        };
        if request_id == 0 {
            return Status::InvalidArgument;
        }
        let handle = match parse_handle(service) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        // SAFETY: Forwarded from this function's pointer contract and copied now.
        let request = match unsafe { borrowed_bytes(request, request_len) } {
            Ok(request) => request.to_vec(),
            Err(status) => return status,
        };
        let implementation = {
            let table = services()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match table.get(handle) {
                Ok(implementation) => Arc::clone(implementation),
                Err(error) => return map_handle_error(error),
            }
        };
        let key = (service, request_id);
        let cancellation = CancellationToken::new();
        {
            let mut commands = invocations()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match commands.entry(key) {
                std::collections::hash_map::Entry::Occupied(_) => return Status::InvalidArgument,
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(cancellation.clone());
                }
            }
        }
        let user_data_address = user_data as usize;
        let spawn = thread::Builder::new()
            .name(format!("tuffite-utility-{service:x}-{request_id}"))
            .spawn(move || {
                let deadline = Instant::now() + UTILITY_DEADLINE;
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    let _activity = implementation
                        .activity
                        .read()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if implementation.closing.load(Ordering::Acquire) {
                        return Err(crate::CommandError::Failed(
                            "utility is stopping".to_owned(),
                        ));
                    }
                    futures_executor::block_on(
                        implementation.implementation.invoke(UtilityRequest::new(
                            request,
                            cancellation,
                            deadline,
                        )),
                    )
                }));
                invocations()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove(&key);
                let (status, payload) = utility_outcome(outcome);
                let status = if payload.len() > MAX_UTILITY_RESPONSE_BYTES {
                    Status::ResourceExhausted
                } else {
                    status
                };
                let payload = if status == Status::ResourceExhausted {
                    Vec::new()
                } else {
                    payload
                };
                // SAFETY: The C++ caller owns `user_data` until this one-shot reply.
                let _ = unsafe {
                    reply(
                        user_data_address as *mut std::ffi::c_void,
                        status,
                        ByteSlice::from_slice(&payload),
                    )
                };
            });
        if spawn.is_err() {
            invocations()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&key);
            return Status::ResourceExhausted;
        }
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

fn utility_outcome(
    outcome: Result<crate::CommandResult, Box<dyn std::any::Any + Send>>,
) -> (Status, Vec<u8>) {
    match outcome {
        Err(_) => (Status::Internal, b"utility panicked".to_vec()),
        Ok(Ok(payload)) => (Status::Ok, payload),
        Ok(Err(crate::CommandError::UnknownCommand)) => {
            (Status::InvalidArgument, b"unknown request".to_vec())
        }
        Ok(Err(
            crate::CommandError::InvalidArgument(message)
            | crate::CommandError::SchemaMismatch(message),
        )) => (Status::InvalidArgument, message.into_bytes()),
        Ok(Err(crate::CommandError::Failed(message))) => {
            (Status::InvalidState, message.into_bytes())
        }
    }
}

pub extern "C" fn cancel_utility(service: FfiHandle, request_id: u64) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        let _ = match parse_handle(service) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        let Some(cancellation) = invocations()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&(service, request_id))
            .cloned()
        else {
            return Status::InvalidHandle;
        };
        cancellation.cancel();
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub unsafe extern "C" fn create_utility_session(
    service: FfiHandle,
    session_id: u64,
    init: *const u8,
    init_len: usize,
    initial_event_credits: u32,
    max_event_bytes: usize,
    user_data: *mut std::ffi::c_void,
    event: Option<UtilitySessionEventFn>,
    session_out: *mut FfiHandle,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        if session_id == 0 || session_out.is_null() || event.is_none() {
            return Status::InvalidArgument;
        }
        // SAFETY: `session_out` is writable by contract.
        unsafe { session_out.write(0) };
        let service_handle = match parse_handle(service) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        // SAFETY: Forwarded from this function's pointer contract and copied now.
        let init = match unsafe { borrowed_bytes(init, init_len) } {
            Ok(init) if init.len() <= MAX_UTILITY_SESSION_COMMAND_BYTES => init.to_vec(),
            Ok(_) => return Status::ResourceExhausted,
            Err(status) => return status,
        };
        let implementation = {
            let table = services()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match table.get(service_handle) {
                Ok(implementation) => Arc::clone(implementation),
                Err(error) => return map_handle_error(error),
            }
        };
        let event = event.expect("checked above");
        let user_data_address = user_data as usize;
        let Ok(emitter) = UtilitySessionEmitter::new_for_host(
            initial_event_credits,
            max_event_bytes,
            move |message| {
                // SAFETY: The Host retains callback state until session close
                // completes. The payload is borrowed only for this callback.
                let status = unsafe {
                    event(
                        user_data_address as *mut std::ffi::c_void,
                        session_id,
                        message.sequence(),
                        message.kind(),
                        ByteSlice::from_slice(message.payload()),
                    )
                };
                if status == Status::Ok {
                    Ok(())
                } else {
                    Err(crate::ApplicationError::new(format!(
                        "Utility session event sink rejected event: {status:?}"
                    )))
                }
            },
        ) else {
            return Status::InvalidArgument;
        };
        // Hold the operation gate through publication so stop cannot miss a
        // session created just before its ownership snapshot.
        let _activity = implementation
            .activity
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if implementation.closing.load(Ordering::Acquire) {
            return Status::InvalidState;
        }
        let session =
            {
                match implementation.implementation.create_session(
                    UtilitySessionRequest::new_for_host(session_id, init, emitter.clone()),
                ) {
                    Ok(session) => session,
                    Err(_) => return Status::InvalidState,
                }
            };
        let active = ActiveUtilitySession {
            service,
            logical_id: session_id,
            implementation: Arc::new(Mutex::new(session)),
            emitter,
        };
        let Ok(handle) = sessions()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(active)
        else {
            return Status::ResourceExhausted;
        };
        // SAFETY: `session_out` is writable by contract.
        unsafe { session_out.write(handle.raw().as_u64()) };
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub unsafe extern "C" fn send_utility_session(
    session: FfiHandle,
    command_id: u64,
    command: *const u8,
    command_len: usize,
    user_data: *mut std::ffi::c_void,
    reply: Option<UtilityReplyFn>,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(reply) = reply else {
            return Status::InvalidArgument;
        };
        if command_id == 0 {
            return Status::InvalidArgument;
        }
        let handle = match parse_session_handle(session) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        // SAFETY: Forwarded from this function's pointer contract and copied now.
        let command = match unsafe { borrowed_bytes(command, command_len) } {
            Ok(command) if command.len() <= MAX_UTILITY_SESSION_COMMAND_BYTES => command.to_vec(),
            Ok(_) => return Status::ResourceExhausted,
            Err(status) => return status,
        };
        let implementation = {
            let table = sessions()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match table.get(handle) {
                Ok(session) => Arc::clone(&session.implementation),
                Err(error) => return map_handle_error(error),
            }
        };
        let key = (session, command_id);
        let cancellation = CancellationToken::new();
        {
            let mut commands = session_commands()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            match commands.entry(key) {
                std::collections::hash_map::Entry::Occupied(_) => return Status::InvalidArgument,
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(cancellation.clone());
                }
            }
        }
        let user_data_address = user_data as usize;
        let spawn = thread::Builder::new()
            .name(format!("tuffite-utility-session-{session:x}-{command_id}"))
            .spawn(move || {
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    let mut implementation = implementation
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    futures_executor::block_on(implementation.send(
                        UtilitySessionCommand::new_for_host(command_id, command, cancellation),
                    ))
                }));
                session_commands()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .remove(&key);
                let (status, payload) = utility_outcome(outcome);
                let status = if payload.len() > MAX_UTILITY_RESPONSE_BYTES {
                    Status::ResourceExhausted
                } else {
                    status
                };
                let payload = if status == Status::ResourceExhausted {
                    Vec::new()
                } else {
                    payload
                };
                // SAFETY: The Host owns callback state until this one-shot reply.
                let _ = unsafe {
                    reply(
                        user_data_address as *mut std::ffi::c_void,
                        status,
                        ByteSlice::from_slice(&payload),
                    )
                };
            });
        if spawn.is_err() {
            session_commands()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&key);
            return Status::ResourceExhausted;
        }
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub extern "C" fn acknowledge_utility_session(
    session: FfiHandle,
    committed_sequence: u64,
    credits: u32,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        let handle = match parse_session_handle(session) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        let emitter = {
            let table = sessions()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let active = match table.get(handle) {
                Ok(active) => active,
                Err(error) => return map_handle_error(error),
            };
            active.emitter.clone()
        };
        if emitter
            .acknowledge_from_host(committed_sequence, credits)
            .is_err()
        {
            return Status::InvalidArgument;
        }
        // Credits belong to the emitter, not the mutable command handler.
        // A pending send may await credits while holding its implementation
        // lock; acknowledgement must remain independent and non-blocking.
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub extern "C" fn cancel_utility_session(session: FfiHandle, command_id: u64) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        if parse_session_handle(session).is_err() {
            return Status::InvalidHandle;
        }
        let Some(cancellation) = session_commands()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&(session, command_id))
            .cloned()
        else {
            return Status::InvalidHandle;
        };
        cancellation.cancel();
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub unsafe extern "C" fn close_utility_session(
    session: FfiHandle,
    reason: u32,
    user_data: *mut std::ffi::c_void,
    reply: Option<UtilityReplyFn>,
) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        let Some(reply) = reply else {
            return Status::InvalidArgument;
        };
        let handle = match parse_session_handle(session) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        let active = match sessions()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(handle)
        {
            Ok(active) => active,
            Err(error) => return map_handle_error(error),
        };
        active.emitter.close_from_host();
        let cancellations = session_commands()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|((active_session, _), _)| *active_session == session)
            .map(|(_, cancellation)| cancellation.clone())
            .collect::<Vec<_>>();
        for cancellation in cancellations {
            cancellation.cancel();
        }
        let user_data_address = user_data as usize;
        let spawn = thread::Builder::new()
            .name(format!(
                "tuffite-utility-session-close-{:x}-{}",
                active.service, active.logical_id
            ))
            .spawn(move || {
                let outcome = catch_unwind(AssertUnwindSafe(|| {
                    let mut implementation = active
                        .implementation
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    implementation.close(reason)
                }));
                let (status, payload) = utility_outcome(outcome);
                let status = if payload.len() > MAX_UTILITY_RESPONSE_BYTES {
                    Status::ResourceExhausted
                } else {
                    status
                };
                let payload = if status == Status::ResourceExhausted {
                    Vec::new()
                } else {
                    payload
                };
                // SAFETY: The Host owns callback state until this one-shot reply.
                let _ = unsafe {
                    reply(
                        user_data_address as *mut std::ffi::c_void,
                        status,
                        ByteSlice::from_slice(&payload),
                    )
                };
            });
        if spawn.is_err() {
            return Status::ResourceExhausted;
        }
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

pub unsafe extern "C" fn stop_utility(service: FfiHandle) -> Status {
    catch_unwind(AssertUnwindSafe(|| {
        let handle = match parse_handle(service) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        let implementation = match services()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(handle)
        {
            Ok(implementation) => implementation,
            Err(error) => return map_handle_error(error),
        };
        implementation.closing.store(true, Ordering::Release);
        implementation.cancellation.cancel();
        let cancellations = invocations()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|((active_service, _), _)| *active_service == service)
            .map(|(_, cancellation)| cancellation.clone())
            .collect::<Vec<_>>();
        for cancellation in cancellations {
            cancellation.cancel();
        }
        // No table locks are held while waiting. After the gate drains, closing
        // rejects queued readers; release it before any session/user cleanup.
        drop(
            implementation
                .activity
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        let owned_sessions = {
            let table = sessions()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            table
                .handles()
                .filter(|session| {
                    table
                        .get(*session)
                        .is_ok_and(|active| active.service == service)
                })
                .collect::<Vec<_>>()
        };
        for session in owned_sessions {
            let raw_session = session.raw().as_u64();
            let cancellations = session_commands()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .filter(|((active_session, _), _)| *active_session == raw_session)
                .map(|(_, cancellation)| cancellation.clone())
                .collect::<Vec<_>>();
            for cancellation in cancellations {
                cancellation.cancel();
            }
            let removed = sessions()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(session);
            let Ok(active) = removed else {
                continue;
            };
            active.emitter.close_from_host();
            let _ = active
                .implementation
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .close(0);
        }
        match implementation.implementation.shutdown() {
            Ok(()) => Status::Ok,
            Err(_) => Status::Internal,
        }
    }))
    .unwrap_or(Status::Internal)
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;
    use std::slice;
    use std::sync::Mutex;
    use std::time::Duration;

    use super::*;
    use crate::{
        ApplicationResult, CommandResult, UtilityFuture, UtilityServiceRegistration,
        UtilitySessionFuture,
    };

    static REPLY: Mutex<Option<(Status, Vec<u8>)>> = Mutex::new(None);
    type SessionEventRecord = (u64, u64, u32, Vec<u8>);
    static SESSION_EVENTS: Mutex<Vec<SessionEventRecord>> = Mutex::new(Vec::new());
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn v8_names_cannot_collide_after_host_canonicalization() {
        struct Assets;
        impl crate::utility::V8 for Assets {
            fn new(_: crate::V8Context) -> ApplicationResult<Self> {
                Ok(Self)
            }
            fn entry(&self) -> &'static str {
                "entry.mjs"
            }
        }
        let existing = [crate::UtilityServiceRegistration::v8::<Assets>("Parser")];
        assert!(
            validate_registration(
                &existing,
                crate::UtilityServiceRegistration::v8::<Assets>("parser")
            )
            .is_err()
        );
        assert!(
            validate_registration(
                &existing,
                crate::UtilityServiceRegistration::v8::<Assets>("indexer")
            )
            .is_ok()
        );
    }

    #[test]
    fn credits_do_not_wait_for_an_active_command_lock() {
        let _test = TEST_LOCK.lock().unwrap();
        let emitter = UtilitySessionEmitter::new_for_host(1, 1024, |_| Ok(())).unwrap();
        emitter.emit(1, Vec::new()).unwrap();
        let implementation: Arc<Mutex<Box<dyn UtilitySession>>> =
            Arc::new(Mutex::new(Box::new(EchoSession {
                emitter: emitter.clone(),
            })));
        let active = ActiveUtilitySession {
            service: 1,
            logical_id: 1,
            implementation: implementation.clone(),
            emitter: emitter.clone(),
        };
        let handle = sessions().lock().unwrap().insert(active).unwrap();
        // Simulate send awaiting credit while exclusively borrowing its handler.
        let pending_command = implementation.lock().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            tx.send(acknowledge_utility_session(handle.raw().as_u64(), 1, 1))
                .unwrap();
        });
        let result = rx.recv_timeout(std::time::Duration::from_secs(1));
        drop(pending_command);
        worker.join().unwrap();
        sessions().lock().unwrap().remove(handle).unwrap();
        assert_eq!(result.unwrap(), Status::Ok);
        assert_eq!(emitter.available_credits(), 1);
    }

    #[test]
    fn utility_stop_closes_admission_before_waiting_for_active_operations() {
        let _test = TEST_LOCK.lock().unwrap();
        let instance = Arc::new(UtilityInstance {
            implementation: Box::new(EchoService),
            activity: RwLock::new(()),
            closing: AtomicBool::new(false),
            cancellation: CancellationToken::new(),
        });
        let handle = services().lock().unwrap().insert(instance.clone()).unwrap();
        let active_operation = instance.activity.read().unwrap();
        let stopping = std::thread::spawn(move || {
            // SAFETY: The test owns this live service handle and joins the stop.
            unsafe { stop_utility(handle.raw().as_u64()) }
        });
        let deadline = Instant::now() + Duration::from_secs(2);
        while !instance.closing.load(Ordering::Acquire) && Instant::now() < deadline {
            std::thread::yield_now();
        }
        let admission_closed = instance.closing.load(Ordering::Acquire);
        let removed = services().lock().unwrap().get(handle).is_err();
        drop(active_operation);
        assert_eq!(stopping.join().unwrap(), Status::Ok);
        assert!(admission_closed);
        assert!(removed);
    }

    static STARTUP_CONTEXT: Mutex<Option<crate::UtilityContext>> = Mutex::new(None);
    #[test]
    fn utility_startup_context_owns_config_and_observes_stop() {
        let _test = TEST_LOCK.lock().unwrap();
        let mut configuration = b"service-local".to_vec();
        let mut handle = 0;
        // SAFETY: Name/config/output are valid for this synchronous call.
        assert_eq!(
            unsafe {
                start_utility_with_context::<TestApplication>(
                    b"echo".as_ptr(),
                    4,
                    ByteSlice::from_slice(&configuration),
                    &raw mut handle,
                )
            },
            Status::Ok
        );
        configuration.fill(0);
        let context = STARTUP_CONTEXT.lock().unwrap().clone().unwrap();
        assert_eq!(context.name(), "echo");
        assert_eq!(context.configuration(), b"service-local");
        assert!(!context.is_stopping());
        // SAFETY: The test owns this service handle with no outstanding calls.
        assert_eq!(unsafe { stop_utility(handle) }, Status::Ok);
        assert!(context.is_stopping());
        let too_large = vec![0; 64 * 1024 + 1];
        handle = 42;
        // SAFETY: All buffers are live; oversized configuration must reject before constructing.
        assert_eq!(
            unsafe {
                start_utility_with_context::<TestApplication>(
                    b"echo".as_ptr(),
                    4,
                    ByteSlice::from_slice(&too_large),
                    &raw mut handle,
                )
            },
            Status::InvalidArgument
        );
        assert_eq!(handle, 0);
    }

    struct EchoService;

    struct EchoSession {
        emitter: UtilitySessionEmitter,
    }

    impl UtilitySession for EchoSession {
        fn send(&mut self, command: UtilitySessionCommand) -> UtilitySessionFuture<'_> {
            Box::pin(async move {
                if command.payload() == b"wait" {
                    for _ in 0..100 {
                        if command.is_cancelled() {
                            return Err(crate::CommandError::Failed("cancelled".to_owned()));
                        }
                        thread::sleep(Duration::from_millis(2));
                    }
                }
                self.emitter
                    .emit(7, command.payload().to_vec())
                    .map_err(|error| crate::CommandError::Failed(error.to_string()))?;
                Ok(format!("accepted:{}", command.id()).into_bytes())
            })
        }

        fn close(&mut self, reason: u32) -> CommandResult {
            Ok(format!("closed:{reason}").into_bytes())
        }
    }

    impl Utility for EchoService {
        fn new(context: crate::UtilityContext) -> ApplicationResult<Self> {
            *STARTUP_CONTEXT.lock().unwrap() = Some(context);
            Ok(Self)
        }

        fn invoke(&self, request: UtilityRequest) -> UtilityFuture<'_> {
            Box::pin(async move {
                let mut response = b"utility: ".to_vec();
                response.extend_from_slice(request.payload());
                Ok(response)
            })
        }

        fn create_session(
            &self,
            request: UtilitySessionRequest,
        ) -> ApplicationResult<Box<dyn UtilitySession>> {
            assert_ne!(request.id(), 0);
            assert_eq!(request.init(), b"init");
            Ok(Box::new(EchoSession {
                emitter: request.emitter(),
            }))
        }
    }

    struct TestApplication;

    impl Registrations for TestApplication {
        fn utility_services() -> &'static [UtilityServiceRegistration] {
            const SERVICES: &[UtilityServiceRegistration] =
                &[UtilityServiceRegistration::new::<EchoService>("echo")];
            SERVICES
        }
    }

    unsafe extern "C" fn capture_reply(
        _user_data: *mut c_void,
        status: Status,
        payload: ByteSlice,
    ) -> Status {
        let bytes = if payload.len == 0 {
            Vec::new()
        } else {
            // SAFETY: The utility ABI guarantees a valid callback-only byte slice.
            unsafe { slice::from_raw_parts(payload.data, payload.len) }.to_vec()
        };
        *REPLY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some((status, bytes));
        Status::Ok
    }

    unsafe extern "C" fn capture_session_event(
        _user_data: *mut c_void,
        session_id: u64,
        sequence: u64,
        kind: u32,
        payload: ByteSlice,
    ) -> Status {
        let bytes = if payload.len == 0 {
            Vec::new()
        } else {
            // SAFETY: The Utility session ABI guarantees a valid callback-only slice.
            unsafe { slice::from_raw_parts(payload.data, payload.len) }.to_vec()
        };
        SESSION_EVENTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((session_id, sequence, kind, bytes));
        Status::Ok
    }

    fn take_reply() -> Option<(Status, Vec<u8>)> {
        for _ in 0..200 {
            if let Some(reply) = REPLY
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
            {
                return Some(reply);
            }
            thread::sleep(Duration::from_millis(2));
        }
        None
    }

    #[test]
    fn registered_utility_replies_asynchronously() {
        let _test = TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *REPLY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        let mut service = 0;
        let name = b"echo";
        // SAFETY: Test buffers and output pointer are valid.
        assert_eq!(
            unsafe {
                start_utility::<TestApplication>(name.as_ptr(), name.len(), &raw mut service)
            },
            Status::Ok
        );
        let request = b"hello";
        // SAFETY: Test buffers and callback remain valid until reply.
        assert_eq!(
            unsafe {
                invoke_utility(
                    service,
                    7,
                    request.as_ptr(),
                    request.len(),
                    std::ptr::null_mut(),
                    Some(capture_reply),
                )
            },
            Status::Ok
        );
        for _ in 0..100 {
            if REPLY
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_some()
            {
                break;
            }
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(
            REPLY
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take(),
            Some((Status::Ok, b"utility: hello".to_vec()))
        );
        // SAFETY: The handle was returned by `start_utility`.
        assert_eq!(unsafe { stop_utility(service) }, Status::Ok);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn utility_session_abi_enforces_credit_ack_cancel_and_stale_handles() {
        let _test = TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *REPLY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        SESSION_EVENTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        let mut service = 0;
        let name = b"echo";
        // SAFETY: Test buffers and output pointers remain valid for each call.
        assert_eq!(
            unsafe {
                start_utility::<TestApplication>(name.as_ptr(), name.len(), &raw mut service)
            },
            Status::Ok
        );
        let mut session = 0;
        // SAFETY: Test callback state is static and init is copied before return.
        assert_eq!(
            unsafe {
                create_utility_session(
                    service,
                    42,
                    b"init".as_ptr(),
                    4,
                    1,
                    16,
                    std::ptr::null_mut(),
                    Some(capture_session_event),
                    &raw mut session,
                )
            },
            Status::Ok
        );
        assert_ne!(session, 0);

        // SAFETY: Test buffers and callback remain valid until the reply.
        assert_eq!(
            unsafe {
                send_utility_session(
                    session,
                    1,
                    b"one".as_ptr(),
                    3,
                    std::ptr::null_mut(),
                    Some(capture_reply),
                )
            },
            Status::Ok
        );
        assert_eq!(take_reply(), Some((Status::Ok, b"accepted:1".to_vec())));
        assert_eq!(
            SESSION_EVENTS.lock().unwrap().as_slice(),
            &[(42, 1, 7, b"one".to_vec())]
        );

        // The second event cannot pass until Rust Host acknowledges sequence 1.
        assert_eq!(
            unsafe {
                send_utility_session(
                    session,
                    2,
                    b"two".as_ptr(),
                    3,
                    std::ptr::null_mut(),
                    Some(capture_reply),
                )
            },
            Status::Ok
        );
        assert_eq!(
            take_reply().map(|value| value.0),
            Some(Status::InvalidState)
        );
        assert_eq!(acknowledge_utility_session(session, 1, 1), Status::Ok);
        assert_eq!(
            unsafe {
                send_utility_session(
                    session,
                    3,
                    b"two".as_ptr(),
                    3,
                    std::ptr::null_mut(),
                    Some(capture_reply),
                )
            },
            Status::Ok
        );
        assert_eq!(take_reply(), Some((Status::Ok, b"accepted:3".to_vec())));
        assert_eq!(SESSION_EVENTS.lock().unwrap()[1].1, 2);

        assert_eq!(
            unsafe {
                send_utility_session(
                    session,
                    4,
                    b"wait".as_ptr(),
                    4,
                    std::ptr::null_mut(),
                    Some(capture_reply),
                )
            },
            Status::Ok
        );
        assert_eq!(cancel_utility_session(session, 4), Status::Ok);
        assert_eq!(
            take_reply().map(|value| value.0),
            Some(Status::InvalidState)
        );

        assert_eq!(
            unsafe { close_utility_session(session, 9, std::ptr::null_mut(), Some(capture_reply)) },
            Status::Ok
        );
        assert_eq!(take_reply(), Some((Status::Ok, b"closed:9".to_vec())));
        assert_eq!(
            acknowledge_utility_session(session, 2, 1),
            Status::InvalidHandle
        );
        // SAFETY: Service remains live and is now free of owned sessions.
        assert_eq!(unsafe { stop_utility(service) }, Status::Ok);
    }

    #[test]
    fn registrations_are_validated() {
        assert!(validate_registrations::<TestApplication>().is_ok());
        assert!(!is_identifier(""));
        assert!(!is_identifier("1service"));
        assert!(is_identifier("$service_1"));
    }
}
