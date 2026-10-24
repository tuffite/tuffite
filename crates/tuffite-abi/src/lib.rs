//! Stable C ABI declarations shared by Tuffite Runtime and Framework.
//!
//! Applications should use the safe `tuffite` crate instead of this crate.

use std::ffi::c_void;

/// ABI crate release. Major changes break the binary contract; minor changes
/// append compatible capabilities; patch releases preserve its layout.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Binary compatibility major, derived from this crate's version.
pub const ABI_MAJOR: u16 = version_component(env!("CARGO_PKG_VERSION_MAJOR"));
/// Required binary capability minor, derived from this crate's version.
pub const ABI_MINOR: u16 = version_component(env!("CARGO_PKG_VERSION_MINOR"));
/// Compatible fixes, derived from this crate's version.
pub const ABI_PATCH: u16 = version_component(env!("CARGO_PKG_VERSION_PATCH"));

#[allow(clippy::cast_possible_truncation)] // The bound is checked before conversion.
const fn version_component(value: &str) -> u16 {
    let bytes = value.as_bytes();
    let mut result = 0_u32;
    let mut index = 0;
    while index < bytes.len() {
        result = result * 10 + (bytes[index] - b'0') as u32;
        assert!(
            result <= u16::MAX as u32,
            "ABI version component exceeds u16"
        );
        index += 1;
    }
    result as u16
}

/// A provider must match the required major and satisfy the full minimum
/// version. This binary minimum rule also applies before 1.0; Cargo requirements
/// follow Cargo's separate pre-1.0 compatibility rules.
#[must_use]
pub const fn is_compatible(provided: (u16, u16, u16), required: (u16, u16, u16)) -> bool {
    provided.0 == required.0
        && (provided.1 > required.1 || (provided.1 == required.1 && provided.2 >= required.2))
}

/// Binary API version using the ABI crate's stable semver contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct Version {
    /// Breaking binary contract revision.
    pub major: u16,
    /// Compatible capability revision.
    pub minor: u16,
    /// Compatible implementation fixes.
    pub patch: u16,
}

impl Version {
    /// Version of this ABI crate.
    pub const CURRENT: Self = Self {
        major: ABI_MAJOR,
        minor: ABI_MINOR,
        patch: ABI_PATCH,
    };

    /// Minimum original contract: 0.1.0 before 1.0, then major.0.0.
    /// Optional capabilities declare their own minimum when they evolve.
    pub const BASELINE: Self = Self {
        major: ABI_MAJOR,
        minor: if ABI_MAJOR == 0 { 1 } else { 0 },
        patch: 0,
    };

    /// Whether this provider satisfies a stable semver minimum requirement.
    #[must_use]
    pub const fn is_compatible_with(self, required: Self) -> bool {
        is_compatible(
            (self.major, self.minor, self.patch),
            (required.major, required.minor, required.patch),
        )
    }
}

/// Common prefix of independently exchanged ABI callback tables.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Header {
    /// Size of the complete outer table, not just this header.
    pub struct_size: usize,
    /// Binary capabilities provided by the table.
    pub version: Version,
    /// Reserved; must be zero.
    pub reserved: u16,
}

impl Header {
    /// Describes an outer ABI table using this crate's version.
    #[must_use]
    pub const fn for_type<T>() -> Self {
        Self {
            struct_size: std::mem::size_of::<T>(),
            version: Version::CURRENT,
            reserved: 0,
        }
    }
}

/// Pointer-free handle representation used by the C ABI.
pub type Handle = u64;

/// Stable status values returned by exported C functions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i32)]
pub enum Status {
    /// Operation completed successfully.
    Ok = 0,
    /// A pointer, length, enum, or payload was invalid.
    InvalidArgument = 1,
    /// The requested ABI major version is incompatible.
    IncompatibleAbi = 2,
    /// The handle is invalid, stale, missing, or the wrong kind.
    InvalidHandle = 3,
    /// Runtime is stopping or already stopped.
    InvalidState = 4,
    /// The request failed because a resource could not be allocated.
    ResourceExhausted = 5,
    /// A panic or unexpected implementation failure was contained.
    Internal = 6,
    /// A resolver deferred to the native app.assets resource loader.
    NotFound = 7,
}

/// Borrowed bytes valid only for the duration of an ABI call.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ByteSlice {
    /// Pointer to the first byte, or null only when `len` is zero.
    pub data: *const u8,
    /// Number of readable bytes.
    pub len: usize,
}

impl ByteSlice {
    /// Creates an ABI view over a Rust slice.
    #[must_use]
    pub const fn from_slice(bytes: &[u8]) -> Self {
        Self {
            data: bytes.as_ptr(),
            len: bytes.len(),
        }
    }
}

/// Events emitted from Runtime toward Framework.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RuntimeEventKind {
    /// Runtime was started and its handle is valid.
    Started = 1,
    /// A dispatch payload was accepted by Runtime.
    DispatchAccepted = 2,
    /// Runtime is about to invalidate its handle.
    Stopping = 3,
    /// Binary Shell Protocol response produced by Runtime.
    CommandResponse = 4,
    /// Named binary event broadcast to authorized renderer documents.
    ApplicationEvent = 5,
}

/// Thread-safe callback implemented by Tuffite Framework.
///
/// `payload` is borrowed for the duration of the callback. The callback must not
/// unwind, throw, block for Runtime work, or synchronously re-enter Runtime. It
/// should copy or enqueue the event and return.
pub type PostEventFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    runtime: Handle,
    kind: RuntimeEventKind,
    payload: ByteSlice,
) -> Status;

/// Priority understood by Chromium's task scheduler.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum TaskPriority {
    /// Deferrable background work.
    BestEffort = 0,
    /// Work whose result will become visible to the user.
    UserVisible = 1,
    /// Work required to respond immediately to a user action.
    UserBlocking = 2,
}

/// Runs a task previously accepted by Framework.
pub type TaskRunFn = unsafe extern "C" fn(task_data: *mut c_void);
/// Releases task state exactly once after an accepted task finishes.
pub type TaskDestroyFn = unsafe extern "C" fn(task_data: *mut c_void);
/// Posts one opaque Rust task to Chromium's process-wide thread pool.
pub type PostTaskFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    priority: TaskPriority,
    may_block: u8,
    task_data: *mut c_void,
    run: Option<TaskRunFn>,
    destroy: Option<TaskDestroyFn>,
) -> Status;

/// Creates one application instance for a browser-process Runtime.
pub type ApplicationStartFn = unsafe extern "C" fn(
    framework: *const Host,
    config: *const u8,
    config_len: usize,
    runtime_out: *mut Handle,
) -> Status;

/// Dispatches one binary Shell Protocol request to an application instance.
pub type ApplicationDispatchFn =
    unsafe extern "C" fn(runtime: Handle, event: *const u8, event_len: usize) -> Status;

/// Stops and destroys one application instance.
pub type ApplicationStopFn = unsafe extern "C" fn(runtime: Handle) -> Status;

/// Reports whether the application registered a named utility service.
pub type UtilityIsRegisteredFn = unsafe extern "C" fn(name: *const u8, name_len: usize) -> Status;

/// Receives one asynchronous utility-service response exactly once.
pub type UtilityReplyFn =
    unsafe extern "C" fn(user_data: *mut c_void, status: Status, payload: ByteSlice) -> Status;

/// Creates one named utility-service instance in a Chromium utility process.
pub type UtilityStartFn =
    unsafe extern "C" fn(name: *const u8, name_len: usize, service_out: *mut Handle) -> Status;

/// Invokes one utility-service instance with a binary request.
pub type UtilityInvokeFn = unsafe extern "C" fn(
    service: Handle,
    request_id: u64,
    request: *const u8,
    request_len: usize,
    user_data: *mut c_void,
    reply: Option<UtilityReplyFn>,
) -> Status;

/// Requests cooperative cancellation of one utility invocation.
pub type UtilityCancelFn = unsafe extern "C" fn(service: Handle, request_id: u64) -> Status;

/// Stops and destroys one utility-service instance.
pub type UtilityStopFn = unsafe extern "C" fn(service: Handle) -> Status;

/// Delivers one ordered event from a long-lived Utility session.
pub type UtilitySessionEventFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    session_id: u64,
    sequence: u64,
    kind: u32,
    payload: ByteSlice,
) -> Status;

/// Creates one long-lived session in a registered Utility service.
pub type UtilitySessionCreateFn = unsafe extern "C" fn(
    service: Handle,
    session_id: u64,
    init: *const u8,
    init_len: usize,
    initial_event_credits: u32,
    max_event_bytes: usize,
    user_data: *mut c_void,
    event: Option<UtilitySessionEventFn>,
    session_out: *mut Handle,
) -> Status;

/// Sends one asynchronous command to a long-lived Utility session.
pub type UtilitySessionSendFn = unsafe extern "C" fn(
    session: Handle,
    command_id: u64,
    command: *const u8,
    command_len: usize,
    user_data: *mut c_void,
    reply: Option<UtilityReplyFn>,
) -> Status;

/// Acknowledges a durably consumed sequence and replenishes event credits.
pub type UtilitySessionAcknowledgeFn =
    unsafe extern "C" fn(session: Handle, committed_sequence: u64, credits: u32) -> Status;

/// Requests cooperative cancellation of one Utility session command.
pub type UtilitySessionCancelFn = unsafe extern "C" fn(session: Handle, command_id: u64) -> Status;

/// Closes one Utility session and asynchronously returns its bounded summary.
pub type UtilitySessionCloseFn = unsafe extern "C" fn(
    session: Handle,
    reason: u32,
    user_data: *mut c_void,
    reply: Option<UtilityReplyFn>,
) -> Status;

/// Reports completion of an asynchronous Utility-session create operation.
pub type UtilityHostCreateReplyFn = unsafe extern "C" fn(user_data: *mut c_void, status: Status);
/// Delivers one ordered Utility-session event to the trusted Rust Host.
pub type UtilityHostEventFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    sequence: u64,
    kind: u32,
    payload: ByteSlice,
) -> Status;
/// Reports terminal Utility-session closure to the trusted Rust Host.
pub type UtilityHostClosedFn = unsafe extern "C" fn(
    user_data: *mut c_void,
    status: Status,
    last_sequence: u64,
    summary: ByteSlice,
);
/// Opens a supervised Utility session from the main Rust Runtime.
pub type CreateUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    init: *const u8,
    init_len: usize,
    initial_event_credits: u32,
    max_event_bytes: usize,
    session_data: *mut c_void,
    event: Option<UtilityHostEventFn>,
    closed: Option<UtilityHostClosedFn>,
    create_data: *mut c_void,
    created: Option<UtilityHostCreateReplyFn>,
) -> Status;
/// Sends a command from the main Rust Runtime to a Utility session.
pub type SendUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    command_id: u64,
    command: *const u8,
    command_len: usize,
    reply_data: *mut c_void,
    reply: Option<UtilityReplyFn>,
) -> Status;
/// Acknowledges committed events from the main Rust Runtime.
pub type AcknowledgeUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    committed_sequence: u64,
    credits: u32,
) -> Status;
/// Cancels one Utility-session command from the main Rust Runtime.
pub type CancelUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    command_id: u64,
) -> Status;
/// Closes a Utility session from the main Rust Runtime.
pub type CloseUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    reason: u32,
    reply_data: *mut c_void,
    reply: Option<UtilityReplyFn>,
) -> Status;

/// Reports a Shell API result using its RPC status domain, distinct from Status.
pub type ShellApiReplyFn =
    unsafe extern "C" fn(user_data: *mut c_void, status: i32, payload: ByteSlice) -> Status;

/// Invokes a generated Shell API endpoint from the native application caller.
pub type ShellApiInvokeFn = unsafe extern "C" fn(
    host_data: *mut c_void,
    channel: *const u8,
    channel_len: usize,
    request: *const u8,
    request_len: usize,
    reply_data: *mut c_void,
    reply: Option<ShellApiReplyFn>,
) -> Status;

/// Browser-process client callbacks routed to supervised remote sessions.
/// The host owns routing and process/session lifecycle; Rust supplies replies.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct UtilitySessionClient {
    /// Size and version of this independently exchanged table.
    pub header: Header,
    /// Native context for session operations, valid until application stop.
    pub user_data: *mut c_void,
    /// Opens a supervised long-lived Utility session.
    pub create: Option<CreateUtilityHostSessionFn>,
    /// Sends one command to a Utility session.
    pub send: Option<SendUtilityHostSessionFn>,
    /// Acknowledges Utility session events and restores credits.
    pub acknowledge: Option<AcknowledgeUtilityHostSessionFn>,
    /// Cancels one active Utility session command.
    pub cancel: Option<CancelUtilityHostSessionFn>,
    /// Closes a Utility session.
    pub close: Option<CloseUtilityHostSessionFn>,
}

impl Default for UtilitySessionClient {
    fn default() -> Self {
        Self {
            header: Header::for_type::<Self>(),
            user_data: std::ptr::null_mut(),
            create: None,
            send: None,
            acknowledge: None,
            cancel: None,
            close: None,
        }
    }
}

/// Stable capability IDs. Query accepts a raw u32 so unknown IDs remain valid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Capability {
    /// Legacy ABI 0.1.0 discovery ID. New clients use `BrowserHost::shell_api`.
    ShellApi = 0,
    /// Legacy ABI 0.1.0 discovery ID. New clients use `BrowserHost::utility`.
    Utility = 1,
    /// Browser-process host capabilities, introduced in ABI 0.1.0.
    Browser = 2,
    /// Framework supports interface-to-utility routing and blocks raw access to typed owners.
    UtilityInterfaces = 3,
}

/// Returns an immutable capability table's header, or null when unsupported.
/// Tables and contexts remain valid until application stop completes. Query is
/// thread-safe, performs no process launch, and never transfers ownership.
pub type QueryExtensionFn =
    unsafe extern "C" fn(user_data: *mut c_void, capability: u32) -> *const Header;

/// Utility-process client facade. This is not the Rust service implementation.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Utility {
    /// Size and version of this table.
    pub header: Header,
    /// Optional separately described session client table.
    pub session: *const UtilitySessionClient,
}

/// Generated native Shell API facade, independent of utility session transport.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct ShellApi {
    /// Size and version of this table.
    pub header: Header,
    /// Native context valid until application stop completes.
    pub user_data: *mut c_void,
    /// Optional generated binary request/reply entry.
    pub invoke: Option<ShellApiInvokeFn>,
}

/// Browser-process native capabilities, discovered through `Capability::Browser`.
///
/// Nested tables are independently headed and borrowed through browser stop.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BrowserHost {
    /// Size/version of this independently exchanged browser capability table.
    pub header: Header,
    /// Optional native Shell API facade.
    pub shell_api: *const ShellApi,
    /// Optional supervised utility client facade.
    pub utility: *const Utility,
}

/// Utility-process service lifecycle and long-lived session operations.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct UtilityService {
    /// Checks explicitly registered Rust utility capability before process launch.
    pub is_registered: Option<UtilityIsRegisteredFn>,
    /// Creates a registered application utility service.
    pub start: Option<UtilityStartFn>,
    /// Handles one binary utility-service request.
    pub invoke: Option<UtilityInvokeFn>,
    /// Cancels one active utility-service request.
    pub cancel: Option<UtilityCancelFn>,
    /// Shuts a utility-service instance down.
    pub stop: Option<UtilityStopFn>,
    /// Immutable service implementation table. Static storage matches the SDK
    /// exporter; the C ABI only requires validity until Framework run returns.
    pub session: &'static UtilitySessionService,
}

/// Utility-process callbacks implemented by the Rust application service.
/// Handles are local to that process and never replace client routing IDs.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct UtilitySessionService {
    /// Size/version of this independent service-side session table.
    pub header: Header,
    /// Opens a long-lived Utility session.
    pub create: Option<UtilitySessionCreateFn>,
    /// Sends a command to a long-lived Utility session.
    pub send: Option<UtilitySessionSendFn>,
    /// Acknowledges committed Utility session events and restores credits.
    pub acknowledge: Option<UtilitySessionAcknowledgeFn>,
    /// Cancels one active Utility session command.
    pub cancel: Option<UtilitySessionCancelFn>,
    /// Closes a long-lived Utility session.
    pub close: Option<UtilitySessionCloseFn>,
}

/// Versioned function table supplied by Tuffite Framework.
#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Host {
    /// Version and size of this complete callback table.
    pub header: Header,
    /// Opaque pointer passed back to callbacks.
    pub user_data: *mut c_void,
    /// Posts one Runtime event to Framework.
    pub post_event: Option<PostEventFn>,
    /// Posts short-lived application work to Chromium's thread pool.
    pub post_task: Option<PostTaskFn>,
    /// Discovers independent optional capability tables.
    pub query_extension: Option<QueryExtensionFn>,
}

/// Synchronous argument sink. It copies the borrowed UTF-8 bytes before returning.
pub type LaunchArgumentFn =
    unsafe extern "C" fn(user_data: *mut c_void, argument: ByteSlice) -> Status;

/// Browser-only argument preparation, available in ABI 0.1.0.
/// Input slices are borrowed; accepted output is copied through the sink.
pub type PrepareBrowserFn = unsafe extern "C" fn(
    args: *const ByteSlice,
    count: usize,
    user_data: *mut c_void,
    emit: Option<LaunchArgumentFn>,
) -> Status;

/// Synchronous argument hook for every native process role, available in ABI 0.1.0.
pub type PrepareProcessFn = unsafe extern "C" fn(
    process_kind: u32,
    args: *const ByteSlice,
    count: usize,
    user_data: *mut c_void,
    emit: Option<LaunchArgumentFn>,
) -> Status;

/// Looks up a registered utility backend: 0 Rust, 1 V8, unknown names reject.
pub type UtilityRuntimeFn =
    unsafe extern "C" fn(name: *const u8, name_len: usize, runtime: *mut u32) -> Status;

/// Asynchronous browser-side V8 resolver. Acceptance owns exactly one reply.
/// Both input buffers are borrowed; source bytes are borrowed during the reply.
/// A `NotFound` reply defers to native app.assets; all other errors fail loading.
pub type ResolveV8Fn = unsafe extern "C" fn(
    runtime: Handle,
    name: ByteSlice,
    url: ByteSlice,
    user_data: *mut c_void,
    reply: Option<UtilityReplyFn>,
) -> Status;

/// ABI 0.1.0 Utility startup with service-local configuration (at most 64 KiB).
/// Input bytes are borrowed; the child copies them before constructing its service.
pub type UtilityStartWithContextFn = unsafe extern "C" fn(
    name: *const u8,
    name_len: usize,
    configuration: ByteSlice,
    service_out: *mut Handle,
) -> Status;

/// Versioned application function table supplied by the Rust application cdylib.
///
/// Framework copies this table before entering Chromium. Function pointers must
/// remain valid until its run function returns.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Application {
    /// Version and size of this complete callback table.
    pub header: Header,
    /// Creates the Rust application state.
    pub start: Option<ApplicationStartFn>,
    /// Handles one binary command envelope.
    pub dispatch: Option<ApplicationDispatchFn>,
    /// Shuts the Rust application down.
    pub stop: Option<ApplicationStopFn>,
    /// Registered utility services and their session operations.
    pub utility: UtilityService,
    /// Optional browser-only startup hook; may be absent in a shorter table.
    pub prepare_browser: Option<PrepareBrowserFn>,
    /// Optional ABI 0.1.0 all-process hook, preferred over the legacy browser hook.
    pub prepare_process: Option<PrepareProcessFn>,
    /// Optional ABI 0.1.0 named runtime lookup. Absent means legacy Rust backend.
    pub utility_runtime: Option<UtilityRuntimeFn>,
    /// ABI 0.1.0 asynchronous named V8 module resolution in the browser process.
    pub resolve_v8: Option<ResolveV8Fn>,
    /// ABI 0.1.0 service startup configuration; absent means legacy empty context.
    pub utility_start_with_context: Option<UtilityStartWithContextFn>,
    /// Optional pure lookup. Returns a borrowed static owner for an interface or typed owner.
    pub resolve_utility_route: Option<ResolveUtilityRouteFn>,
}

/// Resolves typed interface ownership without launching a process. The output
/// borrows application-lifetime storage and must be copied by the caller.
pub type ResolveUtilityRouteFn =
    unsafe extern "C" fn(name: ByteSlice, owner_out: *mut ByteSlice) -> Status;

/// Entrypoint exported by the prebuilt native Chromium Framework library.
pub type RunFn = unsafe extern "C" fn(
    application: *const Application,
    argc: i32,
    argv: *const *const std::ffi::c_char,
) -> i32;

/// ABI version + wire kinds that Rust, the internal Chromium header, and the
/// public installer header must all agree on. Every ABI evolution has to
/// update this module so build-time smokes stay in sync.
pub mod abi {
    use super::{ABI_MAJOR, ABI_MINOR, ABI_PATCH, RuntimeEventKind, TaskPriority};

    /// Canonical event-kind enumerants exposed to C headers.
    pub const RUNTIME_EVENT_KINDS: &[(&str, u32)] = &[
        ("STARTED", RuntimeEventKind::Started as u32),
        (
            "DISPATCH_ACCEPTED",
            RuntimeEventKind::DispatchAccepted as u32,
        ),
        ("STOPPING", RuntimeEventKind::Stopping as u32),
        ("COMMAND_RESPONSE", RuntimeEventKind::CommandResponse as u32),
        (
            "APPLICATION_EVENT",
            RuntimeEventKind::ApplicationEvent as u32,
        ),
    ];

    /// Canonical task priorities exposed to C headers.
    pub const TASK_PRIORITIES: &[(&str, u32)] = &[
        ("BEST_EFFORT", TaskPriority::BestEffort as u32),
        ("USER_VISIBLE", TaskPriority::UserVisible as u32),
        ("USER_BLOCKING", TaskPriority::UserBlocking as u32),
    ];

    /// Canonical ABI version reported by the C header.
    pub const VERSION: (u16, u16, u16) = (ABI_MAJOR, ABI_MINOR, ABI_PATCH);
}

#[cfg(test)]
mod header_parity {
    //! Cross-checks the Rust-side ABI constants against the canonical C header so
    //! divergence is caught at `cargo test` time, not at ninja/link time.

    use super::abi;

    const HEADER: &str = include_str!("../../../framework/public/abi.h");

    fn find_define(source: &str, name: &str) -> Option<String> {
        // Join `\`-continued preprocessor lines before matching so multi-line
        // enumerants defined for column-limited headers still parse.
        let joined = source.replace("\\\r\n", " ").replace("\\\n", " ");
        for line in joined.lines() {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed.strip_prefix("#define ") else {
                continue;
            };
            let Some((token, rest)) = rest.split_once(char::is_whitespace) else {
                continue;
            };
            if token == name {
                return Some(rest.trim().to_owned());
            }
        }
        None
    }

    fn expect_number(source: &str, header: &str, name: &str) -> u64 {
        let raw =
            find_define(source, name).unwrap_or_else(|| panic!("{header}: missing #define {name}"));
        // Common shapes: `1`, `((tuffite_status_t)0)`, `((tuffite_runtime_event_kind_t)5)`.
        // Select the value, not digits embedded in a cast such as uint32_t.
        raw.split(|character: char| !character.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .next_back()
            .unwrap_or_else(|| panic!("{header}: missing number in #define {name}: {raw}"))
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("{header}: cannot parse #define {name}: {raw}"))
    }

    #[test]
    fn abi_version_matches_header() {
        let output = std::env::temp_dir().join(format!("tuffite-abi-{}.h", std::process::id()));
        let status = std::process::Command::new("python3")
            .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/abi.py"))
            .arg("--output")
            .arg(&output)
            .status()
            .expect("generate the native ABI version header");
        assert!(status.success());
        let header = std::fs::read_to_string(&output).unwrap();
        std::fs::remove_file(output).unwrap();
        assert!(HEADER.contains("tuffite/framework/public/abi_version.h"));
        let (major, minor, patch) = abi::VERSION;
        assert_eq!(
            expect_number(&header, "generated", "TUFFITE_FFI_ABI_MAJOR"),
            u64::from(major),
            "canonical header ABI major disagrees with Rust",
        );
        assert_eq!(
            expect_number(&header, "generated", "TUFFITE_FFI_ABI_MINOR"),
            u64::from(minor),
            "canonical header ABI minor disagrees with Rust",
        );
        assert_eq!(
            expect_number(&header, "generated", "TUFFITE_FFI_ABI_PATCH"),
            u64::from(patch),
            "canonical header ABI patch disagrees with Rust",
        );
    }

    #[test]
    fn compatibility_requires_matching_major_and_sufficient_minor() {
        assert!(super::is_compatible((2, 4, 0), (2, 4, 0)));
        assert!(super::is_compatible((2, 5, 0), (2, 4, 0)));
        assert!(super::is_compatible((2, 4, 10), (2, 4, 9)));
        assert!(!super::is_compatible((2, 4, 1), (2, 4, 9)));
        assert!(super::is_compatible((2, 5, 0), (2, 4, 9)));
        assert!(!super::is_compatible((2, 3, 9), (2, 4, 0)));
        assert!(!super::is_compatible((1, 9, 9), (2, 0, 0)));
        assert!(!super::is_compatible((3, 0, 0), (2, 0, 0)));
    }

    #[test]
    fn current_release_satisfies_reset_baseline() {
        assert!(super::Version::CURRENT.is_compatible_with(super::Version::BASELINE));
        assert!(
            !super::Version {
                major: 2,
                minor: 5,
                patch: 0
            }
            .is_compatible_with(super::Version {
                major: 0,
                minor: 1,
                patch: 0
            })
        );
        assert!(!super::is_compatible((0, 0, 9), (0, 1, 0)));
        assert!(super::is_compatible((0, 1, 1), (0, 1, 0)));
    }

    #[test]
    fn tables_share_a_fixed_size_semver_prefix() {
        use super::{Application, Header, Host, Version};
        use std::mem::{offset_of, size_of};

        let size = size_of::<usize>();
        assert_eq!(offset_of!(Host, header), 0);
        assert_eq!(offset_of!(Application, header), 0);
        assert_eq!(offset_of!(Header, struct_size), 0);
        assert_eq!(offset_of!(Header, version), size);
        assert_eq!(offset_of!(Header, reserved), size + 6);
        assert_eq!(offset_of!(Version, major), 0);
        assert_eq!(offset_of!(Version, minor), 2);
        assert_eq!(offset_of!(Version, patch), 4);
        assert_eq!(size_of::<Version>(), 6);
        assert_eq!(size_of::<Header>(), size + 8);
        assert_eq!(offset_of!(Host, user_data), size + 8);
        assert_eq!(offset_of!(Application, start), size + 8);
    }

    #[test]
    fn callback_tables_have_fixed_prefixes() {
        use super::{
            Application, Header, Host, UtilityService, UtilitySessionClient, UtilitySessionService,
        };
        use std::mem::{offset_of, size_of};
        let pointer = size_of::<usize>();
        let header = size_of::<Header>();
        assert_eq!(offset_of!(Host, query_extension), header + 3 * pointer);
        assert_eq!(size_of::<Host>(), header + 4 * pointer);
        assert_eq!(offset_of!(super::Utility, session), header);
        assert_eq!(size_of::<super::Utility>(), header + pointer);
        assert_eq!(offset_of!(UtilitySessionClient, user_data), header);
        assert_eq!(offset_of!(UtilitySessionClient, create), header + pointer);
        assert_eq!(
            offset_of!(UtilitySessionClient, close),
            header + 5 * pointer
        );
        assert_eq!(size_of::<UtilitySessionClient>(), header + 6 * pointer);
        assert_eq!(offset_of!(super::ShellApi, invoke), header + pointer);
        assert_eq!(offset_of!(super::BrowserHost, shell_api), header);
        assert_eq!(offset_of!(super::BrowserHost, utility), header + pointer);
        assert_eq!(size_of::<super::BrowserHost>(), header + 2 * pointer);
        assert_eq!(offset_of!(Application, utility), header + 3 * pointer);
        assert_eq!(offset_of!(UtilityService, session), 5 * pointer);
        assert_eq!(
            offset_of!(Application, prepare_browser),
            header + 9 * pointer
        );
        assert_eq!(
            offset_of!(Application, prepare_process),
            header + 10 * pointer
        );
        assert_eq!(
            offset_of!(Application, utility_runtime),
            header + 11 * pointer
        );
        assert_eq!(
            offset_of!(Application, utility_start_with_context),
            header + 13 * pointer
        );
        assert_eq!(
            offset_of!(Application, resolve_utility_route),
            header + 14 * pointer
        );
        assert_eq!(size_of::<Application>(), header + 15 * pointer);
        assert_eq!(offset_of!(UtilitySessionService, header), 0);
        assert_eq!(offset_of!(UtilitySessionService, create), header);
        assert_eq!(
            offset_of!(UtilitySessionService, close),
            header + 4 * pointer
        );
        assert_eq!(size_of::<UtilitySessionService>(), header + 5 * pointer);
    }

    #[test]
    fn capability_ids_match_header() {
        for (name, value) in [
            ("UTILITY", super::Capability::Utility),
            ("SHELL_API", super::Capability::ShellApi),
            ("BROWSER", super::Capability::Browser),
            ("UTILITY_INTERFACES", super::Capability::UtilityInterfaces),
        ] {
            assert_eq!(
                expect_number(HEADER, "canonical", &format!("TUFFITE_CAPABILITY_{name}")),
                u64::from(value as u32)
            );
        }
    }

    #[test]
    fn runtime_event_kinds_match_header() {
        for (name, value) in abi::RUNTIME_EVENT_KINDS {
            let symbol = format!("TUFFITE_RUNTIME_EVENT_{name}");
            assert_eq!(
                expect_number(HEADER, "canonical", &symbol),
                u64::from(*value),
                "canonical header {symbol} disagrees with Rust value {value}",
            );
        }
    }

    #[test]
    fn task_priorities_match_header() {
        for (name, value) in abi::TASK_PRIORITIES {
            let symbol = format!("TUFFITE_TASK_PRIORITY_{name}");
            assert_eq!(
                expect_number(HEADER, "canonical", &symbol),
                u64::from(*value),
                "canonical header {symbol} disagrees with Rust value {value}",
            );
        }
    }
}
