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
use crate::browser::{Browser, BrowserContext};
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

unsafe fn discover_capabilities(
    host: &Host,
) -> Result<
    (
        tuffite_abi::UtilitySessionClient,
        Option<tuffite_abi::ShellApi>,
    ),
    Status,
> {
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
    executor.close_and_wait();
    providers.release();
}

struct RuntimeInstance {
    runtime: Runtime,
    providers: Arc<crate::utility::V8Providers>,
    app_scheme: Option<String>,
    callbacks: NativeHost,
    runtime_identity: Arc<AtomicU64>,
    scheduler: DispatchScheduler,
    executor: ChromiumExecutor,
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
        let browser = match catch_unwind(AssertUnwindSafe(|| B::new(context))) {
            Ok(Ok(browser)) => Box::new(browser) as Box<dyn Browser>,
            result => {
                abandon_browser(&providers, &executor);
                if let Ok(Err(error)) = result {
                    tracing::error!(error = %error, "Tuffite browser initialization failed");
                    return Status::InvalidState;
                }
                return Status::Internal;
            }
        };
        let runtime = Runtime::new();
        // Start the scheduler with an atomic runtime handle initialized to 0.
        // The worker will only post_event once the handle has been published,
        // which happens after we insert the instance into the table below.
        let scheduler = DispatchScheduler::start(&runtime_identity, browser, callbacks);
        let instance = RuntimeInstance {
            runtime,
            providers,
            app_scheme,
            callbacks,
            runtime_identity: Arc::clone(&runtime_identity),
            scheduler,
            executor,
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
            if let Ok(mut instance) = removed {
                instance.providers.stop();
                let _ = instance.scheduler.shutdown();
                instance.executor.close_and_wait();
                let _ = instance.runtime.stop();
                instance.providers.release();
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

/// Resolves a V8 source on a supervised browser worker, joined during stop.
///
/// # Safety
/// Input slices are readable for the call. On acceptance reply/context remain
/// valid and thread-safe until one reply; rejection does not invoke the callback.
pub unsafe extern "C" fn resolve_v8(
    runtime: FfiHandle,
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
        let callback = ModuleReply { data, reply };
        let url = url.to_owned();
        let name = name.to_owned();
        let providers = instance.providers.clone();
        match instance.runtime.spawn("v8-resolve", move |context| {
            let result = catch_unwind(AssertUnwindSafe(|| {
                let provider = providers.get(&name)?;
                crate::utility::resolve_source(provider.as_ref(), &name, &url)
            }));
            let (status, bytes) = if context.is_cancelled() {
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
        let mut instance = match runtimes()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(handle)
        {
            Ok(instance) => instance,
            Err(error) => return map_handle_error(error),
        };
        // Publish lifecycle control before waiting for application work. The
        // Framework can immediately reject new/pending requests even when a
        // business handler is slow. Clearing the worker-visible identity then
        // prevents late CommandResponse events from crossing the stopping
        // boundary while shutdown joins the worker.
        let _ = post_event(instance.callbacks, handle, RuntimeEventKind::Stopping, &[]);
        instance.providers.stop();
        instance.runtime_identity.store(0, Ordering::Release);
        let application = instance.scheduler.shutdown();
        instance.executor.close_and_wait();
        let shutdown_status = match application {
            Some(mut application) => {
                match catch_unwind(AssertUnwindSafe(|| application.shutdown())) {
                    Ok(Ok(())) => Status::Ok,
                    Ok(Err(_)) | Err(_) => Status::Internal,
                }
            }
            None => Status::Internal,
        };
        let runtime_status = instance.runtime.stop();
        instance.providers.release();
        match (shutdown_status, runtime_status) {
            (Status::Ok, Ok(())) => Status::Ok,
            (status, _) if status != Status::Ok => status,
            (_, Err(_)) => Status::InvalidState,
            _ => Status::Internal,
        }
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

        fn shutdown(&mut self) -> crate::ApplicationResult<()> {
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
