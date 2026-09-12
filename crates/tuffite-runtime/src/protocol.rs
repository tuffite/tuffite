//! Binary Shell Protocol decoding and application command dispatch.

use std::panic::{AssertUnwindSafe, catch_unwind};

pub const REQUEST_HEADER_LEN: usize = 36;
const RESPONSE_HEADER_LEN: usize = 20;
const FRAME_MAGIC: [u8; 4] = *b"CERM";
const REQUEST_KIND: u16 = 1;
const RESPONSE_KIND: u16 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum ResponseStatus {
    Ok = 0,
    InvalidRequest = 1,
    UnknownCommand = 2,
    HandlerFailed = 3,
    InvalidArgument = 4,
    SchemaMismatch = 5,
}

#[allow(clippy::struct_field_names)]
struct Request<'a> {
    request_id: u64,
    command_id: u64,
    schema_fingerprint: u64,
    payload: &'a [u8],
}

fn decode_request(bytes: &[u8]) -> Option<Request<'_>> {
    if bytes.len() < REQUEST_HEADER_LEN || bytes[..4] != FRAME_MAGIC {
        return None;
    }
    let kind = u16::from_le_bytes(bytes[4..6].try_into().ok()?);
    let flags = u16::from_le_bytes(bytes[6..8].try_into().ok()?);
    let payload_len = usize::try_from(u32::from_le_bytes(bytes[32..36].try_into().ok()?)).ok()?;
    if kind != REQUEST_KIND || flags != 0 || payload_len != bytes.len() - REQUEST_HEADER_LEN {
        return None;
    }
    Some(Request {
        request_id: u64::from_le_bytes(bytes[8..16].try_into().ok()?),
        command_id: u64::from_le_bytes(bytes[16..24].try_into().ok()?),
        schema_fingerprint: u64::from_le_bytes(bytes[24..32].try_into().ok()?),
        payload: &bytes[REQUEST_HEADER_LEN..],
    })
}

fn encode_response(request_id: u64, status: ResponseStatus, payload: &[u8]) -> Vec<u8> {
    let mut response = Vec::with_capacity(RESPONSE_HEADER_LEN + payload.len());
    response.extend_from_slice(&FRAME_MAGIC);
    response.extend_from_slice(&RESPONSE_KIND.to_le_bytes());
    response.extend_from_slice(&(status as u16).to_le_bytes());
    response.extend_from_slice(&request_id.to_le_bytes());
    response.extend_from_slice(
        &u32::try_from(payload.len())
            .unwrap_or(u32::MAX)
            .to_le_bytes(),
    );
    response.extend_from_slice(payload);
    response
}

/// Encodes a scheduler-generated failure for a specific `request_id`.
/// Used when the dispatch scheduler drops a request without executing the
/// application handler (queue timeout, runtime shutdown, etc.).
pub(crate) fn encode_scheduler_failure(request_id: u64, message: &str) -> Vec<u8> {
    encode_response(
        request_id,
        ResponseStatus::HandlerFailed,
        message.as_bytes(),
    )
}

/// Extracts the `request_id` from a request header if the slice looks like a
/// well-formed request frame. Returns `None` for any frame the scheduler
/// should reject before enqueueing.
#[must_use]
pub fn request_request_id(bytes: &[u8]) -> Option<u64> {
    let request = decode_request(bytes)?;
    Some(request.request_id)
}

/// Extracts the `request_id` from a response header if the slice looks like a
/// well-formed response frame. Returned so Framework can correlate async
/// completions to enqueued invocations.
#[must_use]
pub fn response_request_id(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < RESPONSE_HEADER_LEN || bytes[..4] != FRAME_MAGIC {
        return None;
    }
    let payload_len = usize::try_from(u32::from_le_bytes(bytes[16..20].try_into().ok()?)).ok()?;
    if u16::from_le_bytes(bytes[4..6].try_into().ok()?) != RESPONSE_KIND
        || payload_len != bytes.len() - RESPONSE_HEADER_LEN
    {
        return None;
    }
    Some(u64::from_le_bytes(bytes[8..16].try_into().ok()?))
}

pub fn dispatch_with(
    bytes: &[u8],
    handler: impl FnOnce(u64, u64, &[u8]) -> Result<Vec<u8>, CommandError>,
) -> Vec<u8> {
    let Some(request) = decode_request(bytes) else {
        return encode_response(0, ResponseStatus::InvalidRequest, b"invalid request");
    };
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        handler(
            request.command_id,
            request.schema_fingerprint,
            request.payload,
        )
    }));
    match outcome {
        Err(_) => encode_response(
            request.request_id,
            ResponseStatus::HandlerFailed,
            b"command panicked",
        ),
        Ok(Ok(payload)) => encode_response(request.request_id, ResponseStatus::Ok, &payload),
        Ok(Err(CommandError::UnknownCommand)) => encode_response(
            request.request_id,
            ResponseStatus::UnknownCommand,
            b"unknown command",
        ),
        Ok(Err(CommandError::Failed(message))) => encode_response(
            request.request_id,
            ResponseStatus::HandlerFailed,
            message.as_bytes(),
        ),
        Ok(Err(CommandError::InvalidArgument(message))) => encode_response(
            request.request_id,
            ResponseStatus::InvalidArgument,
            message.as_bytes(),
        ),
        Ok(Err(CommandError::SchemaMismatch(message))) => encode_response(
            request.request_id,
            ResponseStatus::SchemaMismatch,
            message.as_bytes(),
        ),
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum CommandError {
    UnknownCommand,
    InvalidArgument(String),
    SchemaMismatch(String),
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(request_id: u64, command_id: u64, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&FRAME_MAGIC);
        bytes.extend_from_slice(&REQUEST_KIND.to_le_bytes());
        bytes.extend_from_slice(&0_u16.to_le_bytes());
        bytes.extend_from_slice(&request_id.to_le_bytes());
        bytes.extend_from_slice(&command_id.to_le_bytes());
        bytes.extend_from_slice(&11_u64.to_le_bytes());
        bytes.extend_from_slice(&u32::try_from(payload.len()).unwrap().to_le_bytes());
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn command_round_trip_is_binary_and_correlated() {
        let response = dispatch_with(
            &request(42, 7, b"hello"),
            |command, fingerprint, payload| {
                assert_eq!(fingerprint, 11);
                (command == 7)
                    .then(|| payload.to_vec())
                    .ok_or(CommandError::UnknownCommand)
            },
        );
        assert_eq!(&response[..4], b"CERM");
        assert_eq!(u64::from_le_bytes(response[8..16].try_into().unwrap()), 42);
        assert_eq!(&response[20..], b"hello");
    }

    #[test]
    fn malformed_request_is_rejected_without_deserializing() {
        let response = dispatch_with(b"not a protocol packet", |_, _, _| {
            Err(CommandError::UnknownCommand)
        });
        assert_eq!(u16::from_le_bytes(response[6..8].try_into().unwrap()), 1);
    }

    #[test]
    fn unknown_and_failed_commands_have_distinct_statuses() {
        let unknown = dispatch_with(&request(1, 99, b""), |_, _, _| {
            Err(CommandError::UnknownCommand)
        });
        assert_eq!(u16::from_le_bytes(unknown[6..8].try_into().unwrap()), 2);

        let failed = dispatch_with(&request(2, 7, b""), |_, _, _| {
            Err(CommandError::Failed("application failure".into()))
        });
        assert_eq!(u16::from_le_bytes(failed[6..8].try_into().unwrap()), 3);
        assert_eq!(&failed[20..], b"application failure");
    }

    #[test]
    fn command_panics_are_contained_as_failed_responses() {
        let response = dispatch_with(&request(44, 7, b""), |_, _, _| panic!("application bug"));
        assert_eq!(u64::from_le_bytes(response[8..16].try_into().unwrap()), 44);
        assert_eq!(u16::from_le_bytes(response[6..8].try_into().unwrap()), 3);
        assert_eq!(&response[20..], b"command panicked");
    }

    #[test]
    fn invalid_kind_flags_and_short_headers_are_rejected() {
        for malformed in [
            vec![],
            request(1, 7, b"")[..35].to_vec(),
            {
                let mut value = request(1, 7, b"");
                value[4] = 2;
                value
            },
            {
                let mut value = request(1, 7, b"payload");
                value[32..36].copy_from_slice(&99_u32.to_le_bytes());
                value
            },
            {
                let mut value = request(1, 7, b"");
                value[6] = 1;
                value
            },
        ] {
            let response = dispatch_with(&malformed, |_, _, _| {
                panic!("malformed requests must not reach handlers")
            });
            assert_eq!(u16::from_le_bytes(response[6..8].try_into().unwrap()), 1);
        }
    }
}
