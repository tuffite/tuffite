//! Exercise V8 imports, restricted authority, cancellation and session teardown.
use super::PlaygroundBrowser;
use std::{sync::Arc, thread, time::Duration};
use tuffite::{CommandError, JavaScriptExtensionSessionHandle};

#[allow(clippy::too_many_lines)]
pub(super) fn check(browser: &PlaygroundBrowser) -> Result<String, CommandError> {
    let mut current = browser
        .javascript_extension_session
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if current
        .as_ref()
        .is_some_and(JavaScriptExtensionSessionHandle::is_active)
    {
        return Err(CommandError::Failed(
            "JavaScript extension smoke is already running".to_owned(),
        ));
    }
    let session_slot = Arc::clone(&browser.javascript_extension_session);
    let open_slot = Arc::clone(&session_slot);
    let close_slot = Arc::clone(&session_slot);
    let events = browser.events.clone();
    let session = browser
        .utilities
        .create_v8(
            "parser",
            move |_| {
                *close_slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            },
            move |opened| {
                let handle = open_slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                let Some(handle) = handle else {
                    let _ = events.emit("playground:javascript-extension", b"error");
                    return;
                };
                if opened.is_err() {
                    let _ = events.emit("playground:javascript-extension", b"error");
                    return;
                }
                let close_handle = handle.clone();
                let result_events = events.clone();
                if handle
                    .call("describe", &[], move |result| {
                        let valid = result.is_ok_and(|value| {
                            value.get("id").and_then(serde_json::Value::as_str)
                                == Some("io.tuffite.playground")
                                && value
                                    .get("staticImport")
                                    .and_then(serde_json::Value::as_bool)
                                    == Some(true)
                                && value
                                    .get("cycleImport")
                                    .and_then(serde_json::Value::as_bool)
                                    == Some(true)
                                && value
                                    .get("standardUrls")
                                    .and_then(serde_json::Value::as_bool)
                                    == Some(true)
                                && value
                                    .get("sharedState")
                                    .and_then(serde_json::Value::as_bool)
                                    == Some(true)
                                && value
                                    .get("sharedImport")
                                    .and_then(serde_json::Value::as_bool)
                                    == Some(true)
                                && value
                                    .get("evalBlocked")
                                    .and_then(serde_json::Value::as_bool)
                                    == Some(true)
                                && value
                                    .get("ambientAuthority")
                                    .and_then(|authority| authority.get("process"))
                                    .and_then(serde_json::Value::as_str)
                                    == Some("undefined")
                                && value
                                    .get("ambientAuthority")
                                    .and_then(|authority| authority.get("require"))
                                    .and_then(serde_json::Value::as_str)
                                    == Some("undefined")
                                && value
                                    .get("ambientAuthority")
                                    .and_then(|authority| authority.get("fetch"))
                                    .and_then(serde_json::Value::as_str)
                                    == Some("undefined")
                        });
                        if !valid {
                            let _ = result_events.emit("playground:javascript-extension", b"error");
                            let _ = close_handle.close(1, |_| {});
                            return;
                        }
                        let cancel_handle = close_handle.clone();
                        let cancelled_close = close_handle.clone();
                        let cancelled_events = result_events.clone();
                        let Ok(command_id) = close_handle.call("spin", &[], move |result| {
                            let _ = cancelled_events.emit(
                                "playground:javascript-extension",
                                if result.is_err() { b"ok" } else { b"error" },
                            );
                            let _ = cancelled_close.close(0, |_| {});
                        }) else {
                            let _ = result_events.emit("playground:javascript-extension", b"error");
                            let _ = close_handle.close(1, |_| {});
                            return;
                        };
                        let _ = thread::Builder::new()
                            .name("playground-js-extension-cancel".to_owned())
                            .spawn(move || {
                                thread::sleep(Duration::from_millis(30));
                                let _ = cancel_handle.cancel(command_id);
                            });
                    })
                    .is_err()
                {
                    let _ = events.emit("playground:javascript-extension", b"error");
                }
            },
        )
        .map_err(|error| CommandError::Failed(error.to_string()))?;
    *current = Some(session);
    Ok("JavaScript extension session started".to_owned())
}
