//! Typed access to native profile lifetimes and window/content composition.
use crate::{BrowserContext, Error, Result, StopSignal};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::task::{Poll, Waker};
use tuffite_abi::{ByteSlice, Status};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProfileId(String);
impl ProfileId {
    pub fn new(id: impl Into<String>) -> Result<Self> {
        let id = id.into();
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
        {
            return Err(Error::new("invalid profile ID"));
        }
        Ok(Self(id))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct BrowserContextHandle(pub(crate) u64);
impl BrowserContextHandle {
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
    pub fn from_u64(raw: u64) -> Result<Self> {
        if raw == 0 {
            Err(Error::new("invalid context handle"))
        } else {
            Ok(Self(raw))
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ViewHandle(pub(crate) u64);
impl ViewHandle {
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
    pub fn from_u64(raw: u64) -> Result<Self> {
        if raw == 0 {
            Err(Error::new("invalid view handle"))
        } else {
            Ok(Self(raw))
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct WindowId(pub i64);
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileInfo {
    pub id: ProfileId,
    pub name: String,
    pub loaded: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct Client {
    table: Option<tuffite_abi::Contexts>,
    stop: StopSignal,
    native_calls: Arc<RwLock<bool>>,
}
// SAFETY: The headed native capability accepts concurrent submissions and
// retains its table through runtime stop. Admission is guarded on both sides.
unsafe impl Send for Client {}
// SAFETY: Same native contract permits shared callers.
unsafe impl Sync for Client {}
impl Client {
    #[cfg(test)]
    pub(crate) fn for_test(table: tuffite_abi::Contexts) -> Self {
        Self {
            table: Some(table),
            stop: StopSignal(crate::supervisor::CancellationToken::new()),
            native_calls: Arc::new(RwLock::new(true)),
        }
    }

    pub(crate) fn is_available(&self) -> bool {
        self.table.is_some_and(|table| table.apply.is_some())
    }
    pub(crate) fn discover(
        host: crate::abi::NativeHost,
        stop: StopSignal,
        native_calls: Arc<RwLock<bool>>,
    ) -> Self {
        let admission = native_calls
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let table = if !*admission || stop.is_stopping() {
            None
        } else {
            host.0.query_extension.and_then(|query| {
                // SAFETY: Validated native Host provides immutable capability tables.
                let header =
                    unsafe { query(host.0.user_data, tuffite_abi::TUFFITE_CAPABILITY_CONTEXTS) };
                if header.is_null() {
                    return None;
                }
                // SAFETY: A capability always exposes a readable leading Header.
                let h = unsafe { *header };
                if h.struct_size < std::mem::size_of::<tuffite_abi::Contexts>()
                    || !h.version.is_compatible_with(tuffite_abi::Version::BASELINE)
                {
                    return None;
                }
                // SAFETY: Size and ABI version checked above.
                Some(unsafe { *header.cast::<tuffite_abi::Contexts>() })
            })
        };
        drop(admission);
        Self {
            table,
            stop,
            native_calls,
        }
    }
    fn submit(
        &self,
        bytes: &[u8],
        data: *mut c_void,
        reply: tuffite_abi::UtilityReplyFn,
    ) -> Status {
        let admission = self
            .native_calls
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !*admission || self.stop.is_stopping() {
            return Status::InvalidState;
        }
        let Some(table) = self.table else {
            return Status::InvalidState;
        };
        let Some(apply) = table.apply else {
            return Status::InvalidState;
        };
        // SAFETY: Admission pins native tables while submission transfers one
        // completion allocation on acceptance. Native copies request bytes.
        unsafe {
            apply(
                table.user_data,
                ByteSlice::from_slice(bytes),
                data,
                Some(reply),
            )
        }
    }
    fn discard(&self, kind: &str, value: &Value, context: Option<Value>) {
        let mut request = json!({"op":if kind == "shortcut" { "shortcuts.unregister".to_owned() } else { format!("{kind}s.close") }});
        request[if kind == "shortcut" { "id" } else { kind }] = value.clone();
        if let Some(context) = context {
            request["context"] = context;
        }
        let Ok(bytes) = serde_json::to_vec(&request) else {
            return;
        };
        let data = Box::into_raw(Box::new(0_u8)).cast();
        if self.submit(&bytes, data, discarded) != Status::Ok {
            // SAFETY: Rejection retains completion ownership.
            drop(unsafe { Box::from_raw(data.cast::<u8>()) });
        }
    }
    pub(crate) async fn apply(&self, request: Value) -> Result<Value> {
        let bytes = serde_json::to_vec(&request).map_err(|e| Error::new(e.to_string()))?;
        let state = Arc::new(Mutex::new(Pending {
            result: None,
            waker: None,
            alive: true,
        }));
        let receiver = Receiver {
            state: state.clone(),
            client: self.clone(),
            completed: false,
            cleanup: match request["op"].as_str() {
                Some("views.create" | "windows.setContent") => Some("view"),
                Some("windows.create") => Some("window"),
                Some("shortcuts.register") => Some("shortcut"),
                _ => None,
            },
            context: request.get("context").cloned(),
        };
        let data = Box::into_raw(Box::new(state)).cast::<c_void>();
        let status = self.submit(&bytes, data, reply);
        if status != Status::Ok {
            // SAFETY: Rejection retains completion ownership and never replies.
            drop(unsafe { Box::from_raw(data.cast::<Arc<Mutex<Pending>>>()) });
            return Err(Error::new(format!("context request rejected: {status:?}")));
        }
        receiver.await
    }
}
struct Pending {
    result: Option<Result<Value>>,
    waker: Option<Waker>,
    alive: bool,
}
unsafe extern "C" fn reply(data: *mut c_void, status: Status, bytes: ByteSlice) -> Status {
    if data.is_null() {
        return Status::InvalidArgument;
    }
    // SAFETY: Accepted completion owns this allocation exactly once.
    let state = unsafe { Box::from_raw(data.cast::<Arc<Mutex<Pending>>>()) };
    let result = if status != Status::Ok {
        Err(Error::new(format!(
            "native context operation failed: {status:?}"
        )))
    } else if bytes.len > 1024 * 1024 || bytes.data.is_null() {
        Err(Error::new("invalid context response"))
    } else {
        // SAFETY: Native borrows readable bytes for the completion.
        serde_json::from_slice(unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) })
            .map_err(|e| Error::new(e.to_string()))
    };
    let wake = {
        let mut state = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.alive {
            return Status::InvalidState;
        }
        state.result = Some(result);
        state.waker.take()
    };
    if let Some(wake) = wake {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| wake.wake()));
    }
    Status::Ok
}
struct Receiver {
    state: Arc<Mutex<Pending>>,
    client: Client,
    completed: bool,
    cleanup: Option<&'static str>,
    context: Option<Value>,
}
impl std::future::Future for Receiver {
    type Output = Result<Value>;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let result = {
            let mut state = this
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(result) = state.result.take() {
                Some(result)
            } else {
                state.waker = Some(cx.waker().clone());
                None
            }
        };
        if let Some(result) = result {
            this.completed = true;
            Poll::Ready(result)
        } else {
            Poll::Pending
        }
    }
}
impl Drop for Receiver {
    fn drop(&mut self) {
        if self.completed {
            return;
        }
        let result = {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.alive = false;
            state.waker = None;
            state.result.take()
        };
        // If completion won the race with cancellation, release the returned
        // detached view. Otherwise the producer sees the rejected completion
        // and releases it. Neither path can strand a profile reference.
        if let Some(kind) = self.cleanup
            && let Some(Ok(view)) = result
            && view.is_string()
        {
            self.client.discard(kind, &view, self.context.take());
        }
    }
}
unsafe extern "C" fn discarded(data: *mut c_void, _: Status, _: ByteSlice) -> Status {
    if data.is_null() {
        return Status::InvalidArgument;
    }
    // SAFETY: Native owns exactly one accepted fire-and-forget completion.
    drop(unsafe { Box::from_raw(data.cast::<u8>()) });
    Status::Ok
}

fn handle(value: &Value) -> Result<u64> {
    value
        .as_str()
        .and_then(|s| s.parse().ok())
        .filter(|v| *v != 0)
        .ok_or_else(|| Error::new("invalid native handle"))
}
#[derive(Clone, Debug)]
pub struct ProfileManager {
    client: Client,
}
impl ProfileManager {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }
    pub async fn list(&self) -> Result<Vec<ProfileInfo>> {
        serde_json::from_value(self.client.apply(json!({"op":"profiles.list"})).await?)
            .map_err(|e| Error::new(e.to_string()))
    }
    pub async fn create(&self, id: ProfileId, name: impl Into<String>) -> Result<()> {
        self.client
            .apply(json!({"op":"profiles.create", "id":id, "name":name.into()}))
            .await?;
        Ok(())
    }
    pub async fn open(&self, id: &ProfileId) -> Result<BrowserContext> {
        let value = self
            .client
            .apply(json!({"op":"profiles.open", "id":id}))
            .await?;
        crate::abi::context_for_runtime(handle(&value)?)
            .map_err(|_| Error::new("profile context is closed"))
    }
    /// Fails while views still reference the context. Closes the exact loaded
    /// generation; it can never close a later reopening of the same profile.
    pub async fn close(&self, context: BrowserContextHandle) -> Result<()> {
        self.client
            .apply(json!({"op":"profiles.close", "context":context.0.to_string()}))
            .await?;
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Views {
    client: Client,
    context: Arc<AtomicU64>,
}
impl Views {
    pub(crate) fn new(client: Client, context: Arc<AtomicU64>) -> Self {
        Self { client, context }
    }
    /// Creates detached content and waits for successful primary navigation.
    pub async fn create(&self, url: impl Into<String>) -> Result<ViewHandle> {
        let result = self.client.apply(json!({"op":"views.create", "context":self.context.load(Ordering::Acquire).to_string(), "url":url.into()})).await?;
        Ok(ViewHandle(handle(&result)?))
    }
    /// Closes detached content. Attached content belongs to its window.
    pub async fn close(&self, view: ViewHandle) -> Result<()> {
        self.client.apply(json!({"op":"views.close", "view":view.0.to_string(), "context":self.context.load(Ordering::Acquire).to_string()})).await?;
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Windows {
    client: Client,
}
/// Native window container options; content is attached separately.
#[derive(Clone, Debug)]
pub struct WindowOptions {
    pub width: i32,
    pub height: i32,
    pub decorations: bool,
}
impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            decorations: true,
        }
    }
}
impl Windows {
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }
    /// Creates an empty native window, independently of any profile.
    pub async fn create(&self, options: WindowOptions) -> Result<WindowId> {
        let value = self.client.apply(json!({"op":"windows.create", "width":options.width, "height":options.height, "decorations":options.decorations})).await?;
        let id = i64::try_from(handle(&value)?).map_err(|_| Error::new("invalid window ID"))?;
        Ok(WindowId(id))
    }
    pub async fn close(&self, window: WindowId) -> Result<()> {
        self.client
            .apply(json!({"op":"windows.close", "window":window.0.to_string()}))
            .await?;
        Ok(())
    }
    /// Atomically installs already prepared content. The previous view becomes
    /// detached and its handle is returned for reuse or explicit disposal.
    pub async fn set_content(
        &self,
        window: WindowId,
        view: ViewHandle,
    ) -> Result<Option<ViewHandle>> {
        let result = self.client.apply(json!({"op":"windows.setContent", "window":window.0.to_string(), "view":view.0.to_string()})).await?;
        if result.is_null() {
            Ok(None)
        } else {
            Ok(Some(ViewHandle(handle(&result)?)))
        }
    }
    pub async fn close_view(&self, view: ViewHandle) -> Result<()> {
        self.client
            .apply(json!({"op":"views.close", "view":view.0.to_string()}))
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    type Completion = (usize, tuffite_abi::UtilityReplyFn);
    #[derive(Default)]
    struct Native {
        pending: Mutex<Option<Completion>>,
        released: Mutex<Vec<String>>,
        contexts: Mutex<Vec<String>>,
    }
    unsafe extern "C" fn apply(
        native: *mut c_void,
        bytes: ByteSlice,
        data: *mut c_void,
        callback: Option<tuffite_abi::UtilityReplyFn>,
    ) -> Status {
        // SAFETY: Each test pins Native and borrows valid request bytes.
        let native = unsafe { &*native.cast::<Native>() };
        let request: Value =
            serde_json::from_slice(unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) })
                .unwrap();
        let callback = callback.unwrap();
        if let Some(context) = request.get("context").and_then(Value::as_str) {
            native.contexts.lock().unwrap().push(context.to_owned());
        }
        if request["op"] == "views.close" || request["op"] == "shortcuts.unregister" {
            native.released.lock().unwrap().push(
                request
                    .get("view")
                    .or_else(|| request.get("id"))
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned(),
            );
            // SAFETY: Accepted test operation completes its owned callback once.
            unsafe {
                callback(data, Status::Ok, ByteSlice::from_slice(b"null"));
            }
        } else {
            *native.pending.lock().unwrap() = Some((data as usize, callback));
        }
        Status::Ok
    }
    fn client(native: &mut Native) -> Client {
        Client {
            table: Some(tuffite_abi::Contexts {
                header: tuffite_abi::Header::for_type::<tuffite_abi::Contexts>(),
                user_data: std::ptr::from_mut(native).cast(),
                apply: Some(apply),
            }),
            stop: StopSignal(crate::supervisor::CancellationToken::new()),
            native_calls: Arc::new(RwLock::new(true)),
        }
    }
    #[test]
    fn cancelled_view_preparation_rejects_a_late_completion() {
        let mut native = Box::<Native>::default();
        let client = client(&mut native);
        let mut future = Box::pin(
            client.apply(json!({"op":"views.create", "context":"7", "url":"about:blank"})),
        );
        assert!(
            future
                .as_mut()
                .poll(&mut std::task::Context::from_waker(Waker::noop()))
                .is_pending()
        );
        drop(future);
        let (data, callback) = native.pending.lock().unwrap().take().unwrap();
        // SAFETY: The retained completion still owns its allocation after cancel.
        assert_eq!(
            unsafe {
                callback(
                    data as *mut c_void,
                    Status::Ok,
                    ByteSlice::from_slice(b"\"42\""),
                )
            },
            Status::InvalidState
        );
        assert!(native.released.lock().unwrap().is_empty());
    }
    #[test]
    fn cancellation_after_completion_releases_the_unconsumed_view() {
        let mut native = Box::<Native>::default();
        let client = client(&mut native);
        let mut future = Box::pin(
            client.apply(json!({"op":"views.create", "context":"7", "url":"about:blank"})),
        );
        assert!(
            future
                .as_mut()
                .poll(&mut std::task::Context::from_waker(Waker::noop()))
                .is_pending()
        );
        let (data, callback) = native.pending.lock().unwrap().take().unwrap();
        // SAFETY: Exactly one accepted completion supplies valid JSON bytes.
        assert_eq!(
            unsafe {
                callback(
                    data as *mut c_void,
                    Status::Ok,
                    ByteSlice::from_slice(b"\"42\""),
                )
            },
            Status::Ok
        );
        drop(future);
        assert_eq!(*native.released.lock().unwrap(), ["42"]);
    }
    #[test]
    fn cancelled_registration_releases_a_completed_native_shortcut() {
        let mut native = Box::<Native>::default();
        let client = client(&mut native);
        let mut future = Box::pin(client.apply(json!({"op":"shortcuts.register","id":"0/80/4"})));
        assert!(
            future
                .as_mut()
                .poll(&mut std::task::Context::from_waker(Waker::noop()))
                .is_pending()
        );
        let (data, callback) = native.pending.lock().unwrap().take().unwrap();
        // SAFETY: The pending completion is owned once and borrows valid JSON bytes.
        assert_eq!(
            unsafe {
                callback(
                    data as *mut c_void,
                    Status::Ok,
                    ByteSlice::from_slice(br#""0/80/4""#),
                )
            },
            Status::Ok
        );
        drop(future);
        assert_eq!(*native.released.lock().unwrap(), ["0/80/4"]);
    }
    #[test]
    fn closed_clients_never_submit_to_retired_native_tables() {
        let mut native = Box::<Native>::default();
        let client = client(&mut native);
        *client.native_calls.write().unwrap() = false;
        assert!(futures_executor::block_on(client.apply(json!({"op":"profiles.list"}))).is_err());
        assert!(native.pending.lock().unwrap().is_none());
    }
    #[test]
    fn view_clients_captured_during_construction_use_the_published_generation() {
        let mut native = Box::<Native>::default();
        let identity = Arc::new(AtomicU64::new(0));
        let views = Views::new(client(&mut native), identity.clone());
        identity.store(7, Ordering::Release);
        let mut future = Box::pin(views.create("about:blank"));
        assert!(
            future
                .as_mut()
                .poll(&mut std::task::Context::from_waker(Waker::noop()))
                .is_pending()
        );
        assert_eq!(*native.contexts.lock().unwrap(), ["7"]);
        let (data, callback) = native.pending.lock().unwrap().take().unwrap();
        // SAFETY: Exactly one accepted completion still owns this allocation.
        assert_eq!(
            unsafe {
                callback(
                    data as *mut c_void,
                    Status::Ok,
                    ByteSlice::from_slice(b"\"42\""),
                )
            },
            Status::Ok
        );
        assert_eq!(futures_executor::block_on(future).unwrap().as_u64(), 42);
        assert!(native.released.lock().unwrap().is_empty());
    }
}
