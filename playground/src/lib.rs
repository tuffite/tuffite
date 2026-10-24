//! Playground application runtime loaded by the native Tuffite host.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tuffite::{
    ApplicationResult, Browser, BrowserContext, ChromiumExecutor, CommandError, EventEmitter,
    JavaScriptExtensionSessionHandle, TaskPriority, UtilitySessionHandle, UtilitySessionHost,
    UtilitySessionOptions,
};

mod echo_utility;
mod typed_utility;
mod v8_utility;

mod shell_api {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

struct PlaygroundBrowser {
    events: EventEmitter,
    executor: ChromiumExecutor,
    utility_sessions: UtilitySessionHost,
    utility_session: Arc<Mutex<Option<UtilitySessionHandle>>>,
    utilities: tuffite::utility::Host,
    javascript_extension_session: Arc<Mutex<Option<JavaScriptExtensionSessionHandle>>>,
}

#[tuffite::browser(interfaces(shell_api::Commands, shell_api::Diagnostics))]
impl Browser for PlaygroundBrowser {
    fn new(context: BrowserContext) -> ApplicationResult<Self> {
        context.set_state(std::sync::Arc::new(v8_utility::ParserState {
            fixture_id: "io.tuffite.playground".to_owned(),
        }));
        init_tracing();
        Ok(Self {
            events: context.events(),
            executor: context.executor(),
            utility_sessions: context.utility_sessions(),
            utility_session: Arc::new(Mutex::new(None)),
            utilities: context.utilities(),
            javascript_extension_session: Arc::new(Mutex::new(None)),
        })
    }

    fn shutdown(&mut self) -> ApplicationResult<()> {
        tracing::info!("playground application shutdown");
        Ok(())
    }
}

#[tuffite::main(browser = PlaygroundBrowser, utilities(echo_utility::EchoUtility, v8_utility::Parser, typed_utility::Worker))]
fn main() -> tuffite::app::Builder<PlaygroundBrowser> {
    tuffite::app::Builder::<PlaygroundBrowser>::new()
}

impl shell_api::Diagnostics for PlaygroundBrowser {
    fn process_id(&self) -> Result<u32, tuffite::CommandError> {
        Ok(std::process::id())
    }
}

impl shell_api::Commands for PlaygroundBrowser {
    fn do_something(&self, message: String) -> Result<String, CommandError> {
        do_something(self, &message)
    }
    fn check_utility_session(&self) -> Result<String, CommandError> {
        check_utility_session(self)
    }
    fn check_javascript_extension(&self) -> Result<String, CommandError> {
        check_javascript_extension(self)
    }
    fn echo_mojom(
        &self,
        value: shell_api::MojomSample,
    ) -> Result<shell_api::MojomSample, CommandError> {
        Ok(value)
    }
}

#[allow(clippy::too_many_lines)]
fn check_javascript_extension(app: &PlaygroundBrowser) -> Result<String, CommandError> {
    let mut current = app
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
    let session_slot = Arc::clone(&app.javascript_extension_session);
    let open_slot = Arc::clone(&session_slot);
    let close_slot = Arc::clone(&session_slot);
    let events = app.events.clone();
    let session = app
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

fn do_something(app: &PlaygroundBrowser, message: &str) -> Result<String, CommandError> {
    app.events
        .emit("playground:command", message.as_bytes())
        .map_err(|error| CommandError::Failed(error.to_string()))?;
    let events = app.events.clone();
    app.executor
        .post(TaskPriority::UserVisible, false, move || {
            let _ = events.emit("playground:thread-pool", b"chromium-thread-pool");
        })
        .map_err(|error| CommandError::Failed(error.to_string()))?;
    Ok(format!("Rust received: {message}"))
}

fn check_utility_session(app: &PlaygroundBrowser) -> Result<String, CommandError> {
    let mut current = app
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
    let session_slot = Arc::clone(&app.utility_session);
    let event_slot = Arc::clone(&session_slot);
    let open_slot = Arc::clone(&session_slot);
    let close_slot = Arc::clone(&session_slot);
    let seen = Arc::new(Mutex::new(Vec::<(u64, Vec<u8>)>::new()));
    let event_seen = Arc::clone(&seen);
    let close_seen = Arc::clone(&seen);
    let events = app.events.clone();
    let session = app
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

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .try_init();
}

#[cfg(test)]
mod mojom_tests {
    use super::shell_api::MojomSample;
    use tuffite::__private::mojom::Struct;

    #[test]
    fn matches_official_cpp_fixture_and_rejects_corrupt_data() {
        let hex = include_str!("../../components/dispatch/test/mojom_sample.inc")
            .trim()
            .trim_matches('"');
        let bytes = hex
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let sample = MojomSample::decode_mojom(&bytes).unwrap();
        assert_eq!(sample.large, (1_u64 << 63) + 17);
        assert_eq!(sample.signed_value, -12345);
        assert_eq!(sample.nested.label, "你好");
        assert_eq!(sample.encode_mojom().unwrap(), bytes);
        for length in 0..bytes.len() {
            assert!(MojomSample::decode_mojom(&bytes[..length]).is_err());
        }
        let mut corrupt = bytes.clone();
        corrupt[4] = 1;
        assert!(MojomSample::decode_mojom(&corrupt).is_err());
        let mut corrupt = bytes.clone();
        corrupt[32..40].fill(255);
        assert!(MojomSample::decode_mojom(&corrupt).is_err());
        let mut corrupt = bytes.clone();
        corrupt.extend_from_slice(&[0; 8]);
        assert!(MojomSample::decode_mojom(&corrupt).is_err());
        let mut corrupt = bytes.clone();
        corrupt[48..56].copy_from_slice(&8_u64.to_le_bytes());
        assert!(MojomSample::decode_mojom(&corrupt).is_err());
    }
}
