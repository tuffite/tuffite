//! Native Remoting capability transport used by the SDK's typed client.
use crate::{Error, Result, StopSignal};
use std::ffi::c_void;
use std::sync::{Arc, RwLock};
use tuffite_abi::{ByteSlice, Status};
#[derive(Clone, Debug)]
pub struct Host {
    table: Option<tuffite_abi::Remoting>,
    stop: StopSignal,
    native_calls: Arc<RwLock<bool>>,
}
// SAFETY: Native tables accept concurrent calls and live until runtime stop.
unsafe impl Send for Host {}
// SAFETY: The same native contract permits shared submissions.
unsafe impl Sync for Host {}
impl Host {
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
                // SAFETY: Admission pins the provider's immutable capability tables.
                let header =
                    unsafe { query(host.0.user_data, tuffite_abi::Capability::Remoting as u32) };
                if header.is_null() {
                    return None;
                }
                // SAFETY: Provider returns at least a readable size prefix.
                unsafe { crate::abi::read_capability::<tuffite_abi::Remoting>(header) }.ok()
            })
        };
        drop(admission);
        Self {
            table,
            stop,
            native_calls,
        }
    }
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.table.is_some() && !self.stop.is_stopping()
    }
    fn call<F>(
        &self,
        method: Option<tuffite_abi::RemotingCallFn>,
        bytes: &[u8],
        reply: F,
    ) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        let admission = self
            .native_calls
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !*admission || self.stop.is_stopping() {
            return Err(Error::new("remoting host is stopped"));
        }
        let table = self
            .table
            .ok_or_else(|| Error::new("native remoting is unavailable"))?;
        let method =
            method.ok_or_else(|| Error::new("native remoting operation is unavailable"))?;
        if bytes.len() > 16 * 1024 * 1024 {
            return Err(Error::new("remoting request is too large"));
        }
        let data = Box::into_raw(Box::new(Reply(Box::new(reply)))).cast();
        // SAFETY: Native copies bytes; only OK admission transfers completion ownership.
        let status = unsafe {
            method(
                table.user_data,
                ByteSlice::from_slice(bytes),
                data,
                Some(on_reply),
            )
        };
        if status != Status::Ok {
            // SAFETY: Rejection retains ownership and never invokes the callback.
            drop(unsafe { Box::from_raw(data.cast::<Reply>()) });
            return Err(Error::new(format!("remoting request rejected: {status:?}")));
        }
        Ok(())
    }
    pub fn capabilities<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.capabilities), bytes, reply)
    }
    pub fn start<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.start), bytes, reply)
    }
    pub fn process_signal<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.process_signal), bytes, reply)
    }
    pub fn take_signals<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.take_signals), bytes, reply)
    }
    pub fn send_input<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.send_input), bytes, reply)
    }
    pub fn inject_input<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.inject_input), bytes, reply)
    }
    pub fn stop<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.stop), bytes, reply)
    }
    pub fn stats<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.stats), bytes, reply)
    }
    pub fn capture<F>(&self, bytes: &[u8], reply: F) -> Result<()>
    where
        F: FnOnce(Result<Vec<u8>>) + Send + 'static,
    {
        self.call(self.table.and_then(|t| t.capture), bytes, reply)
    }
}
struct Reply(Box<dyn FnOnce(Result<Vec<u8>>) + Send>);
unsafe extern "C" fn on_reply(data: *mut c_void, status: i32, bytes: ByteSlice) -> Status {
    if data.is_null() {
        return Status::InvalidArgument;
    }
    // SAFETY: Native transfers exactly one accepted completion.
    let reply = unsafe { Box::from_raw(data.cast::<Reply>()) };
    let result = if status != 0 {
        Err(Error::new(format!("native remoting failed: {status}")))
    } else if bytes.len > 64 * 1024 * 1024 || (bytes.len != 0 && bytes.data.is_null()) {
        Err(Error::new("invalid remoting reply"))
    } else if bytes.len == 0 {
        Ok(Vec::new())
    } else {
        // SAFETY: Native borrows a readable response for the callback duration.
        Ok(unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) }.to_vec())
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (reply.0)(result))) {
        Ok(()) => Status::Ok,
        Err(_) => Status::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn host(method: Option<tuffite_abi::RemotingCallFn>) -> Host {
        Host {
            table: Some(tuffite_abi::Remoting {
                header: tuffite_abi::Header::for_type::<tuffite_abi::Remoting>(),
                user_data: std::ptr::null_mut(),
                capabilities: method,
                start: None,
                process_signal: None,
                take_signals: None,
                send_input: None,
                inject_input: None,
                stop: None,
                stats: None,
                capture: None,
            }),
            stop: StopSignal(crate::supervisor::CancellationToken::new()),
            native_calls: Arc::new(RwLock::new(true)),
        }
    }
    #[test]
    fn accepted_reply_copies_borrowed_bytes_and_contains_panics() {
        unsafe extern "C" fn call(
            _: *mut c_void,
            _: ByteSlice,
            data: *mut c_void,
            reply: Option<tuffite_abi::RemotingReplyFn>,
        ) -> Status {
            let bytes = vec![1, 2, 3];
            // SAFETY: Test completes the accepted owned callback exactly once.
            unsafe {
                reply.unwrap()(data, 0, ByteSlice::from_slice(&bytes));
            }
            Status::Ok
        }
        let replies = Arc::new(AtomicUsize::new(0));
        let count = replies.clone();
        host(Some(call))
            .capabilities(&[], move |result| {
                assert_eq!(result.unwrap(), vec![1, 2, 3]);
                count.fetch_add(1, Ordering::Relaxed);
            })
            .unwrap();
        assert_eq!(replies.load(Ordering::Relaxed), 1);
        host(Some(call))
            .capabilities(&[], |_| panic!("contained callback panic"))
            .unwrap();
    }
    #[test]
    fn rejected_and_stopped_calls_release_reply_without_invoking_it() {
        unsafe extern "C" fn reject(
            _: *mut c_void,
            _: ByteSlice,
            _: *mut c_void,
            _: Option<tuffite_abi::RemotingReplyFn>,
        ) -> Status {
            Status::InvalidState
        }
        struct Dropped(Arc<AtomicUsize>);
        impl Drop for Dropped {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }
        let count = Arc::new(AtomicUsize::new(0));
        let dropped = Dropped(count.clone());
        let host = host(Some(reject));
        assert!(
            host.capabilities(&[], move |_| {
                let _ = &dropped;
                panic!("rejected callback ran")
            })
            .is_err()
        );
        assert_eq!(count.load(Ordering::Relaxed), 1);
        *host.native_calls.write().unwrap() = false;
        assert!(
            host.capabilities(&[], |_| panic!("stopped callback ran"))
                .is_err()
        );
        assert!(
            host.start(&[], |_| panic!("missing operation ran"))
                .is_err()
        );
    }
    #[test]
    fn rpc_error_is_independent_of_abi_admission_status() {
        unsafe extern "C" fn call(
            _: *mut c_void,
            _: ByteSlice,
            data: *mut c_void,
            reply: Option<tuffite_abi::RemotingReplyFn>,
        ) -> Status {
            // SAFETY: Test completes the accepted owned callback exactly once.
            unsafe {
                reply.unwrap()(data, 2, ByteSlice::from_slice(&[]));
            }
            Status::Ok
        }
        host(Some(call))
            .capabilities(&[], |result| {
                assert!(result.unwrap_err().to_string().contains("failed: 2"));
            })
            .unwrap();
    }
}
