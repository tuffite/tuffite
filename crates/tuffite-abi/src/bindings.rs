pub const TUFFITE_STATUS_OK: u32 = 0;
pub const TUFFITE_STATUS_INVALID_ARGUMENT: u32 = 1;
pub const TUFFITE_STATUS_INCOMPATIBLE_ABI: u32 = 2;
pub const TUFFITE_STATUS_INVALID_HANDLE: u32 = 3;
pub const TUFFITE_STATUS_INVALID_STATE: u32 = 4;
pub const TUFFITE_STATUS_RESOURCE_EXHAUSTED: u32 = 5;
pub const TUFFITE_STATUS_INTERNAL: u32 = 6;
pub const TUFFITE_STATUS_NOT_FOUND: u32 = 7;
pub const TUFFITE_RUNTIME_EVENT_STARTED: u32 = 1;
pub const TUFFITE_RUNTIME_EVENT_DISPATCH_ACCEPTED: u32 = 2;
pub const TUFFITE_RUNTIME_EVENT_STOPPING: u32 = 3;
pub const TUFFITE_RUNTIME_EVENT_COMMAND_RESPONSE: u32 = 4;
pub const TUFFITE_RUNTIME_EVENT_APPLICATION_EVENT: u32 = 5;
pub const TUFFITE_TASK_PRIORITY_BEST_EFFORT: u32 = 0;
pub const TUFFITE_TASK_PRIORITY_USER_VISIBLE: u32 = 1;
pub const TUFFITE_TASK_PRIORITY_USER_BLOCKING: u32 = 2;
pub const TUFFITE_CAPABILITY_UTILITY_SESSION: u32 = 0;
pub const TUFFITE_CAPABILITY_UTILITY_INTERFACES: u32 = 1;
pub const TUFFITE_CAPABILITY_ISOLATED: u32 = 2;
pub const TUFFITE_CAPABILITY_CONTEXTS: u32 = 3;
pub const TUFFITE_CAPABILITY_REMOTING: u32 = 4;
pub const TUFFITE_PROCESS_BROWSER: u32 = 0;
pub const TUFFITE_PROCESS_UTILITY: u32 = 1;
pub const TUFFITE_PROCESS_RENDERER: u32 = 2;
pub const TUFFITE_PROCESS_GPU: u32 = 3;
pub const TUFFITE_PROCESS_OTHER: u32 = 4;
pub const TUFFITE_UTILITY_RUNTIME_RUST: u32 = 0;
pub const TUFFITE_UTILITY_RUNTIME_V8: u32 = 1;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Header {
    pub struct_size: usize,
    pub version: Version,
    pub reserved: u16,
}
pub type Handle = u64;
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Status(pub i32);
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ByteSlice {
    pub data: *const u8,
    pub len: usize,
}
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RuntimeEventKind(pub u32);
pub type PostEventFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    runtime: Handle,
    kind: RuntimeEventKind,
    payload: ByteSlice,
) -> Status;
#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TaskPriority(pub u32);
pub type TaskRunFn = unsafe extern "C" fn(task_data: *mut ::std::os::raw::c_void);
pub type TaskDestroyFn = unsafe extern "C" fn(task_data: *mut ::std::os::raw::c_void);
pub type PostTaskFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    priority: TaskPriority,
    may_block: u8,
    task_data: *mut ::std::os::raw::c_void,
    run: ::std::option::Option<TaskRunFn>,
    destroy: ::std::option::Option<TaskDestroyFn>,
) -> Status;
pub type QueryExtensionFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    capability: u32,
) -> *const Header;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Host {
    pub header: Header,
    pub user_data: *mut ::std::os::raw::c_void,
    pub post_event: ::std::option::Option<PostEventFn>,
    pub post_task: ::std::option::Option<PostTaskFn>,
    pub query_extension: ::std::option::Option<QueryExtensionFn>,
}
pub type UtilityReplyFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    status: Status,
    payload: ByteSlice,
) -> Status;
pub type UtilityHostCreateReplyFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    status: Status,
);
pub type UtilityHostEventFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    sequence: u64,
    kind: u32,
    payload: ByteSlice,
) -> Status;
pub type UtilityHostClosedFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    status: Status,
    last_sequence: u64,
    summary: ByteSlice,
);
pub type CreateUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    init: *const u8,
    init_len: usize,
    initial_event_credits: u32,
    max_event_bytes: usize,
    session_data: *mut ::std::os::raw::c_void,
    event: ::std::option::Option<UtilityHostEventFn>,
    closed: ::std::option::Option<UtilityHostClosedFn>,
    create_data: *mut ::std::os::raw::c_void,
    created: ::std::option::Option<UtilityHostCreateReplyFn>,
) -> Status;
pub type SendUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    command_id: u64,
    command: *const u8,
    command_len: usize,
    reply_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
pub type AcknowledgeUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    committed_sequence: u64,
    credits: u32,
) -> Status;
pub type CancelUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    command_id: u64,
) -> Status;
pub type CloseUtilityHostSessionFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    name: *const u8,
    name_len: usize,
    session_id: u64,
    reason: u32,
    reply_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UtilitySessionClient {
    pub header: Header,
    pub user_data: *mut ::std::os::raw::c_void,
    pub create: ::std::option::Option<CreateUtilityHostSessionFn>,
    pub send: ::std::option::Option<SendUtilityHostSessionFn>,
    pub acknowledge: ::std::option::Option<AcknowledgeUtilityHostSessionFn>,
    pub cancel: ::std::option::Option<CancelUtilityHostSessionFn>,
    pub close: ::std::option::Option<CloseUtilityHostSessionFn>,
}
impl ::std::fmt::Debug for UtilitySessionClient {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "UtilitySessionClient {{ header: {:?}, user_data: {:?}, create: FunctionPointer, send: {:?}, acknowledge: {:?}, cancel: {:?}, close: {:?} }}",
            self.header,
            self.user_data,
            self.send,
            self.acknowledge,
            self.cancel,
            self.close,
        )
    }
}
pub type UtilityIsRegisteredFn = unsafe extern "C" fn(
    name: *const u8,
    name_len: usize,
) -> Status;
pub type UtilityStartFn = unsafe extern "C" fn(
    name: *const u8,
    name_len: usize,
    service_out: *mut Handle,
) -> Status;
pub type UtilityInvokeFn = unsafe extern "C" fn(
    service: Handle,
    request_id: u64,
    request: *const u8,
    request_len: usize,
    user_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
pub type UtilityCancelFn = unsafe extern "C" fn(
    service: Handle,
    request_id: u64,
) -> Status;
pub type UtilityStopFn = unsafe extern "C" fn(service: Handle) -> Status;
pub type UtilitySessionEventFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    session_id: u64,
    sequence: u64,
    kind: u32,
    payload: ByteSlice,
) -> Status;
pub type UtilitySessionCreateFn = unsafe extern "C" fn(
    service: Handle,
    session_id: u64,
    init: *const u8,
    init_len: usize,
    initial_event_credits: u32,
    max_event_bytes: usize,
    user_data: *mut ::std::os::raw::c_void,
    event: ::std::option::Option<UtilitySessionEventFn>,
    session_out: *mut Handle,
) -> Status;
pub type UtilitySessionSendFn = unsafe extern "C" fn(
    session: Handle,
    command_id: u64,
    command: *const u8,
    command_len: usize,
    user_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
pub type UtilitySessionAcknowledgeFn = unsafe extern "C" fn(
    session: Handle,
    committed_sequence: u64,
    credits: u32,
) -> Status;
pub type UtilitySessionCancelFn = unsafe extern "C" fn(
    session: Handle,
    command_id: u64,
) -> Status;
pub type UtilitySessionCloseFn = unsafe extern "C" fn(
    session: Handle,
    reason: u32,
    user_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UtilitySessionService {
    pub header: Header,
    pub create: ::std::option::Option<UtilitySessionCreateFn>,
    pub send: ::std::option::Option<UtilitySessionSendFn>,
    pub acknowledge: ::std::option::Option<UtilitySessionAcknowledgeFn>,
    pub cancel: ::std::option::Option<UtilitySessionCancelFn>,
    pub close: ::std::option::Option<UtilitySessionCloseFn>,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UtilityService {
    pub is_registered: ::std::option::Option<UtilityIsRegisteredFn>,
    pub start: ::std::option::Option<UtilityStartFn>,
    pub invoke: ::std::option::Option<UtilityInvokeFn>,
    pub cancel: ::std::option::Option<UtilityCancelFn>,
    pub stop: ::std::option::Option<UtilityStopFn>,
    pub session: *const UtilitySessionService,
}
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MojomMethod {
    pub channel: ByteSlice,
    pub command_id: u64,
    pub schema_fingerprint: u64,
}
pub type ResolveMojomMethodFn = unsafe extern "C" fn(
    interface_name: ByteSlice,
    ordinal: u32,
    method_out: *mut MojomMethod,
) -> Status;
pub type DispatchMojomFn = unsafe extern "C" fn(
    runtime: Handle,
    command_id: u64,
    schema_fingerprint: u64,
    parameters: ByteSlice,
    reply_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
pub type InvokeUtilityMojomFn = unsafe extern "C" fn(
    service: Handle,
    request_id: u64,
    command_id: u64,
    schema_fingerprint: u64,
    parameters: ByteSlice,
    reply_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
pub type ApplicationStartFn = unsafe extern "C" fn(
    host: *const Host,
    config: *const u8,
    config_len: usize,
    runtime_out: *mut Handle,
) -> Status;
pub type ApplicationDispatchFn = unsafe extern "C" fn(
    runtime: Handle,
    event: *const u8,
    event_len: usize,
) -> Status;
pub type ApplicationStopFn = unsafe extern "C" fn(runtime: Handle) -> Status;
pub type LaunchArgumentFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    argument: ByteSlice,
) -> Status;
pub type PrepareBrowserFn = unsafe extern "C" fn(
    args: *const ByteSlice,
    count: usize,
    user_data: *mut ::std::os::raw::c_void,
    emit: ::std::option::Option<LaunchArgumentFn>,
) -> Status;
pub type PrepareProcessFn = unsafe extern "C" fn(
    process_kind: u32,
    args: *const ByteSlice,
    count: usize,
    user_data: *mut ::std::os::raw::c_void,
    emit: ::std::option::Option<LaunchArgumentFn>,
) -> Status;
pub type UtilityRuntimeFn = unsafe extern "C" fn(
    name: *const u8,
    name_len: usize,
    runtime: *mut u32,
) -> Status;
pub type ResolveV8Fn = unsafe extern "C" fn(
    runtime: Handle,
    session_id: u64,
    name: ByteSlice,
    url: ByteSlice,
    user_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
pub type UtilityStartWithContextFn = unsafe extern "C" fn(
    name: *const u8,
    name_len: usize,
    configuration: ByteSlice,
    service_out: *mut Handle,
) -> Status;
pub type ResolveUtilityRouteFn = unsafe extern "C" fn(
    name: ByteSlice,
    owner_out: *mut ByteSlice,
) -> Status;
pub type SetupBrowserFn = unsafe extern "C" fn(
    user_data: *mut ::std::os::raw::c_void,
    emit: ::std::option::Option<LaunchArgumentFn>,
) -> Status;
pub type InitializeBrowserFn = unsafe extern "C" fn(
    runtime: Handle,
    user_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
pub type CancelV8SessionFn = unsafe extern "C" fn(
    runtime: Handle,
    name: ByteSlice,
    session_id: u64,
) -> Status;
pub type StartProfileFn = unsafe extern "C" fn(
    parent: Handle,
    host: *const Host,
    configuration: ByteSlice,
    profile_id: ByteSlice,
    runtime_out: *mut Handle,
) -> Status;
pub type CommandScopeFn = unsafe extern "C" fn(command_id: u64) -> u32;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Application {
    pub header: Header,
    pub start: ::std::option::Option<ApplicationStartFn>,
    pub dispatch: ::std::option::Option<ApplicationDispatchFn>,
    pub stop: ::std::option::Option<ApplicationStopFn>,
    pub utility: UtilityService,
    pub prepare_browser: ::std::option::Option<PrepareBrowserFn>,
    pub prepare_process: ::std::option::Option<PrepareProcessFn>,
    pub utility_runtime: ::std::option::Option<UtilityRuntimeFn>,
    pub resolve_v8: ::std::option::Option<ResolveV8Fn>,
    pub utility_start_with_context: ::std::option::Option<UtilityStartWithContextFn>,
    pub resolve_utility_route: ::std::option::Option<ResolveUtilityRouteFn>,
    pub setup_browser: ::std::option::Option<SetupBrowserFn>,
    pub initialize_browser: ::std::option::Option<InitializeBrowserFn>,
    pub cancel_v8_session: ::std::option::Option<CancelV8SessionFn>,
    pub start_profile: ::std::option::Option<StartProfileFn>,
    pub command_scope: ::std::option::Option<CommandScopeFn>,
    pub configuration: ::std::option::Option<unsafe extern "C" fn() -> ByteSlice>,
    pub resolve_mojom_method: ::std::option::Option<ResolveMojomMethodFn>,
    pub dispatch_mojom: ::std::option::Option<DispatchMojomFn>,
    pub invoke_utility_mojom: ::std::option::Option<InvokeUtilityMojomFn>,
}
pub type ContextsApplyFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    request: ByteSlice,
    reply_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<UtilityReplyFn>,
) -> Status;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Contexts {
    pub header: Header,
    pub user_data: *mut ::std::os::raw::c_void,
    pub apply: ::std::option::Option<ContextsApplyFn>,
}
pub type IsolatedReplyFn = unsafe extern "C" fn(
    reply_data: *mut ::std::os::raw::c_void,
    status: Status,
    generation: u64,
) -> Status;
pub type IsolatedApplyFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    request: ByteSlice,
    reply_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<IsolatedReplyFn>,
) -> Status;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Isolated {
    pub header: Header,
    pub user_data: *mut ::std::os::raw::c_void,
    pub apply: ::std::option::Option<IsolatedApplyFn>,
}
pub type RemotingReplyFn = unsafe extern "C" fn(
    reply_data: *mut ::std::os::raw::c_void,
    rpc_status: i32,
    response: ByteSlice,
) -> Status;
pub type RemotingCallFn = unsafe extern "C" fn(
    host_data: *mut ::std::os::raw::c_void,
    request: ByteSlice,
    reply_data: *mut ::std::os::raw::c_void,
    reply: ::std::option::Option<RemotingReplyFn>,
) -> Status;
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Remoting {
    pub header: Header,
    pub user_data: *mut ::std::os::raw::c_void,
    pub capabilities: ::std::option::Option<RemotingCallFn>,
    pub start: ::std::option::Option<RemotingCallFn>,
    pub process_signal: ::std::option::Option<RemotingCallFn>,
    pub take_signals: ::std::option::Option<RemotingCallFn>,
    pub send_input: ::std::option::Option<RemotingCallFn>,
    pub inject_input: ::std::option::Option<RemotingCallFn>,
    pub stop: ::std::option::Option<RemotingCallFn>,
    pub stats: ::std::option::Option<RemotingCallFn>,
    pub capture: ::std::option::Option<RemotingCallFn>,
}
