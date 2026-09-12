//! Explicit application construction and generated command registration.

use std::any::Any;
use std::sync::OnceLock;

use crate::{
    ApplicationError, ApplicationResult, Browser, BrowserContext, Command, CommandResult,
    UtilityServiceRegistration,
};
use tuffite_abi::Application as AbiApplication;

/// Generated metadata for one application `ShellAPI` endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandDescriptor {
    channel: &'static str,
    id: u64,
    schema_fingerprint: u64,
}

impl CommandDescriptor {
    /// Creates generated endpoint metadata.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(channel: &'static str, id: u64, schema_fingerprint: u64) -> Self {
        Self {
            channel,
            id,
            schema_fingerprint,
        }
    }

    /// Returns the complete API path used as the `ShellAPI` channel.
    #[must_use]
    pub const fn channel(self) -> &'static str {
        self.channel
    }

    /// Returns the stable protocol command identifier.
    #[must_use]
    pub const fn id(self) -> u64 {
        self.id
    }

    /// Returns the fingerprint of the normalized request/result schema.
    #[must_use]
    pub const fn schema_fingerprint(self) -> u64 {
        self.schema_fingerprint
    }
}

type DispatchFn<A> = for<'a> fn(&mut A, Command<'a>) -> CommandResult;

/// Compile-time binding between one generated endpoint and its Rust handler.
///
/// Implementations are emitted by `#[tuffite::command(endpoint::Type)]`;
/// application code should not implement this trait directly.
#[doc(hidden)]
pub trait CommandHandler {
    type Browser: Browser;

    const COMMAND: CommandDescriptor;

    fn dispatch(browser: &mut Self::Browser, command: Command<'_>) -> CommandResult;
}

/// Statically generated command dispatch table for one browser backend.
pub struct CommandRegistry<A> {
    dispatch: DispatchFn<A>,
    descriptors: &'static [CommandDescriptor],
}

impl<A> CommandRegistry<A> {
    /// Creates a generated registry.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(dispatch: DispatchFn<A>, descriptors: &'static [CommandDescriptor]) -> Self {
        Self {
            dispatch,
            descriptors,
        }
    }
}

trait ErasedRegistry: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn utility_services(&self) -> &'static [UtilityServiceRegistration];
}

struct TypedRegistry<A> {
    dispatch: DispatchFn<A>,
    utility_services: &'static [UtilityServiceRegistration],
    prepare: Option<StartupHook>,
}

impl<A: Send + 'static> ErasedRegistry for TypedRegistry<A> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn utility_services(&self) -> &'static [UtilityServiceRegistration] {
        self.utility_services
    }
}

fn registry() -> &'static OnceLock<Box<dyn ErasedRegistry>> {
    static REGISTRY: OnceLock<Box<dyn ErasedRegistry>> = OnceLock::new();
    &REGISTRY
}

pub(crate) fn utility_definition(name: &str) -> Option<UtilityServiceRegistration> {
    registry()
        .get()?
        .utility_services()
        .iter()
        .copied()
        .find(|item| item.name() == name)
}

struct RegisteredApplication<B>(std::marker::PhantomData<fn() -> B>);

struct RegisteredBrowser<B> {
    browser: B,
    dispatch: DispatchFn<B>,
}

impl<B: Browser> crate::application::Registrations for RegisteredApplication<B> {
    fn utility_services() -> &'static [UtilityServiceRegistration] {
        registry()
            .get()
            .and_then(|entry| entry.as_any().downcast_ref::<TypedRegistry<B>>())
            .map_or(&[], |entry| entry.utility_services)
    }
}

impl<B: Browser> Browser for RegisteredBrowser<B> {
    fn new(context: BrowserContext) -> ApplicationResult<Self> {
        let dispatch = registry()
            .get()
            .and_then(|entry| entry.as_any().downcast_ref::<TypedRegistry<B>>())
            .ok_or_else(|| ApplicationError::new("command registry is not installed"))?
            .dispatch;
        Ok(Self {
            browser: B::new(context)?,
            dispatch,
        })
    }

    fn dispatch(&mut self, command: Command<'_>) -> CommandResult {
        (self.dispatch)(&mut self.browser, command)
    }

    fn shutdown(&mut self) -> ApplicationResult<()> {
        self.browser.shutdown()
    }
}

/// Native process role, classified before application argument rewriting.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessKind {
    Browser,
    Utility,
    Renderer,
    Gpu,
    Other,
}

impl ProcessKind {
    fn from_raw(kind: u32) -> Option<Self> {
        match kind {
            0 => Some(Self::Browser),
            1 => Some(Self::Utility),
            2 => Some(Self::Renderer),
            3 => Some(Self::Gpu),
            4 => Some(Self::Other),
            _ => None,
        }
    }
}

/// Process launch arguments, including the executable at index zero.
///
/// The startup hook runs before `ContentMain` and may follow sandbox bootstrap. The
/// executable and Chromium process role must remain unchanged.
#[derive(Clone, Debug)]
pub struct ProcessLaunch {
    process_kind: ProcessKind,
    args: Vec<String>,
}

impl ProcessLaunch {
    /// Returns the original role supplied by native bootstrap.
    #[must_use]
    pub const fn process_kind(&self) -> ProcessKind {
        self.process_kind
    }

    /// Original or currently edited UTF-8 command-line arguments.
    #[must_use]
    pub fn argv(&self) -> &[String] {
        &self.args
    }

    /// Edits process arguments. Index zero must retain the executable.
    pub fn argv_mut(&mut self) -> &mut Vec<String> {
        &mut self.args
    }
}

type StartupHook = Box<dyn Fn(&mut ProcessLaunch) -> ApplicationResult<()> + Send + Sync>;

/// Assembles process definitions and Framework startup configuration.
///
/// The main factory is evaluated once in every process loading the business
/// library. It must only assemble definitions; hooks and browser state run later.
pub struct Builder<B: Browser> {
    commands: Option<CommandRegistry<B>>,
    prepare: Option<StartupHook>,
}

impl<B: Browser> Builder<B> {
    /// Creates a builder using the backend's dispatch implementation.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            commands: None,
            prepare: None,
        }
    }

    /// Installs generated handlers in place of the backend's dispatch method.
    #[must_use]
    pub fn commands(mut self, commands: CommandRegistry<B>) -> Self {
        self.commands = Some(commands);
        self
    }

    /// Installs a hook for all process roles before Chromium `ContentMain`.
    #[must_use]
    pub fn before_startup(
        mut self,
        hook: impl Fn(&mut ProcessLaunch) -> ApplicationResult<()> + Send + Sync + 'static,
    ) -> Self {
        self.prepare = Some(Box::new(hook));
        self
    }

    /// Installs definitions selected by the main macro and builds the ABI table.
    #[doc(hidden)]
    pub fn into_application(
        self,
        utility_services: &'static [UtilityServiceRegistration],
    ) -> ApplicationResult<AbiApplication> {
        let commands = self
            .commands
            .unwrap_or_else(|| CommandRegistry::new(B::dispatch, &[]));
        for (index, descriptor) in commands.descriptors.iter().enumerate() {
            if descriptor.channel().is_empty() || !descriptor.channel().contains('.') {
                return Err(ApplicationError::new(
                    "registered command channel is invalid",
                ));
            }
            if commands.descriptors[..index].iter().any(|registered| {
                registered.channel() == descriptor.channel() || registered.id() == descriptor.id()
            }) {
                return Err(ApplicationError::new(
                    "duplicate registered ShellAPI command",
                ));
            }
        }
        for (index, registration) in utility_services.iter().copied().enumerate() {
            crate::utility_entry::validate_registration(&utility_services[..index], registration)
                .map_err(ApplicationError::new)?;
        }
        let has_prepare = self.prepare.is_some();
        registry()
            .set(Box::new(TypedRegistry::<B> {
                dispatch: commands.dispatch,
                utility_services,
                prepare: self.prepare,
            }))
            .map_err(|_| ApplicationError::new("a Tuffite application is already installed"))?;
        let mut table =
            crate::application::for_application::<RegisteredBrowser<B>, RegisteredApplication<B>>();
        if has_prepare {
            table.prepare_process = Some(prepare_process::<B>);
        }
        Ok(table)
    }
}

impl<B: Browser> Default for Builder<B> {
    fn default() -> Self {
        Self::new()
    }
}

/// ABI 2.3 process launch adapter. Input and sink bytes are borrowed for each call.
///
/// # Safety
/// The input array and each byte slice must be readable for the call. The sink
/// context and callback must remain valid until this synchronous call returns.
pub(crate) unsafe extern "C" fn prepare_process<B: Browser>(
    process_kind: u32,
    args: *const tuffite_abi::ByteSlice,
    count: usize,
    user_data: *mut std::ffi::c_void,
    emit: Option<tuffite_abi::LaunchArgumentFn>,
) -> tuffite_abi::Status {
    use tuffite_abi::Status;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let Some(process_kind) = ProcessKind::from_raw(process_kind) else {
            return Status::InvalidArgument;
        };
        let Some(emit) = emit else {
            return Status::InvalidArgument;
        };
        if args.is_null() || count == 0 || count > 4096 {
            return Status::InvalidArgument;
        }
        // SAFETY: Input array is readable by the caller's contract.
        let input = unsafe { std::slice::from_raw_parts(args, count) };
        let mut decoded = Vec::with_capacity(count);
        let mut total = 0usize;
        for arg in input {
            if arg.len > 1024 * 1024 || (arg.len != 0 && arg.data.is_null()) {
                return Status::InvalidArgument;
            }
            total = total.saturating_add(arg.len);
            if total > 1024 * 1024 {
                return Status::InvalidArgument;
            }
            let bytes = if arg.len == 0 {
                &[]
            } else {
                // SAFETY: Each nonempty slice is readable by contract.
                unsafe { std::slice::from_raw_parts(arg.data, arg.len) }
            };
            let Ok(value) = std::str::from_utf8(bytes) else {
                return Status::InvalidArgument;
            };
            if value.contains('\0') {
                return Status::InvalidArgument;
            }
            decoded.push(value.to_owned());
        }
        let executable = decoded[0].clone();
        let mut launch = ProcessLaunch {
            process_kind,
            args: decoded,
        };
        let Some(entry) = registry()
            .get()
            .and_then(|entry| entry.as_any().downcast_ref::<TypedRegistry<B>>())
        else {
            return Status::InvalidState;
        };
        if let Some(hook) = &entry.prepare
            && let Err(error) = hook(&mut launch)
        {
            tracing::error!(%error, "process launch preparation failed");
            return Status::InvalidState;
        }
        if launch.args.first() != Some(&executable)
            || executable.is_empty()
            || launch.args.len() > 4096
            || launch.args.iter().any(|arg| arg.contains('\0'))
            || launch.args.iter().map(String::len).sum::<usize>() > 1024 * 1024
        {
            return Status::InvalidArgument;
        }
        for arg in &launch.args {
            // SAFETY: The synchronous sink copies bytes before returning.
            let status = unsafe {
                emit(
                    user_data,
                    tuffite_abi::ByteSlice::from_slice(arg.as_bytes()),
                )
            };
            if status != Status::Ok {
                return status;
            }
        }
        Status::Ok
    }))
    .unwrap_or(Status::Internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct App;

    impl Browser for App {
        fn new(_: BrowserContext) -> ApplicationResult<Self> {
            Ok(Self)
        }
    }

    struct Backend;

    struct Worker;

    static BROWSER_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    impl Browser for Backend {
        fn new(context: BrowserContext) -> ApplicationResult<Self> {
            context.set_state(std::sync::Arc::new(String::from("shared")));
            BROWSER_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Self)
        }
    }

    impl crate::Utility for Worker {
        fn new(_: crate::UtilityContext) -> ApplicationResult<Self> {
            Ok(Self)
        }

        fn invoke(&self, _: crate::UtilityRequest) -> crate::UtilityFuture<'_> {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    unsafe extern "C" fn collect_argument(
        context: *mut std::ffi::c_void,
        argument: tuffite_abi::ByteSlice,
    ) -> tuffite_abi::Status {
        // SAFETY: Test provides a live vector and ABI borrows a live byte slice.
        let output = unsafe { &mut *context.cast::<Vec<String>>() };
        let bytes = unsafe { std::slice::from_raw_parts(argument.data, argument.len) };
        output.push(std::str::from_utf8(bytes).unwrap().to_owned());
        tuffite_abi::Status::Ok
    }

    unsafe extern "C" fn ignore_event(
        _: *mut std::ffi::c_void,
        _: tuffite_abi::Handle,
        _: tuffite_abi::RuntimeEventKind,
        _: tuffite_abi::ByteSlice,
    ) -> tuffite_abi::Status {
        tuffite_abi::Status::Ok
    }

    unsafe extern "C" fn reject_task(
        _: *mut std::ffi::c_void,
        _: tuffite_abi::TaskPriority,
        _: u8,
        _: *mut std::ffi::c_void,
        _: Option<tuffite_abi::TaskRunFn>,
        _: Option<tuffite_abi::TaskDestroyFn>,
    ) -> tuffite_abi::Status {
        tuffite_abi::Status::InvalidState
    }

    #[test]
    fn application_install_and_utility_start_do_not_construct_browser() {
        use std::sync::atomic::Ordering;
        use tuffite_abi::Status;

        const SERVICES: &[UtilityServiceRegistration] = &[
            UtilityServiceRegistration::new::<Worker>("entry_test"),
            UtilityServiceRegistration::v8::<V8Worker>("v8_entry_test"),
            UtilityServiceRegistration::v8::<FailedV8Worker>("failed_v8"),
        ];
        let table = Builder::<Backend>::new()
            .before_startup(|launch| {
                assert!(
                    !launch.argv().iter().any(|arg| arg == "--panic"),
                    "hook panic"
                );
                if launch.argv().iter().any(|arg| arg == "--error") {
                    return Err(ApplicationError::new("hook failed"));
                }
                if launch
                    .argv()
                    .iter()
                    .any(|arg| arg == "--replace-executable")
                {
                    launch.argv_mut()[0] = "different".into();
                }
                launch.argv_mut().retain(|arg| arg != "--business-debug");
                Ok(())
            })
            .into_application(SERVICES)
            .expect("application definitions install");
        assert_eq!(BROWSER_COUNT.load(Ordering::SeqCst), 0);

        let prepare = table.prepare_process.expect("startup hook exported");
        for (flag, expected) in [
            ("--business-debug", tuffite_abi::Status::Ok),
            ("--panic", tuffite_abi::Status::Internal),
            ("--error", tuffite_abi::Status::InvalidState),
            ("--replace-executable", tuffite_abi::Status::InvalidArgument),
        ] {
            let input = [
                tuffite_abi::ByteSlice::from_slice(b"app"),
                tuffite_abi::ByteSlice::from_slice(flag.as_bytes()),
            ];
            let mut output = Vec::<String>::new();
            // SAFETY: Input and synchronous output state stay live throughout the call.
            let status = unsafe {
                prepare(
                    0,
                    input.as_ptr(),
                    input.len(),
                    (&raw mut output).cast(),
                    Some(collect_argument),
                )
            };
            assert_eq!(status, expected);
            if expected == tuffite_abi::Status::Ok {
                assert_eq!(output, ["app"]);
            } else {
                assert!(output.is_empty());
            }
        }
        check_process_roles(prepare);
        check_v8_definition();
        assert_eq!(BROWSER_COUNT.load(Ordering::SeqCst), 0);

        let name = b"entry_test";
        let mut worker = 0;
        // SAFETY: Static service name and writable handle satisfy the ABI contract.
        let status =
            unsafe { table.utility.start.unwrap()(name.as_ptr(), name.len(), &raw mut worker) };
        assert_eq!(status, Status::Ok);
        assert_ne!(worker, 0);
        assert_eq!(BROWSER_COUNT.load(Ordering::SeqCst), 0);
        // SAFETY: This test owns the live worker handle and has no pending calls.
        assert_eq!(unsafe { table.utility.stop.unwrap()(worker) }, Status::Ok);

        let host = tuffite_abi::Host {
            header: tuffite_abi::Header::for_type::<tuffite_abi::Host>(),
            user_data: std::ptr::null_mut(),
            post_event: Some(ignore_event),
            post_task: Some(reject_task),
            query_extension: None,
        };
        let mut browser = 0;
        // SAFETY: Host remains live until stop; the empty configuration is valid.
        let status = unsafe {
            table.start.unwrap()(
                &raw const host,
                PROVIDER_CONFIG.as_ptr(),
                PROVIDER_CONFIG.len(),
                &raw mut browser,
            )
        };
        assert_eq!(status, Status::Ok);
        assert_ne!(browser, 0);
        assert_eq!(BROWSER_COUNT.load(Ordering::SeqCst), 1);
        check_resolver_lifecycle(browser, table.resolve_v8.unwrap());
        // SAFETY: This test owns the live browser handle and has no pending calls.
        assert_eq!(unsafe { table.stop.unwrap()(browser) }, Status::Ok);
    }

    fn check_resolver_lifecycle(browser: tuffite_abi::Handle, resolve: tuffite_abi::ResolveV8Fn) {
        use tuffite_abi::{ByteSlice, Status};
        unsafe extern "C" fn reply(
            data: *mut std::ffi::c_void,
            status: Status,
            bytes: ByteSlice,
        ) -> Status {
            // SAFETY: Sender stays alive until the worker has delivered its one reply.
            let sender = unsafe { &*data.cast::<std::sync::mpsc::Sender<(Status, Vec<u8>)>>() };
            let bytes = if bytes.len == 0 {
                Vec::new()
            } else {
                // SAFETY: The resolver borrows the response buffer for this callback.
                unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) }.to_vec()
            };
            sender.send((status, bytes)).unwrap();
            Status::Ok
        }
        let (sender, receiver) = std::sync::mpsc::channel::<(Status, Vec<u8>)>();
        let context = std::ptr::from_ref(&sender).cast_mut().cast();
        for (url, expected) in [
            ("myapp://v8_entry_test.utility/entry.mjs", Status::Ok),
            ("https://example.com/sdk/runtime.mjs?v=1", Status::Ok),
            ("file:///tmp/sdk/runtime.mjs", Status::Ok),
            ("myapp://v8_entry_test.utility/asset.mjs", Status::NotFound),
            (
                "myapp://v8_entry_test.utility/missing.mjs",
                Status::InvalidArgument,
            ),
            ("myapp://v8_entry_test.utility/panic.mjs", Status::Internal),
        ] {
            // SAFETY: Inputs are borrowed for the call; callback state lives through completion.
            assert_eq!(
                unsafe {
                    resolve(
                        browser,
                        ByteSlice::from_slice(b"v8_entry_test"),
                        ByteSlice::from_slice(url.as_bytes()),
                        context,
                        Some(reply),
                    )
                },
                Status::Ok
            );
            let (status, source) = receiver
                .recv_timeout(std::time::Duration::from_secs(5))
                .unwrap();
            assert_eq!(status, expected);
            assert_eq!(!source.is_empty(), expected == Status::Ok);
            assert!(
                receiver.try_recv().is_err(),
                "accepted request replies exactly once"
            );
        }
        for (handle, url, expected) in [
            (browser, "../escape.mjs", Status::InvalidArgument),
            (
                0,
                "myapp://v8_entry_test.utility/entry.mjs",
                Status::InvalidHandle,
            ),
        ] {
            // SAFETY: Rejection retains ownership of callback context in this caller.
            assert_eq!(
                unsafe {
                    resolve(
                        handle,
                        ByteSlice::from_slice(b"v8_entry_test"),
                        ByteSlice::from_slice(url.as_bytes()),
                        context,
                        Some(reply),
                    )
                },
                expected
            );
            assert!(
                receiver.try_recv().is_err(),
                "rejected request must not reply"
            );
        }
    }

    static V8_RESOURCES: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    const PROVIDER_CONFIG: &[u8] =
        br#"{"app":{"utilities":{"v8_entry_test":{"configuration":{"flag":true}}}}}"#;
    static V8_STARTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    static FAILED_V8_STARTS: std::sync::atomic::AtomicUsize =
        std::sync::atomic::AtomicUsize::new(0);

    struct FailedV8Worker;
    impl crate::utility::V8 for FailedV8Worker {
        fn new(_: crate::V8Context) -> ApplicationResult<Self> {
            FAILED_V8_STARTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(ApplicationError::new("provider startup failed"))
        }
        fn entry(&self) -> &'static str {
            "entry.mjs"
        }
    }

    struct V8Worker {
        context: crate::V8Context,
        resolutions: std::sync::atomic::AtomicUsize,
    }

    impl crate::utility::V8 for V8Worker {
        fn new(context: crate::V8Context) -> ApplicationResult<Self> {
            assert_eq!(context.name(), "v8_entry_test");
            assert_eq!(context.configuration(), br#"{"flag":true}"#);
            assert_eq!(
                context.state::<String>().as_deref().map(String::as_str),
                Some("shared")
            );
            assert!(!context.is_stopping());
            V8_STARTS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Self {
                context,
                resolutions: std::sync::atomic::AtomicUsize::new(0),
            })
        }
        fn entry(&self) -> &'static str {
            "entry.mjs"
        }

        fn resolve(
            &self,
            url: &str,
        ) -> Result<Option<crate::utility::v8::ModuleSource>, crate::utility::v8::Error> {
            assert!(!self.context.is_stopping());
            let count = self
                .resolutions
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                + 1;
            V8_RESOURCES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            assert!(!url.ends_with("/panic.mjs"), "resolver panic");
            if url.ends_with("/missing.mjs") {
                return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
            }
            if url.ends_with("/asset.mjs") {
                return Ok(None);
            }
            Ok(Some(crate::utility::v8::ModuleSource::new(
                url.to_owned(),
                format!("export default {{ping: () => {count}}};"),
            )))
        }
    }

    unsafe extern "C" fn observe_v8_create(
        _: *mut std::ffi::c_void,
        name: *const u8,
        name_len: usize,
        _: u64,
        init: *const u8,
        init_len: usize,
        _: u32,
        _: usize,
        _: *mut std::ffi::c_void,
        _: Option<tuffite_abi::UtilityHostEventFn>,
        _: Option<tuffite_abi::UtilityHostClosedFn>,
        _: *mut std::ffi::c_void,
        _: Option<tuffite_abi::UtilityHostCreateReplyFn>,
    ) -> tuffite_abi::Status {
        // SAFETY: Session creation borrows both buffers for this synchronous call.
        assert_eq!(
            unsafe { std::slice::from_raw_parts(name, name_len) },
            b"v8_entry_test"
        );
        let package: serde_json::Value =
            serde_json::from_slice(unsafe { std::slice::from_raw_parts(init, init_len) }).unwrap();
        assert_eq!(package["entry"], "myapp://v8_entry_test.utility/entry.mjs");
        assert_eq!(package["schemaVersion"], 2);
        assert!(package.get("modules").is_none());
        // Rejection retains callback ownership in the runtime and creates no process.
        tuffite_abi::Status::InvalidState
    }

    fn test_providers() -> std::sync::Arc<crate::utility::V8Providers> {
        let host = crate::abi::NativeHost(tuffite_abi::Host {
            header: tuffite_abi::Header::for_type::<tuffite_abi::Host>(),
            user_data: std::ptr::null_mut(),
            post_event: Some(ignore_event),
            post_task: Some(reject_task),
            query_extension: None,
        });
        let state = crate::browser::SharedState::default();
        state.set(std::sync::Arc::new(String::from("shared")));
        std::sync::Arc::new(crate::utility::V8Providers::new(
            std::sync::Arc::from(PROVIDER_CONFIG),
            crate::ChromiumExecutor::new(host),
            state,
        ))
    }

    fn check_v8_definition() {
        use std::sync::atomic::Ordering;
        let host = crate::utility::Host::new(
            crate::UtilitySessionHost::new(tuffite_abi::UtilitySessionClient {
                create: Some(observe_v8_create),
                ..Default::default()
            }),
            Some("myapp".to_owned()),
            test_providers(),
        );
        assert_eq!(V8_RESOURCES.load(Ordering::SeqCst), 0);
        assert!(host.create_v8("unknown", |_| {}, |_| {}).is_err());
        assert!(host.create_v8("entry_test", |_| {}, |_| {}).is_err());
        assert_eq!(V8_RESOURCES.load(Ordering::SeqCst), 0);
        assert!(host.create_v8("v8_entry_test", |_| {}, |_| {}).is_err());
        assert_eq!(V8_RESOURCES.load(Ordering::SeqCst), 0);
        let providers = test_providers();
        let threads: Vec<_> = (0..8)
            .map(|_| {
                let providers = providers.clone();
                std::thread::spawn(move || providers.get("v8_entry_test").unwrap())
            })
            .collect();
        let provider = threads
            .into_iter()
            .map(|worker| worker.join().unwrap())
            .reduce(|first, next| {
                assert!(std::sync::Arc::ptr_eq(&first, &next));
                first
            })
            .unwrap();
        assert_eq!(
            V8_STARTS.load(Ordering::SeqCst),
            2,
            "one provider per name per runtime"
        );
        assert!(providers.get("failed_v8").is_err());
        assert!(providers.get("failed_v8").is_err());
        assert_eq!(
            FAILED_V8_STARTS.load(Ordering::SeqCst),
            1,
            "startup failure is cached"
        );
        assert!(
            crate::utility::resolve_source(provider.as_ref(), "v8_entry_test", "../escape.mjs")
                .is_err()
        );
        assert_eq!(V8_RESOURCES.load(Ordering::SeqCst), 0);
        assert!(
            crate::utility::resolve_source(
                provider.as_ref(),
                "v8_entry_test",
                "myapp://v8_entry_test.utility/entry.mjs"
            )
            .is_ok()
        );
        assert_eq!(V8_RESOURCES.load(Ordering::SeqCst), 1);
        let second = providers.get("v8_entry_test").unwrap();
        let source = crate::utility::resolve_source(
            second.as_ref(),
            "v8_entry_test",
            "myapp://v8_entry_test.utility/entry.mjs",
        )
        .unwrap()
        .unwrap();
        assert!(
            std::str::from_utf8(&source).unwrap().contains("=> 2"),
            "instance state survives calls"
        );
        providers.stop();
        assert!(providers.get("v8_entry_test").is_err());
    }

    fn check_process_roles(prepare: tuffite_abi::PrepareProcessFn) {
        use tuffite_abi::Status;
        for (kind, expected) in [
            (0, ProcessKind::Browser),
            (1, ProcessKind::Utility),
            (2, ProcessKind::Renderer),
            (3, ProcessKind::Gpu),
            (4, ProcessKind::Other),
        ] {
            let input = [tuffite_abi::ByteSlice::from_slice(b"app")];
            let mut output = Vec::<String>::new();
            assert_eq!(ProcessKind::from_raw(kind), Some(expected));
            // SAFETY: Input and sink remain valid for this synchronous call.
            assert_eq!(
                unsafe {
                    prepare(
                        kind,
                        input.as_ptr(),
                        input.len(),
                        (&raw mut output).cast(),
                        Some(collect_argument),
                    )
                },
                Status::Ok
            );
            assert_eq!(output, ["app"]);
        }
        let input = [tuffite_abi::ByteSlice::from_slice(b"app")];
        // SAFETY: Input remains valid and the unknown role is rejected before sink access.
        assert_eq!(
            unsafe {
                prepare(
                    u32::MAX,
                    input.as_ptr(),
                    input.len(),
                    std::ptr::null_mut(),
                    Some(collect_argument),
                )
            },
            Status::InvalidArgument
        );
    }

    #[test]
    fn descriptor_exposes_full_channel_and_id() {
        const DESCRIPTOR: CommandDescriptor =
            CommandDescriptor::new("example.commands.ping", 42, 77);
        assert_eq!(DESCRIPTOR.channel(), "example.commands.ping");
        assert_eq!(DESCRIPTOR.id(), 42);
        assert_eq!(DESCRIPTOR.schema_fingerprint(), 77);
    }

    #[test]
    fn builder_starts_without_registered_commands() {
        let builder = Builder::<App>::new();
        assert!(builder.commands.is_none());
        assert!(builder.prepare.is_none());
    }
}
