use std::ffi::c_void;
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::slice;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use tuffite_abi::{ByteSlice, Handle as FfiHandle, Host, RuntimeEventKind, Status};

use crate::Runtime;
use crate::browser::{Browser, BrowserContext, ErasedBrowser};
use crate::dispatch::DispatchScheduler;
use crate::executor::ChromiumExecutor;
use crate::handle::{Handle, HandleError, HandleKind, HandleTable, HandleType, RawHandle};
use crate::protocol;

enum RuntimeTag {}

impl HandleType for RuntimeTag {
    const KIND: HandleKind = HandleKind::Runtime;
}

/// Validated core Host snapshot, without any discovered capability tables.
#[derive(Clone, Copy, Debug)]
pub(crate) struct NativeHost(pub(crate) Host);

impl NativeHost {
    pub(crate) fn post_event(self, runtime: u64, kind: RuntimeEventKind, payload: &[u8]) -> Status {
        let Some(callback) = self.0.post_event else {
            return Status::InvalidState;
        };
        // SAFETY: Startup validated the native context and callback contract;
        // the borrowed payload is alive throughout this call.
        unsafe {
            callback(
                self.0.user_data,
                runtime,
                kind,
                ByteSlice::from_slice(payload),
            )
        }
    }

    /// Caller supplies live task state and callbacks, transferring ownership only
    /// when the native operation accepts the task.
    pub(crate) unsafe fn post_task(
        self,
        priority: tuffite_abi::TaskPriority,
        may_block: u8,
        task_data: *mut c_void,
        run: Option<tuffite_abi::TaskRunFn>,
        destroy: Option<tuffite_abi::TaskDestroyFn>,
    ) -> Status {
        let Some(callback) = self.0.post_task else {
            return Status::InvalidState;
        };
        // SAFETY: Forwarded task-state contract; Host context was validated at startup.
        unsafe {
            callback(
                self.0.user_data,
                priority,
                may_block,
                task_data,
                run,
                destroy,
            )
        }
    }
}

// SAFETY: Startup accepts Host's callback/context lifetime and thread-safety
// contract. This wrapper is used only for that validated native table.
unsafe impl Send for NativeHost {}
// SAFETY: Accepted Host callbacks permit concurrent calls through application stop.
unsafe impl Sync for NativeHost {}

/// Copies only our known prefix after size/version validation. Provider pointers
/// must be aligned, readable, immutable and valid through application stop.
unsafe fn read_capability<T: Copy>(table: *const tuffite_abi::Header) -> Result<T, Status> {
    if table.is_null() {
        return Err(Status::InvalidArgument);
    }
    // SAFETY: Capability providers guarantee at least a readable size prefix.
    let size = unsafe { ptr::read(table.cast::<usize>()) };
    if size < mem::size_of::<T>() {
        return Err(Status::IncompatibleAbi);
    }
    // SAFETY: Provider declares a readable complete known prefix.
    let header = unsafe { ptr::read(table) };
    if !header
        .version
        .is_compatible_with(tuffite_abi::Version::BASELINE)
    {
        return Err(Status::IncompatibleAbi);
    }
    if header.reserved != 0 {
        return Err(Status::InvalidArgument);
    }
    // SAFETY: Checked the known prefix size; unknown tail fields are not copied.
    Ok(unsafe { ptr::read(table.cast::<T>()) })
}

unsafe fn require_utility_interfaces(host: &Host) -> Result<(), Status> {
    let query = host.query_extension.ok_or(Status::IncompatibleAbi)?;
    // SAFETY: Host supplies a valid immutable capability header when supported.
    let table = unsafe {
        query(
            host.user_data,
            tuffite_abi::Capability::UtilityInterfaces as u32,
        )
    };
    if table.is_null() {
        return Err(Status::IncompatibleAbi);
    }
    // SAFETY: The capability query follows the host's startup pointer contract.
    unsafe { read_capability::<tuffite_abi::Header>(table) }.map(|_| ())
}

unsafe fn discover_capabilities(
    host: &Host,
) -> Result<
    (
        tuffite_abi::UtilitySessionClient,
        Option<tuffite_abi::ShellApi>,
    ),
    Status,
> {
    if crate::builder::requires_utility_interfaces() {
        // SAFETY: Host capability tables obey the startup lifetime contract.
        unsafe { require_utility_interfaces(host) }?;
    }
    let mut session = tuffite_abi::UtilitySessionClient::default();
    let mut shell_api = None;
    if let Some(query) = host.query_extension {
        // SAFETY: Host supplies a valid thread-safe query and lifetime contract.
        let table = unsafe { query(host.user_data, tuffite_abi::Capability::Browser as u32) };
        if !table.is_null() {
            // SAFETY: Query guarantees a readable independently headed prefix.
            let browser: tuffite_abi::BrowserHost = unsafe { read_capability(table) }?;
            if !browser
                .header
                .version
                .is_compatible_with(tuffite_abi::Version::BASELINE)
            {
                return Err(Status::IncompatibleAbi);
            }
            if !browser.utility.is_null() {
                // SAFETY: Browser supplies the borrowed-table lifetime contract.
                let utility: tuffite_abi::Utility =
                    unsafe { read_capability(browser.utility.cast()) }?;
                if !utility.session.is_null() {
                    // SAFETY: Utility supplies the same borrowed-table contract.
                    session = unsafe { read_capability(utility.session.cast()) }?;
                }
            }
            if !browser.shell_api.is_null() {
                // SAFETY: Browser supplies the same borrowed-table contract.
                shell_api = Some(unsafe { read_capability(browser.shell_api.cast()) }?);
            }
        }
    }
    Ok((session, shell_api))
}

fn abandon_browser(providers: &crate::utility::V8Providers, executor: &ChromiumExecutor) {
    providers.stop();
    std::thread::scope(|scope| {
        let _ = scope.spawn(|| providers.release()).join();
    });
    executor.close_and_wait();
}

struct RuntimeInstance {
    runtime: Runtime,
    providers: Arc<crate::utility::V8Providers>,
    app_scheme: Option<String>,
    callbacks: NativeHost,
    runtime_identity: Arc<AtomicU64>,
    scheduler: DispatchScheduler,
    executor: ChromiumExecutor,
    stop: crate::StopSignal,
    initialized: Arc<std::sync::atomic::AtomicBool>,
}

type RuntimeHandle = Handle<RuntimeTag>;
type RuntimeTable = HandleTable<RuntimeInstance, RuntimeTag>;

fn runtimes() -> &'static Mutex<RuntimeTable> {
    static RUNTIMES: OnceLock<Mutex<RuntimeTable>> = OnceLock::new();
    RUNTIMES.get_or_init(|| Mutex::new(RuntimeTable::new()))
}

fn ffi_guard(operation: impl FnOnce() -> Status) -> Status {
    catch_unwind(AssertUnwindSafe(operation)).unwrap_or(Status::Internal)
}

unsafe fn borrowed_bytes<'a>(data: *const u8, len: usize) -> Result<&'a [u8], Status> {
    if len == 0 {
        return Ok(&[]);
    }
    if data.is_null() || len > isize::MAX as usize {
        return Err(Status::InvalidArgument);
    }
    // SAFETY: The caller guarantees that `data` is readable for `len` bytes for
    // the duration of this ABI call. Null and oversized slices were rejected.
    Ok(unsafe { slice::from_raw_parts(data, len) })
}

fn parse_runtime_handle(value: FfiHandle) -> Result<RuntimeHandle, Status> {
    let raw = RawHandle::from_u64(value).map_err(map_handle_error)?;
    Handle::from_raw(raw).map_err(map_handle_error)
}

const fn map_handle_error(_error: HandleError) -> Status {
    Status::InvalidHandle
}

fn post_event(
    callbacks: NativeHost,
    runtime: RuntimeHandle,
    kind: RuntimeEventKind,
    payload: &[u8],
) -> Status {
    callbacks.post_event(runtime.raw().as_u64(), kind, payload)
}

/// Starts one Runtime and returns a generation-aware opaque handle.
///
/// # Safety
///
/// `host` and `runtime_out` must be valid for reads/writes of their types.
/// `config` must be readable for `config_len` bytes. Callback state must remain
/// valid and thread-safe until stop returns.
pub unsafe extern "C" fn start_browser<B: Browser>(
    host: *const Host,
    config: *const u8,
    config_len: usize,
    runtime_out: *mut FfiHandle,
) -> Status {
    ffi_guard(|| {
        if host.is_null() || runtime_out.is_null() {
            return Status::InvalidArgument;
        }
        // SAFETY: `runtime_out` is writable by the function's ABI contract. A
        // failure never leaves a nonzero, unusable handle behind.
        unsafe { runtime_out.write(0) };
        // SAFETY: The pointer is aligned and readable for at least its leading
        // `struct_size` field by the function's ABI contract.
        let struct_size = unsafe { ptr::read(host.cast::<usize>()) };
        if struct_size < mem::size_of::<Host>() {
            return Status::IncompatibleAbi;
        }
        // SAFETY: The size check now proves the caller declared storage for the
        // complete host table. Copying prevents later table mutation.
        let host = unsafe { ptr::read(host) };
        if !host
            .header
            .version
            .is_compatible_with(tuffite_abi::Version::BASELINE)
        {
            return Status::IncompatibleAbi;
        }
        if host.header.reserved != 0 {
            return Status::InvalidArgument;
        }
        if host.post_event.is_none() || host.post_task.is_none() {
            return Status::InvalidArgument;
        }
        // SAFETY: Forwarded from this function's pointer contract.
        let config = match unsafe { borrowed_bytes(config, config_len) } {
            Ok(config) => config.to_vec(),
            Err(status) => return status,
        };
        // SAFETY: Host and its capability tables satisfy the startup contract.
        let (session, shell_api) = match unsafe { discover_capabilities(&host) } {
            Ok(capabilities) => capabilities,
            Err(status) => return status,
        };
        let callbacks = NativeHost(host);
        let runtime_identity = Arc::new(AtomicU64::new(0));
        let app_scheme = crate::utility::v8::app_scheme(&config);
        let context = BrowserContext::new(
            config,
            Arc::clone(&runtime_identity),
            callbacks,
            session,
            shell_api,
        );
        let executor = context.executor();
        let providers = context.providers();
        let stop = context.stop_signal();
        let initialized = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let browser = match catch_unwind(AssertUnwindSafe(|| B::new(context))) {
            Ok(Ok(browser)) => Box::new(browser) as Box<dyn ErasedBrowser>,
            result => {
                stop.0.cancel();
                abandon_browser(&providers, &executor);
                if let Ok(Err(error)) = result {
                    tracing::error!(error = %error, "Tuffite browser construction failed");
                    return Status::InvalidState;
                }
                return Status::Internal;
            }
        };
        // Start the scheduler with an atomic runtime handle initialized to 0.
        // The worker will only post_event once the handle has been published,
        // which happens after we insert the instance into the table below.
        let scheduler =
            DispatchScheduler::start(&runtime_identity, browser, callbacks, stop.clone());
        let instance = RuntimeInstance {
            runtime: Runtime::new(),
            providers,
            app_scheme,
            callbacks,
            runtime_identity: Arc::clone(&runtime_identity),
            scheduler,
            executor,
            stop,
            initialized,
        };
        let Ok(handle) = runtimes()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(instance)
        else {
            return Status::ResourceExhausted;
        };
        // Publish the handle so the scheduler worker can post responses. All
        // enqueue calls have to happen after this point.
        runtime_identity.store(handle.raw().as_u64(), Ordering::Release);
        let status = post_event(callbacks, handle, RuntimeEventKind::Started, &[]);
        if status != Status::Ok {
            // Extract under the handle-table lock; never join workers while
            // holding it, because callbacks may re-enter runtime entry points.
            let removed = runtimes()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(handle);
            if let Ok(instance) = removed {
                let _ = stop_instance(instance, handle);
            }
            return status;
        }
        if runtimes()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(handle)
            .is_err()
        {
            return Status::InvalidState;
        }
        // SAFETY: `runtime_out` is writable by the function's ABI contract.
        unsafe { runtime_out.write(handle.raw().as_u64()) };
        Status::Ok
    })
}

/// Schedules initialization after native `BrowserContext` services are bound.
/// # Safety
/// Accepted callback state stays valid and thread-safe until exactly one reply.
pub unsafe extern "C" fn initialize_browser(
    runtime: FfiHandle,
    data: *mut c_void,
    reply: Option<tuffite_abi::UtilityReplyFn>,
) -> Status {
    ffi_guard(|| {
        let Some(reply) = reply else {
            return Status::InvalidArgument;
        };
        let handle = match parse_runtime_handle(runtime) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        let mut table = runtimes()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Ok(instance) = table.get_mut(handle) else {
            return Status::InvalidHandle;
        };
        let callback = ModuleReply { data, reply };
        let initialized = instance.initialized.clone();
        match instance.scheduler.initialize(Box::new(move |result| {
            initialized.store(result.is_ok(), Ordering::Release);
            match result {
                Ok(()) => callback.deliver(Status::Ok, &[]),
                Err(error) => {
                    tracing::error!(%error, "Browser initialization failed");
                    callback.deliver(Status::InvalidState, &[]);
                }
            }
        })) {
            Ok(()) => Status::Ok,
            Err(status) => status,
        }
    })
}

/// Enqueues one binary control event for Runtime to dispatch asynchronously.
///
/// This function never runs the application handler on the caller's thread.
/// It performs O(1) work (header validation + queue push) and returns before
/// the handler executes so Chromium's UI sequence stays responsive.
///
/// The corresponding response is delivered via
/// [`RuntimeEventKind::CommandResponse`] with the wire header preserving
/// the caller-supplied `request_id`. Framework correlates completions by
/// reading that field.
///
/// # Safety
///
/// `event` must be readable for `event_len` bytes for the duration of the
/// call. The bytes are copied into the queue before the function returns.
#[must_use]
pub unsafe extern "C" fn dispatch_browser(
    runtime: FfiHandle,
    event: *const u8,
    event_len: usize,
) -> Status {
    ffi_guard(|| {
        let handle = match parse_runtime_handle(runtime) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        // SAFETY: Forwarded from this function's pointer contract.
        let event = match unsafe { borrowed_bytes(event, event_len) } {
            Ok(event) => event,
            Err(status) => return status,
        };
        // Extract the request_id from the wire header. Malformed frames are
        // rejected here without touching Runtime state so the queue is not
        // polluted with impossible work.
        let Some(request_id) = protocol::request_request_id(event) else {
            return Status::InvalidArgument;
        };
        let (callbacks, enqueue_result) = {
            let table = runtimes()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let instance = match table.get(handle) {
                Ok(instance) => instance,
                Err(error) => return map_handle_error(error),
            };
            if instance.runtime.state() != crate::RuntimeState::Running {
                return Status::InvalidState;
            }
            let enqueue = instance.scheduler.enqueue(request_id, event.to_vec());
            (instance.callbacks, enqueue)
        };
        match enqueue_result {
            Ok(()) => {
                // Advertise queue acceptance so Framework knows it may await a
                // matching CommandResponse. We deliberately do not block on
                // handler execution.
                let _ = post_event(
                    callbacks,
                    handle,
                    RuntimeEventKind::DispatchAccepted,
                    &request_id.to_le_bytes(),
                );
                Status::Ok
            }
            Err(status) => status,
        }
    })
}

struct ModuleReply {
    data: *mut c_void,
    reply: tuffite_abi::UtilityReplyFn,
}

// SAFETY: Accepted resolver callbacks supply a thread-safe context through runtime stop.
unsafe impl Send for ModuleReply {}

impl ModuleReply {
    fn deliver(self, status: Status, bytes: &[u8]) {
        // SAFETY: The accepted callback context remains valid until this one reply.
        unsafe {
            (self.reply)(self.data, status, tuffite_abi::ByteSlice::from_slice(bytes));
        }
    }
}

/// Cancels module requests admitted for a closing native session.
/// # Safety
/// Name bytes are readable for the duration of the call.
#[must_use]
pub unsafe extern "C" fn cancel_v8_session(
    runtime: FfiHandle,
    name: ByteSlice,
    session: u64,
) -> Status {
    ffi_guard(|| {
        if name.len > 128 || session == 0 {
            return Status::InvalidArgument;
        }
        let Ok(name) = (unsafe { borrowed_bytes(name.data, name.len) })
            .and_then(|value| std::str::from_utf8(value).map_err(|_| Status::InvalidArgument))
        else {
            return Status::InvalidArgument;
        };
        let Ok(handle) = parse_runtime_handle(runtime) else {
            return Status::InvalidHandle;
        };
        let table = runtimes()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Ok(instance) = table.get(handle) else {
            return Status::InvalidHandle;
        };
        let providers = instance.providers.clone();
        drop(table);
        providers.cancel_session(name, session);
        Status::Ok
    })
}

/// Resolves a V8 source on a supervised browser worker, joined during stop.
///
/// # Safety
/// Input slices are readable for the call. On acceptance reply/context remain
/// valid and thread-safe until one reply; rejection does not invoke the callback.
pub unsafe extern "C" fn resolve_v8(
    runtime: FfiHandle,
    session_id: u64,
    name: tuffite_abi::ByteSlice,
    url: tuffite_abi::ByteSlice,
    data: *mut c_void,
    reply: Option<tuffite_abi::UtilityReplyFn>,
) -> Status {
    ffi_guard(|| {
        let Some(reply) = reply else {
            return Status::InvalidArgument;
        };
        if name.len > 128 || url.len > 512 {
            return Status::InvalidArgument;
        }
        // SAFETY: Input slices follow the caller's borrowed buffer contract.
        let Ok(name) = (unsafe { borrowed_bytes(name.data, name.len) })
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| Status::InvalidArgument))
        else {
            return Status::InvalidArgument;
        };
        // SAFETY: Same borrowed buffer contract for the module URL.
        let Ok(url) = (unsafe { borrowed_bytes(url.data, url.len) })
            .and_then(|bytes| std::str::from_utf8(bytes).map_err(|_| Status::InvalidArgument))
        else {
            return Status::InvalidArgument;
        };
        let Some(definition) = crate::builder::utility_definition(name) else {
            return Status::InvalidArgument;
        };
        if definition.runtime() != 1 || crate::utility::v8::validate_url(name, url, None).is_err() {
            return Status::InvalidArgument;
        }
        let Ok(handle) = parse_runtime_handle(runtime) else {
            return Status::InvalidHandle;
        };
        let table = runtimes()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Ok(instance) = table.get(handle) else {
            return Status::InvalidHandle;
        };
        if crate::utility::v8::validate_url(name, url, instance.app_scheme.as_deref()).is_err() {
            return Status::InvalidArgument;
        }
        if session_id == 0 {
            return Status::InvalidArgument;
        }
        let callback = ModuleReply { data, reply };
        let signal = instance.providers.session_signal(name, session_id);
        let request =
            crate::utility::v8::ModuleRequest::new(url.to_owned(), session_id, signal.clone());
        let name = name.to_owned();
        let providers = instance.providers.clone();
        match instance.runtime.spawn("v8-resolve", move |context| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                let provider = providers.get(&name)?;
                crate::lifecycle::run_until(
                    crate::utility::resolve_source(provider.as_ref(), &name, request.clone()),
                    &signal.0,
                    request.deadline(),
                )
            }));
            let (status, bytes) = if context.is_cancelled() || signal.is_stopping() {
                (Status::InvalidState, Vec::new())
            } else {
                match result {
                    Ok(Ok(Some(bytes))) => (Status::Ok, bytes),
                    Ok(Ok(None)) => (Status::NotFound, Vec::new()),
                    Ok(Err(error)) => {
                        tracing::error!(%error, "V8 module resolution failed");
                        (Status::InvalidArgument, Vec::new())
                    }
                    Err(_) => (Status::Internal, Vec::new()),
                }
            };
            callback.deliver(status, &bytes);
        }) {
            Ok(_) => Status::Ok,
            Err(_) => Status::InvalidState,
        }
    })
}

/// Stops Runtime, joins its work, and invalidates its handle.
#[must_use]
pub extern "C" fn stop_browser(runtime: FfiHandle) -> Status {
    ffi_guard(|| {
        let handle = match parse_runtime_handle(runtime) {
            Ok(handle) => handle,
            Err(status) => return status,
        };
        let instance = match runtimes()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(handle)
        {
            Ok(instance) => instance,
            Err(error) => return map_handle_error(error),
        };
        stop_instance(instance, handle)
    })
}

fn stop_instance(instance: RuntimeInstance, handle: RuntimeHandle) -> Status {
    // Stop is a synchronous ABI boundary after the native main loop exits, but
    // application futures and destructors still belong on a Rust worker.
    match std::thread::Builder::new()
        .name("tuffite-shutdown".into())
        .spawn(move || finish_stop(instance, handle))
    {
        Ok(worker) => worker.join().unwrap_or(Status::Internal),
        Err(_) => Status::Internal,
    }
}

fn finish_stop(mut instance: RuntimeInstance, handle: RuntimeHandle) -> Status {
    // Publish lifecycle control before waiting for application work. The
    // Framework can immediately reject new/pending requests even when a
    // business handler is slow. Clearing the worker-visible identity then
    // prevents late CommandResponse events from crossing the stopping
    // boundary while shutdown joins the worker.
    let _ = post_event(instance.callbacks, handle, RuntimeEventKind::Stopping, &[]);
    instance.stop.0.cancel();
    instance.providers.stop();
    instance.runtime_identity.store(0, Ordering::Release);
    let application = instance.scheduler.shutdown();
    // Resolver futures observe stop before being joined. Provider teardown
    // and Browser cleanup retain the Chromium executor until both finish.
    let runtime_status = instance.runtime.stop();
    let provider_status = instance.providers.release();
    let shutdown_status = match application {
        Some(mut application) => {
            let reason = if instance.initialized.load(Ordering::Acquire) {
                crate::ShutdownReason::Requested
            } else {
                crate::ShutdownReason::InitializationFailed
            };
            let context = crate::ShutdownContext::new(
                reason,
                std::time::Instant::now() + crate::lifecycle::LIFECYCLE_TIMEOUT,
            );
            match catch_unwind(AssertUnwindSafe(|| {
                context.run(application.shutdown(context.clone()))
            })) {
                Ok(Ok(())) => Status::Ok,
                Ok(Err(error)) => {
                    tracing::error!(%error, "Browser cleanup failed");
                    Status::Internal
                }
                Err(_) => Status::Internal,
            }
        }
        None => Status::Internal,
    };
    instance.executor.close_and_wait();
    match (shutdown_status, provider_status, runtime_status) {
        (Status::Ok, Ok(()), Ok(())) => Status::Ok,
        (status, _, _) if status != Status::Ok => status,
        (_, _, Err(_)) => Status::InvalidState,
        _ => Status::Internal,
    }
}

/// Trusted browser setup adapter. The sink copies each borrowed string.
///
/// # Safety
/// Sink context and callback must remain valid throughout this synchronous call.
pub(crate) unsafe extern "C" fn setup_browser<B: Browser>(
    user_data: *mut c_void,
    emit: Option<tuffite_abi::LaunchArgumentFn>,
) -> Status {
    ffi_guard(|| {
        let Some(emit) = emit else {
            return Status::InvalidArgument;
        };
        let setup = match B::setup(crate::browser::Builder::default()) {
            Ok(setup) => setup,
            Err(error) => {
                tracing::error!(%error, "Tuffite browser setup failed");
                return Status::InvalidState;
            }
        };
        if setup.devtools_denied_origins.len() > 256 {
            return Status::ResourceExhausted;
        }
        for origin in setup.devtools_denied_origins {
            if origin.is_empty() || origin.len() > 4096 || origin.contains('\0') {
                return Status::InvalidArgument;
            }
            // SAFETY: Forwarded sink contract; the origin lives through the call.
            let status = unsafe { emit(user_data, ByteSlice::from_slice(origin.as_bytes())) };
            if status != Status::Ok {
                return status;
            }
        }
        Status::Ok
    })
}

#[cfg(test)]
mod tests {
    use std::ffi::c_void;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    use tuffite_abi::RuntimeEventKind;

    use super::*;
    use crate::browser::Command;

    static EVENTS: Mutex<Vec<(FfiHandle, RuntimeEventKind, Vec<u8>)>> = Mutex::new(Vec::new());
    static EVENT_READY: std::sync::Condvar = std::sync::Condvar::new();
    static SHUTDOWN_CALLED: AtomicBool = AtomicBool::new(false);

    #[test]
    fn owned_browser_setup_emits_composed_denials_without_constructing_state() {
        struct Configured;
        impl Browser for Configured {
            fn setup(
                builder: crate::browser::Builder,
            ) -> crate::ApplicationResult<crate::browser::Builder> {
                fn common(builder: crate::browser::Builder) -> crate::browser::Builder {
                    builder.disable_devtools_for_origin("myapp://main")
                }
                Ok(common(builder).disable_devtools_for_origin("https://private.example.com"))
            }
            fn new(_: BrowserContext) -> crate::ApplicationResult<Self> {
                panic!("setup must not construct Browser state")
            }
        }
        unsafe extern "C" fn collect(data: *mut c_void, origin: ByteSlice) -> Status {
            // SAFETY: This test supplies a live Vec and setup borrows valid UTF-8 until return.
            let values = unsafe { &mut *data.cast::<Vec<String>>() };
            let bytes = unsafe { std::slice::from_raw_parts(origin.data, origin.len) };
            values.push(std::str::from_utf8(bytes).unwrap().to_owned());
            Status::Ok
        }
        let mut origins = Vec::<String>::new();
        // SAFETY: The vector and synchronous copying sink live throughout setup.
        assert_eq!(
            unsafe { setup_browser::<Configured>((&raw mut origins).cast(), Some(collect)) },
            Status::Ok
        );
        assert_eq!(origins, ["myapp://main", "https://private.example.com"]);
    }

    #[test]
    fn browser_setup_error_does_not_publish_configuration() {
        struct Rejected;
        impl Browser for Rejected {
            fn setup(
                builder: crate::browser::Builder,
            ) -> crate::ApplicationResult<crate::browser::Builder> {
                let _builder = builder.disable_devtools_for_origin("myapp://main");
                Err(crate::ApplicationError::new("setup rejected"))
            }
            fn new(_: BrowserContext) -> crate::ApplicationResult<Self> {
                unreachable!()
            }
        }
        static EMITTED: AtomicBool = AtomicBool::new(false);
        unsafe extern "C" fn reject_emit(_: *mut c_void, _: ByteSlice) -> Status {
            EMITTED.store(true, Ordering::SeqCst);
            Status::Internal
        }
        // SAFETY: This rejected setup never dereferences the null sink context.
        assert_eq!(
            unsafe { setup_browser::<Rejected>(std::ptr::null_mut(), Some(reject_emit)) },
            Status::InvalidState
        );
        assert!(!EMITTED.load(Ordering::SeqCst));
    }

    struct TestBrowser {
        configuration: Vec<u8>,
        dispatch_count: u8,
    }

    impl Browser for TestBrowser {
        fn new(context: BrowserContext) -> crate::ApplicationResult<Self> {
            Ok(Self {
                configuration: context.configuration().to_vec(),
                dispatch_count: 0,
            })
        }

        fn dispatch(&mut self, command: Command<'_>) -> crate::CommandResult {
            self.dispatch_count += 1;
            let mut response = self.configuration.clone();
            response.push(self.dispatch_count);
            response.extend_from_slice(command.payload());
            Ok(response)
        }

        async fn shutdown(
            &mut self,
            _context: crate::ShutdownContext,
        ) -> crate::ApplicationResult<()> {
            SHUTDOWN_CALLED.store(true, Ordering::Release);
            Ok(())
        }
    }

    unsafe extern "C" fn record_event(
        _user_data: *mut c_void,
        runtime: FfiHandle,
        kind: RuntimeEventKind,
        payload: ByteSlice,
    ) -> Status {
        // SAFETY: Runtime supplies a valid callback-only byte slice.
        let bytes = if payload.len == 0 {
            Vec::new()
        } else {
            unsafe { slice::from_raw_parts(payload.data, payload.len) }.to_vec()
        };
        EVENTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((runtime, kind, bytes));
        EVENT_READY.notify_all();
        Status::Ok
    }

    unsafe extern "C" fn run_task(
        _user_data: *mut c_void,
        _priority: tuffite_abi::TaskPriority,
        _may_block: u8,
        task_data: *mut c_void,
        run: Option<tuffite_abi::TaskRunFn>,
        destroy: Option<tuffite_abi::TaskDestroyFn>,
    ) -> Status {
        let (Some(run), Some(destroy)) = (run, destroy) else {
            return Status::InvalidArgument;
        };
        // SAFETY: The executor supplies a live opaque task allocation.
        unsafe {
            run(task_data);
            destroy(task_data);
        }
        Status::Ok
    }

    fn host() -> Host {
        Host {
            header: tuffite_abi::Header::for_type::<Host>(),
            user_data: ptr::null_mut(),
            post_event: Some(record_event),
            post_task: Some(run_task),
            query_extension: None,
        }
    }

    unsafe extern "C" fn quiet(
        _: *mut c_void,
        _: FfiHandle,
        _: RuntimeEventKind,
        _: ByteSlice,
    ) -> Status {
        Status::Ok
    }

    unsafe extern "C" fn initialized(data: *mut c_void, status: Status, _: ByteSlice) -> Status {
        // SAFETY: Acceptance transfers this owned sender to exactly one reply.
        let tx = unsafe { Box::from_raw(data.cast::<std::sync::mpsc::Sender<Status>>()) };
        tx.send(status).unwrap();
        Status::Ok
    }

    fn initialize_and_wait(runtime: FfiHandle) {
        let (tx, rx) = std::sync::mpsc::channel::<Status>();
        assert_eq!(
            unsafe {
                initialize_browser(
                    runtime,
                    Box::into_raw(Box::new(tx)).cast(),
                    Some(initialized),
                )
            },
            Status::Ok
        );
        assert_eq!(
            rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            Status::Ok
        );
    }

    #[test]
    fn failed_initialization_rejects_dispatch_and_cleanup_keeps_executor_alive() {
        static CLEANED: AtomicBool = AtomicBool::new(false);
        struct FailingBrowser {
            context: BrowserContext,
            created_on: std::thread::ThreadId,
        }
        impl Browser for FailingBrowser {
            fn new(context: BrowserContext) -> crate::ApplicationResult<Self> {
                Ok(Self {
                    context,
                    created_on: std::thread::current().id(),
                })
            }
            async fn initialize(&mut self) -> crate::ApplicationResult<()> {
                // Exercise a future that yields, rather than a synchronous error.
                let mut yielded = false;
                std::future::poll_fn(|cx| {
                    if yielded {
                        std::task::Poll::Ready(())
                    } else {
                        yielded = true;
                        cx.waker().wake_by_ref();
                        std::task::Poll::Pending
                    }
                })
                .await;
                Err(crate::ApplicationError::new("fixture startup failure"))
            }
            async fn shutdown(
                &mut self,
                shutdown: crate::ShutdownContext,
            ) -> crate::ApplicationResult<()> {
                assert_ne!(std::thread::current().id(), self.created_on);
                assert_eq!(
                    shutdown.reason(),
                    crate::ShutdownReason::InitializationFailed
                );
                assert!(self.context.stop_signal().is_stopping());
                assert!(shutdown.remaining() > std::time::Duration::ZERO);
                self.context
                    .executor()
                    .post(crate::TaskPriority::UserVisible, false, || {
                        CLEANED.store(true, Ordering::Release);
                    })
                    .unwrap();
                Ok(())
            }
        }
        let mut native = host();

        native.post_event = Some(quiet);
        let mut runtime = 0;
        // SAFETY: Host and writable output remain live through runtime stop.
        assert_eq!(
            unsafe {
                start_browser::<FailingBrowser>(&raw const native, ptr::null(), 0, &raw mut runtime)
            },
            Status::Ok
        );
        let event = request(99, 1, &[]);
        assert_eq!(
            unsafe { dispatch_browser(runtime, event.as_ptr(), event.len()) },
            Status::InvalidState
        );
        let (tx, rx) = std::sync::mpsc::channel::<Status>();

        assert_eq!(
            unsafe {
                initialize_browser(
                    runtime,
                    Box::into_raw(Box::new(tx)).cast(),
                    Some(initialized),
                )
            },
            Status::Ok
        );
        assert_eq!(
            rx.recv_timeout(std::time::Duration::from_secs(5)).unwrap(),
            Status::InvalidState
        );
        assert_eq!(
            unsafe { initialize_browser(runtime, ptr::null_mut(), Some(initialized)) },
            Status::InvalidState
        );
        assert_eq!(
            unsafe { dispatch_browser(runtime, event.as_ptr(), event.len()) },
            Status::InvalidState
        );
        assert_eq!(stop_browser(runtime), Status::Ok);
        assert!(CLEANED.load(Ordering::Acquire));
    }

    #[test]
    fn capability_validation_accepts_newer_prefix_and_rejects_malformed_tables() {
        #[repr(C)]
        struct Extended {
            table: tuffite_abi::ShellApi,
            optional_tail: usize,
        }
        let mut provider = Extended {
            table: tuffite_abi::ShellApi {
                header: tuffite_abi::Header::for_type::<Extended>(),
                user_data: ptr::null_mut(),
                invoke: None,
            },
            optional_tail: 42,
        };
        let table = &mut provider.table;
        table.header.version.minor += 1;
        // SAFETY: The known prefix is readable; unknown tail bytes are never read.
        assert!(
            unsafe { read_capability::<tuffite_abi::ShellApi>(ptr::from_ref(&table.header)) }
                .is_ok()
        );
        table.header.version.major += 1;
        // SAFETY: Test owns the complete aligned table.
        assert_eq!(
            unsafe { read_capability::<tuffite_abi::ShellApi>(ptr::from_ref(&table.header)) }
                .unwrap_err(),
            Status::IncompatibleAbi
        );
        table.header.version = tuffite_abi::Version::CURRENT;
        table.header.reserved = 1;
        // SAFETY: Test owns the complete aligned table.
        assert_eq!(
            unsafe { read_capability::<tuffite_abi::ShellApi>(ptr::from_ref(&table.header)) }
                .unwrap_err(),
            Status::InvalidArgument
        );
        let size_prefix = mem::size_of::<usize>();
        // SAFETY: The readable size prefix declares no further fields. Validation
        // must reject before reading a Header or capability body.
        assert_eq!(
            unsafe { read_capability::<tuffite_abi::ShellApi>(ptr::from_ref(&size_prefix).cast()) }
                .unwrap_err(),
            Status::IncompatibleAbi
        );
    }

    #[test]
    fn grouped_utilities_require_host_routing_support() {
        unsafe extern "C" fn unsupported(_: *mut c_void, _: u32) -> *const tuffite_abi::Header {
            ptr::null()
        }
        unsafe extern "C" fn supported(
            context: *mut c_void,
            id: u32,
        ) -> *const tuffite_abi::Header {
            if id == tuffite_abi::Capability::UtilityInterfaces as u32 {
                context.cast()
            } else {
                ptr::null()
            }
        }
        let mut host = host();
        // SAFETY: No query callback, so provider storage is not accessed.
        assert_eq!(
            unsafe { require_utility_interfaces(&host) },
            Err(Status::IncompatibleAbi)
        );

        host.query_extension = Some(unsupported);
        // SAFETY: Query returns null without accessing provider storage.
        assert_eq!(
            unsafe { require_utility_interfaces(&host) },
            Err(Status::IncompatibleAbi)
        );

        let mut header = tuffite_abi::Header::for_type::<tuffite_abi::Header>();
        host.user_data = ptr::from_mut(&mut header).cast();
        host.query_extension = Some(supported);
        // SAFETY: Header lives throughout the query and validation.
        assert_eq!(unsafe { require_utility_interfaces(&host) }, Ok(()));
        header.struct_size = 0;
        // SAFETY: Header storage remains readable but declares an invalid size.
        assert_eq!(
            unsafe { require_utility_interfaces(&host) },
            Err(Status::IncompatibleAbi)
        );
    }

    #[test]
    fn capability_discovery_handles_absence_nested_tables_and_independent_contexts() {
        struct Tables {
            browser: tuffite_abi::BrowserHost,
            utility: tuffite_abi::Utility,
            shell_api: tuffite_abi::ShellApi,
        }
        unsafe extern "C" fn query(context: *mut c_void, id: u32) -> *const tuffite_abi::Header {
            // SAFETY: The test retains the provider tables throughout discovery.
            let tables = unsafe { &*context.cast::<Tables>() };
            if id == tuffite_abi::Capability::Browser as u32 {
                ptr::from_ref(&tables.browser.header)
            } else {
                panic!("runtime must only query Browser capabilities")
            }
        }
        unsafe extern "C" fn unsupported(_: *mut c_void, _: u32) -> *const tuffite_abi::Header {
            ptr::null()
        }
        let mut host = host();
        // SAFETY: No capability callback; no provider memory is accessed.
        let (client, shell_api) = unsafe { discover_capabilities(&host) }.unwrap();
        assert!(client.create.is_none());
        assert!(shell_api.is_none());
        host.query_extension = Some(unsupported);
        // SAFETY: Query returns null without accessing caller memory.
        let (client, shell_api) = unsafe { discover_capabilities(&host) }.unwrap();
        assert!(client.send.is_none());
        assert!(shell_api.is_none());
        let mut client_context = 0_u8;
        let mut shell_api_context = 0_u8;
        let mut client = tuffite_abi::UtilitySessionClient {
            user_data: ptr::from_mut(&mut client_context).cast(),
            ..Default::default()
        };
        let mut tables = Tables {
            browser: tuffite_abi::BrowserHost {
                header: tuffite_abi::Header::for_type::<tuffite_abi::BrowserHost>(),
                shell_api: ptr::null(),
                utility: ptr::null(),
            },
            utility: tuffite_abi::Utility {
                header: tuffite_abi::Header::for_type::<tuffite_abi::Utility>(),
                session: ptr::from_ref(&client),
            },
            shell_api: tuffite_abi::ShellApi {
                header: tuffite_abi::Header::for_type::<tuffite_abi::ShellApi>(),
                user_data: ptr::from_mut(&mut shell_api_context).cast(),
                invoke: None,
            },
        };
        tables.browser.utility = ptr::from_ref(&tables.utility);
        tables.browser.shell_api = ptr::from_ref(&tables.shell_api);
        host.user_data = ptr::from_mut(&mut tables).cast();
        host.query_extension = Some(query);
        // SAFETY: Provider, nested table and both contexts remain alive.
        let (snapshot, shell_api) = unsafe { discover_capabilities(&host) }.unwrap();
        assert_eq!(snapshot.user_data, client.user_data);
        assert_eq!(shell_api.unwrap().user_data, tables.shell_api.user_data);
        tables.browser.header.version.major = tuffite_abi::Version::BASELINE.major.wrapping_add(1);
        // SAFETY: Valid table memory advertises an incompatible major version.
        assert!(matches!(
            unsafe { discover_capabilities(&host) },
            Err(Status::IncompatibleAbi)
        ));
        tables.browser.header.version = tuffite_abi::Version::CURRENT;
        tables.browser.header.struct_size = mem::size_of::<usize>();
        // SAFETY: Readable size prefix is intentionally too short.
        assert!(matches!(
            unsafe { discover_capabilities(&host) },
            Err(Status::IncompatibleAbi)
        ));
        tables.browser.header = tuffite_abi::Header::for_type::<tuffite_abi::BrowserHost>();
        tables.browser.shell_api = ptr::null();
        tables.browser.utility = ptr::null();
        // SAFETY: Both independently optional child tables are absent.
        let (snapshot, shell_api) = unsafe { discover_capabilities(&host) }.unwrap();
        assert!(snapshot.create.is_none());
        assert!(shell_api.is_none());
        tables.browser.shell_api = ptr::from_ref(&tables.shell_api);
        tables.browser.utility = ptr::from_ref(&tables.utility);
        tables.utility.session = ptr::null();
        // SAFETY: Missing optional session is explicitly supported.
        assert!(
            unsafe { discover_capabilities(&host) }
                .unwrap()
                .0
                .create
                .is_none()
        );
        client.header.struct_size = mem::size_of::<usize>();
        tables.utility.session = ptr::from_ref(&client);
        // SAFETY: Provider tables remain live; nested size is intentionally invalid.
        assert!(matches!(
            unsafe { discover_capabilities(&host) },
            Err(Status::IncompatibleAbi)
        ));
    }

    fn request(request_id: u64, command_id: u64, payload: &[u8]) -> Vec<u8> {
        let mut bytes = b"CERM".to_vec();
        bytes.extend_from_slice(&1_u16.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&request_id.to_le_bytes());
        bytes.extend_from_slice(&command_id.to_le_bytes());
        bytes.extend_from_slice(&0_u64.to_le_bytes());
        bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn lifecycle_uses_generation_aware_runtime_handles() {
        EVENTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        SHUTDOWN_CALLED.store(false, Ordering::Release);
        let host = host();
        let mut runtime = 0;
        // SAFETY: Test pointers remain valid through each call.
        assert_eq!(
            unsafe {
                start_browser::<TestBrowser>(
                    &raw const host,
                    b"config:".as_ptr(),
                    b"config:".len(),
                    &raw mut runtime,
                )
            },
            Status::Ok
        );
        assert_ne!(runtime, 0);
        initialize_and_wait(runtime);
        let first = request(7, 42, b"binary-event");
        // SAFETY: Payload is readable for its declared length.
        assert_eq!(
            unsafe { dispatch_browser(runtime, first.as_ptr(), first.len()) },
            Status::Ok
        );
        let second = request(8, 42, b"again");
        // SAFETY: Payload is readable for its declared length.
        assert_eq!(
            unsafe { dispatch_browser(runtime, second.as_ptr(), second.len()) },
            Status::Ok
        );
        // Wait for both successful replies before testing lifecycle shutdown.
        // Queued work at shutdown is now cancelled instead of being executed.
        let (events, timeout) = EVENT_READY
            .wait_timeout_while(
                EVENTS.lock().unwrap(),
                std::time::Duration::from_secs(5),
                |events| {
                    events
                        .iter()
                        .filter(|(_, kind, _)| *kind == RuntimeEventKind::CommandResponse)
                        .count()
                        < 2
                },
            )
            .unwrap();
        assert!(!timeout.timed_out(), "dispatch responses did not arrive");
        drop(events);
        assert_eq!(stop_browser(runtime), Status::Ok);
        assert!(SHUTDOWN_CALLED.load(Ordering::Acquire));
        assert_eq!(stop_browser(runtime), Status::InvalidHandle);

        let events = EVENTS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // Expected sequence contains Started, two DispatchAccepted, two
        // CommandResponse and Stopping. The two Accepted/Response pairs may
        // interleave with each other but must all appear before Stopping.
        assert!(
            events.len() >= 6,
            "events must include Started, 2x Accepted, 2x Response and Stopping: {events:?}",
        );
        assert_eq!(events.first().map(|e| e.1), Some(RuntimeEventKind::Started));
        let stopping_index = events
            .iter()
            .position(|event| event.1 == RuntimeEventKind::Stopping)
            .expect("Stopping event must be delivered");
        assert!(
            events[stopping_index + 1..]
                .iter()
                .all(|event| event.1 != RuntimeEventKind::CommandResponse),
            "no business response may cross the stopping boundary"
        );
        let responses: Vec<&(FfiHandle, RuntimeEventKind, Vec<u8>)> = events
            .iter()
            .filter(|(_, kind, _)| *kind == RuntimeEventKind::CommandResponse)
            .collect();
        assert_eq!(responses.len(), 2);
        for response in &responses {
            assert_eq!(&response.2[..4], b"CERM");
        }
        // Response payloads deterministically embed the dispatch counter.
        // Since the dispatch counter is 1 then 2, we should see both markers.
        let mut have_first = false;
        let mut have_second = false;
        for response in &responses {
            let payload = &response.2[20..];
            if payload == b"config:\x01binary-event".as_slice()
                || payload == b"config:\x01again".as_slice()
            {
                have_first = true;
            } else if payload == b"config:\x02binary-event".as_slice()
                || payload == b"config:\x02again".as_slice()
            {
                have_second = true;
            }
        }
        assert!(
            have_first && have_second,
            "response payloads: {responses:?}"
        );
    }

    #[test]
    fn incompatible_abi_is_rejected_without_a_handle() {
        let mut host = host();
        host.header.version.major += 1;
        let mut runtime = 0;
        // SAFETY: Test pointers remain valid through the call.
        assert_eq!(
            unsafe {
                start_browser::<TestBrowser>(&raw const host, ptr::null(), 0, &raw mut runtime)
            },
            Status::IncompatibleAbi
        );
        assert_eq!(runtime, 0);
    }
}
