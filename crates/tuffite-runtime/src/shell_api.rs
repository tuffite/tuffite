//! Generated binary Shell API calls from the trusted application control plane.

use std::ffi::c_void;

use tuffite_abi::{ByteSlice, Status};

use crate::codec::{Decode, Decoder, Encoder};

const MAX_REQUEST_BYTES: usize = 16 * 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 32 * 1024 * 1024;

/// Generated endpoint request with the same argument/result codecs as renderer calls.
pub trait ShellApiRequest {
    type Response: Decode + Send + 'static;
    const CHANNEL: &'static str;
    fn encode(&self, encoder: &mut Encoder) -> Result<(), crate::codec::CodecError>;
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ShellApiError {
    #[error("native Shell API is unavailable")]
    Unavailable,
    #[error("invalid Shell API request")]
    InvalidArgument,
    #[error("Shell API caller is not authorized")]
    AccessDenied,
    #[error("Shell API operation was cancelled")]
    Cancelled,
    #[error("Shell API operation timed out")]
    TimedOut,
    #[error("Shell API resource limit exceeded")]
    ResourceExhausted,
    #[error("invalid binary Shell API response")]
    InvalidResponse,
    #[error("native Shell API failed")]
    Internal,
}

/// Thread-safe native application client for generated Shell API requests.
#[derive(Clone, Debug)]
pub struct ShellApiHost {
    callbacks: Option<tuffite_abi::ShellApi>,
}

// SAFETY: Startup validates this capability and accepts the provider's callback
// and own-context thread-safety/lifetime contract through application stop.
unsafe impl Send for ShellApiHost {}
// SAFETY: The same accepted contract permits concurrent calls.
unsafe impl Sync for ShellApiHost {}

impl ShellApiHost {
    pub(crate) const fn new(callbacks: Option<tuffite_abi::ShellApi>) -> Self {
        Self { callbacks }
    }

    #[must_use]
    pub const fn is_available(&self) -> bool {
        match self.callbacks {
            Some(table) => table.invoke.is_some(),
            None => false,
        }
    }

    /// Calls a generated endpoint. Successful acceptance transfers reply ownership
    /// to Framework; its eventual RPC result is delivered through `reply`.
    pub fn invoke<R, F>(&self, request: &R, reply: F) -> Result<(), ShellApiError>
    where
        R: ShellApiRequest,
        F: FnOnce(Result<R::Response, ShellApiError>) + Send + 'static,
    {
        let table = self.callbacks.ok_or(ShellApiError::Unavailable)?;
        let invoke = table.invoke.ok_or(ShellApiError::Unavailable)?;
        let mut encoder = Encoder::new();
        request
            .encode(&mut encoder)
            .map_err(|_| ShellApiError::InvalidArgument)?;
        let bytes = encoder.finish();
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(ShellApiError::InvalidArgument);
        }
        let context = Box::into_raw(Box::new(ReplyContext::<R::Response> {
            reply: Box::new(reply),
        }))
        .cast::<c_void>();
        // SAFETY: Generated channel and encoded bytes live through the call.
        // Only acceptance transfers context; Framework copies before returning.
        let status = unsafe {
            invoke(
                table.user_data,
                R::CHANNEL.as_ptr(),
                R::CHANNEL.len(),
                bytes.as_ptr(),
                bytes.len(),
                context,
                Some(on_reply::<R::Response>),
            )
        };
        if status != Status::Ok {
            // SAFETY: Rejection retains the allocation without invoking the callback.
            drop(unsafe { Box::from_raw(context.cast::<ReplyContext<R::Response>>()) });
            return Err(match status {
                Status::InvalidArgument => ShellApiError::InvalidArgument,
                Status::InvalidState => ShellApiError::Unavailable,
                Status::ResourceExhausted => ShellApiError::ResourceExhausted,
                _ => ShellApiError::Internal,
            });
        }
        Ok(())
    }
}

struct ReplyContext<T> {
    reply: Box<dyn FnOnce(Result<T, ShellApiError>) + Send>,
}

unsafe extern "C" fn on_reply<T: Decode>(
    context: *mut c_void,
    status: i32,
    payload: ByteSlice,
) -> Status {
    if context.is_null() {
        return Status::InvalidArgument;
    }
    // SAFETY: Framework owns exactly one accepted reply allocation until completion.
    let context = unsafe { Box::from_raw(context.cast::<ReplyContext<T>>()) };
    let result = match status {
        0 if payload.len <= MAX_RESPONSE_BYTES && (payload.len == 0 || !payload.data.is_null()) => {
            // SAFETY: The callback contract borrows readable bytes during this call.
            let bytes = if payload.len == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(payload.data, payload.len) }
            };
            let mut decoder = Decoder::new(bytes);
            T::decode(&mut decoder)
                .and_then(|value| {
                    decoder.finish()?;
                    Ok(value)
                })
                .map_err(|_| ShellApiError::InvalidResponse)
        }
        0 => Err(ShellApiError::InvalidResponse),
        1 => Err(ShellApiError::InvalidArgument),
        2 => Err(ShellApiError::AccessDenied),
        3 => Err(ShellApiError::Unavailable),
        4 => Err(ShellApiError::Cancelled),
        5 => Err(ShellApiError::TimedOut),
        6 => Err(ShellApiError::ResourceExhausted),
        _ => Err(ShellApiError::Internal),
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (context.reply)(result))) {
        Ok(()) => Status::Ok,
        Err(_) => Status::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::Encode;
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    };

    struct Request(i64);
    impl ShellApiRequest for Request {
        type Response = i64;
        const CHANNEL: &'static str = "tuffite.test.double";
        fn encode(&self, encoder: &mut Encoder) -> Result<(), crate::codec::CodecError> {
            self.0.encode(encoder)
        }
    }
    struct Provider {
        rpc_status: i32,
        trailing: bool,
        reject: bool,
    }
    unsafe extern "C" fn invoke(
        data: *mut c_void,
        channel: *const u8,
        channel_len: usize,
        request: *const u8,
        request_len: usize,
        reply_data: *mut c_void,
        reply: Option<tuffite_abi::ShellApiReplyFn>,
    ) -> Status {
        // SAFETY: Tests keep the provider and borrowed call inputs alive.
        let provider = unsafe { &*data.cast::<Provider>() };
        assert_eq!(
            unsafe { std::slice::from_raw_parts(channel, channel_len) },
            Request::CHANNEL.as_bytes()
        );
        let input = unsafe { std::slice::from_raw_parts(request, request_len) };
        let value = i64::decode(&mut Decoder::new(input)).unwrap();
        if provider.reject {
            return Status::ResourceExhausted;
        }
        let mut encoded = Encoder::new();
        (value * 2).encode(&mut encoded).unwrap();
        let mut bytes = encoded.finish();
        if provider.trailing {
            bytes.push(0);
        }
        // SAFETY: Acceptance transfers the context, completed exactly once here.
        assert_eq!(
            unsafe {
                reply.unwrap()(
                    reply_data,
                    provider.rpc_status,
                    ByteSlice {
                        data: bytes.as_ptr(),
                        len: bytes.len(),
                    },
                )
            },
            Status::Ok
        );
        Status::Ok
    }
    fn host(provider: &mut Provider) -> ShellApiHost {
        ShellApiHost::new(Some(tuffite_abi::ShellApi {
            header: tuffite_abi::Header::for_type::<tuffite_abi::ShellApi>(),
            user_data: std::ptr::from_mut(provider).cast(),
            invoke: Some(invoke),
        }))
    }
    #[test]
    fn generated_codec_and_distinct_rpc_status_domain() {
        for (status, expected) in [
            (0, Ok(42)),
            (2, Err(ShellApiError::AccessDenied)),
            (3, Err(ShellApiError::Unavailable)),
            (4, Err(ShellApiError::Cancelled)),
        ] {
            let mut provider = Provider {
                rpc_status: status,
                trailing: false,
                reject: false,
            };
            let result = Arc::new(Mutex::new(None));
            let output = result.clone();
            host(&mut provider)
                .invoke(&Request(21), move |value| {
                    *output.lock().unwrap() = Some(value);
                })
                .unwrap();
            assert_eq!(*result.lock().unwrap(), Some(expected));
        }
    }
    #[test]
    fn trailing_response_is_rejected() {
        let mut provider = Provider {
            rpc_status: 0,
            trailing: true,
            reject: false,
        };
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        host(&mut provider)
            .invoke(&Request(1), move |value| {
                *output.lock().unwrap() = Some(value);
            })
            .unwrap();
        assert_eq!(
            *result.lock().unwrap(),
            Some(Err(ShellApiError::InvalidResponse))
        );
    }
    struct DropProbe(Arc<AtomicUsize>);
    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    #[test]
    fn rejection_releases_reply_without_invoking_it() {
        let mut provider = Provider {
            rpc_status: 0,
            trailing: false,
            reject: true,
        };
        let drops = Arc::new(AtomicUsize::new(0));
        let probe = DropProbe(drops.clone());
        assert_eq!(
            host(&mut provider).invoke(&Request(1), move |_| {
                drop(probe);
                panic!("rejected call cannot reply");
            }),
            Err(ShellApiError::ResourceExhausted)
        );
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
