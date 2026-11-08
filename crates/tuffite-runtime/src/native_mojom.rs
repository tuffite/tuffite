//! In-process adapters for real Mojo interface calls. No serialized RPC envelope.
use crate::{CommandError, CommandResult};
use tuffite_abi::{ByteSlice, Status, UtilityReplyFn};

pub(crate) struct Reply {
    data: usize,
    callback: UtilityReplyFn,
}
// SAFETY: Native acceptance supplies a thread-safe context until the single reply.
unsafe impl Send for Reply {}
impl Reply {
    pub(crate) fn new(data: *mut std::ffi::c_void, callback: UtilityReplyFn) -> Self {
        Self {
            data: data as usize,
            callback,
        }
    }
    pub(crate) fn complete(self, result: CommandResult) {
        let (status, bytes) = match result {
            Ok(bytes) => (Status::Ok, bytes),
            Err(CommandError::UnknownCommand) => {
                (Status::InvalidArgument, b"unknown Mojo method".to_vec())
            }
            Err(CommandError::InvalidArgument(message) | CommandError::SchemaMismatch(message)) => {
                (Status::InvalidArgument, message.into_bytes())
            }
            Err(CommandError::Failed(message)) => (Status::InvalidState, message.into_bytes()),
        };
        // SAFETY: Reply context is owned by the accepted operation until this call.
        unsafe {
            (self.callback)(self.data as *mut _, status, ByteSlice::from_slice(&bytes));
        }
    }
}
