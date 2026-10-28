//! Exercise ordered events, credit acknowledgement and Utility session lifetime.
use super::PlaygroundBrowser;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use tuffite::{CommandError, UtilitySessionHandle, UtilitySessionOptions};

pub(super) fn check(browser: &PlaygroundBrowser) -> Result<String, CommandError> {
    let mut current = browser
        .utility_session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if current
        .as_ref()
        .is_some_and(UtilitySessionHandle::is_active)
    {
        return Err(CommandError::Failed(
            "Utility session smoke is already running".to_owned(),
        ));
    }
    let session_slot = Arc::clone(&browser.utility_session);
    let event_slot = Arc::clone(&session_slot);
    let open_slot = Arc::clone(&session_slot);
    let close_slot = Arc::clone(&session_slot);
    let seen = Arc::new(Mutex::new(Vec::<(u64, Vec<u8>)>::new()));
    let event_seen = Arc::clone(&seen);
    let close_seen = Arc::clone(&seen);
    let events = browser.events.clone();
    let session = browser
        .utility_sessions
        .create(
            "echo",
            &UtilitySessionOptions {
                init: b"playground-session-v1".to_vec(),
                initial_event_credits: 1,
                max_event_bytes: 1024,
            },
            move |event| {
                event_seen
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push((event.sequence(), event.payload().to_vec()));
                let handle = event_slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                let Some(handle) = handle else {
                    return false;
                };
                if handle.acknowledge(event.sequence(), 1).is_err() {
                    return false;
                }
                if event.sequence() == 1 {
                    thread::Builder::new()
                        .name("playground-utility-session-idle-test".to_owned())
                        .spawn(move || {
                            // Cross the unary Utility host's 30-second idle/
                            // invocation boundary. A live session must keep its
                            // Utility process and event sink alive.
                            thread::sleep(Duration::from_secs(31));
                            let _ = handle.send(b"second", |_| {});
                        })
                        .is_ok()
                } else if event.sequence() == 2 {
                    handle.close(0, |_| {}).is_ok()
                } else {
                    false
                }
            },
            move |closed| {
                let seen = close_seen
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let valid = closed.status.is_ok()
                    && closed.last_sequence == 2
                    && closed.summary == b"commands=2;reason=0"
                    && seen.as_slice() == [(1, b"first".to_vec()), (2, b"second".to_vec())];
                let _ = events.emit(
                    "playground:utility-session",
                    if valid { b"ok" } else { b"error" },
                );
                *close_slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            },
            move |opened| {
                let handle = open_slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                if opened.is_ok()
                    && let Some(handle) = handle
                {
                    let _ = handle.send(b"first", |_| {});
                }
            },
        )
        .map_err(|error| CommandError::Failed(error.to_string()))?;
    *current = Some(session);
    Ok("Utility session started".to_owned())
}
