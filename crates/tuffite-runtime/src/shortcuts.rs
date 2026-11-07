//! Scoped native shortcut registration. Callbacks run on the application worker.
use crate::{Error, Result, WindowId};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortcutScope {
    Window(WindowId),
    Application,
    System,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortcutState {
    Activated,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Modifiers(u8);
impl Modifiers {
    pub const NONE: Self = Self(0);
    pub const SHIFT: Self = Self(2);
    pub const CONTROL: Self = Self(4);
    pub const ALT: Self = Self(8);
    pub const META: Self = Self(128);
    pub const COMMAND_OR_CONTROL: Self = if cfg!(target_os = "macos") {
        Self::META
    } else {
        Self::CONTROL
    };
}
impl std::ops::BitOr for Modifiers {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Key {
    Character(char),
    Function(u8),
    Enter,
    Escape,
    Space,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Shortcut {
    key: u16,
    modifiers: Modifiers,
}
impl Shortcut {
    pub fn new(modifiers: Modifiers, key: Key) -> Result<Self> {
        let key = match key {
            Key::Character(c) if c.is_ascii_alphanumeric() => {
                u16::try_from(u32::from(c.to_ascii_uppercase()))
                    .map_err(|_| Error::new("unsupported shortcut key"))?
            }
            Key::Function(f) if (1..=24).contains(&f) => 111 + u16::from(f),
            Key::Enter => 13,
            Key::Escape => 27,
            Key::Space => 32,
            Key::Tab => 9,
            Key::Backspace => 8,
            Key::Delete => 46,
            Key::Insert => 45,
            Key::Home => 36,
            Key::End => 35,
            Key::PageUp => 33,
            Key::PageDown => 34,
            Key::ArrowLeft => 37,
            Key::ArrowUp => 38,
            Key::ArrowRight => 39,
            Key::ArrowDown => 40,
            _ => return Err(Error::new("unsupported shortcut key")),
        };
        Ok(Self { key, modifiers })
    }
}
impl TryFrom<&str> for Shortcut {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        let (key, modifiers) = parse(value)?;
        Ok(Self {
            key,
            modifiers: Modifiers(modifiers),
        })
    }
}
impl TryFrom<String> for Shortcut {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Self::try_from(value.as_str())
    }
}
impl std::fmt::Display for Shortcut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (flag, name) in [
            (Modifiers::CONTROL, "Control"),
            (Modifiers::ALT, "Alt"),
            (Modifiers::SHIFT, "Shift"),
            (Modifiers::META, "Meta"),
        ] {
            if self.modifiers.0 & flag.0 != 0 {
                write!(f, "{name}+")?;
            }
        }
        match self.key {
            48..=57 | 65..=90 => write!(
                f,
                "{}",
                char::from(u8::try_from(self.key).map_err(|_| std::fmt::Error)?)
            ),
            112..=135 => write!(f, "F{}", self.key - 111),
            key => f.write_str(match key {
                13 => "Enter",
                27 => "Escape",
                32 => "Space",
                9 => "Tab",
                8 => "Backspace",
                46 => "Delete",
                45 => "Insert",
                36 => "Home",
                35 => "End",
                33 => "PageUp",
                34 => "PageDown",
                37 => "ArrowLeft",
                38 => "ArrowUp",
                39 => "ArrowRight",
                40 => "ArrowDown",
                _ => return Err(std::fmt::Error),
            }),
        }
    }
}
#[derive(Clone, Debug)]
pub struct ShortcutEvent {
    pub shortcut: Shortcut,
    pub scope: ShortcutScope,
    pub window: Option<WindowId>,
    pub state: ShortcutState,
}
type Callback = Arc<dyn Fn(ShortcutEvent) + Send + Sync>;
#[derive(Clone)]
struct Record {
    shortcut: Shortcut,
    scope: ShortcutScope,
    native_id: String,
    callback: Option<Callback>,
}
#[derive(Clone, Default)]
pub(crate) struct Callbacks(
    Arc<Mutex<HashMap<String, Record>>>,
    Arc<std::sync::atomic::AtomicU64>,
);
impl Callbacks {
    pub(crate) fn clear(&self) {
        let records = std::mem::take(
            &mut *self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        );
        drop(records);
    }
    fn remove(&self, id: &str, native_id: &str) {
        let record = {
            let mut records = self
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if records
                .get(id)
                .is_some_and(|record| record.native_id == native_id)
            {
                records.remove(id)
            } else {
                None
            }
        };
        drop(record);
    }
    fn native_id(&self, id: &str) -> Option<String> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(id)
            .map(|record| record.native_id.clone())
    }
    // Keep the Rust 1.91 MSRV; newer toolchains rename this atomic operation.
    #[allow(deprecated)]
    fn next_id(&self, key: &str) -> Result<String> {
        let generation = self
            .1
            .fetch_update(
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
                |value| value.checked_add(1),
            )
            .map_err(|_| Error::new("shortcut registration identities exhausted"))?;
        Ok(format!("{key}@{generation}"))
    }
}
impl std::fmt::Debug for Callbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ShortcutCallbacks")
    }
}
#[derive(Clone, Debug)]
pub struct Shortcuts {
    client: crate::contexts::Client,
    callbacks: Callbacks,
}
impl Shortcuts {
    pub(crate) fn new(client: crate::contexts::Client, callbacks: Callbacks) -> Self {
        Self { client, callbacks }
    }
    #[must_use]
    pub fn application(&self) -> ScopedShortcuts {
        self.scoped(ShortcutScope::Application)
    }
    #[must_use]
    pub fn window(&self, window: WindowId) -> ScopedShortcuts {
        self.scoped(ShortcutScope::Window(window))
    }
    #[must_use]
    pub fn system(&self) -> ScopedShortcuts {
        self.scoped(ShortcutScope::System)
    }
    fn scoped(&self, scope: ShortcutScope) -> ScopedShortcuts {
        ScopedShortcuts {
            host: self.clone(),
            scope,
        }
    }
    pub(crate) fn deliver(&self, bytes: &[u8]) -> Result<Option<ShortcutEvent>> {
        #[derive(serde::Deserialize)]
        struct NativeEvent {
            id: String,
            window: String,
        }
        let event: NativeEvent =
            serde_json::from_slice(bytes).map_err(|e| Error::new(e.to_string()))?;
        let record = self
            .callbacks
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .find(|(_, record)| record.native_id == event.id)
            .map(|(key, record)| (key.clone(), record.clone()));
        let Some((key, record)) = record else {
            return Ok(None);
        };
        if event.window == "-1" {
            self.callbacks.remove(&key, &event.id);
            return Ok(None);
        }
        let event = ShortcutEvent {
            shortcut: record.shortcut,
            scope: record.scope,
            window: event
                .window
                .parse::<i64>()
                .ok()
                .filter(|id| *id > 0)
                .map(WindowId),
            state: ShortcutState::Activated,
        };
        if let Some(callback) = record.callback {
            callback(event);
            Ok(None)
        } else {
            Ok(Some(event))
        }
    }
}
#[derive(Clone, Debug)]
pub struct ScopedShortcuts {
    host: Shortcuts,
    scope: ShortcutScope,
}
impl ScopedShortcuts {
    fn id(&self, shortcut: Shortcut) -> String {
        format!(
            "{}/{}/{}",
            match self.scope {
                ShortcutScope::Application => 0,
                ShortcutScope::System => -1,
                ShortcutScope::Window(id) => id.0,
            },
            shortcut.key,
            shortcut.modifiers.0
        )
    }
    pub async fn register<S: TryInto<Shortcut>>(&self, shortcut: S) -> Result<()>
    where
        S::Error: std::fmt::Display,
    {
        self.register_inner(convert(shortcut)?, None).await
    }
    pub async fn on_shortcut<S: TryInto<Shortcut>>(
        &self,
        shortcut: S,
        callback: impl Fn(ShortcutEvent) + Send + Sync + 'static,
    ) -> Result<()>
    where
        S::Error: std::fmt::Display,
    {
        self.register_inner(convert(shortcut)?, Some(Arc::new(callback)))
            .await
    }
    async fn register_inner(&self, shortcut: Shortcut, callback: Option<Callback>) -> Result<()> {
        struct Rollback {
            callbacks: Callbacks,
            id: String,
            native_id: String,
            armed: bool,
        }
        impl Drop for Rollback {
            fn drop(&mut self) {
                if self.armed {
                    self.callbacks.remove(&self.id, &self.native_id);
                }
            }
        }
        let window = match self.scope {
            ShortcutScope::Window(window) if window.0 > 0 => Some(window.0.to_string()),
            ShortcutScope::Window(_) => return Err(Error::new("invalid shortcut window")),
            _ => None,
        };
        let id = self.id(shortcut);
        let native_id = self.host.callbacks.next_id(&id)?;
        {
            let mut records = self
                .host
                .callbacks
                .0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if records.contains_key(&id) {
                return Err(Error::new("shortcut already registered"));
            }
            records.insert(
                id.clone(),
                Record {
                    shortcut,
                    scope: self.scope,
                    native_id: native_id.clone(),
                    callback,
                },
            );
        }
        let mut rollback = Rollback {
            callbacks: self.host.callbacks.clone(),
            id: id.clone(),
            native_id: native_id.clone(),
            armed: true,
        };
        self.host.client.apply(json!({"op":"shortcuts.register","id":native_id,"key":shortcut.key,"modifiers":shortcut.modifiers.0,"window":window,"system":matches!(self.scope,ShortcutScope::System)})).await?;
        rollback.armed = false;
        Ok(())
    }
    pub async fn unregister<S: TryInto<Shortcut>>(&self, shortcut: S) -> Result<()>
    where
        S::Error: std::fmt::Display,
    {
        let id = self.id(convert(shortcut)?);
        let native_id = self
            .host
            .callbacks
            .native_id(&id)
            .ok_or_else(|| Error::new("shortcut is not registered"))?;
        self.host
            .client
            .apply(json!({"op":"shortcuts.unregister","id":native_id}))
            .await?;
        self.host.callbacks.remove(&id, &native_id);
        Ok(())
    }
    /// Queries this application's native registrations, including closed-window cleanup.
    pub async fn is_registered<S: TryInto<Shortcut>>(&self, shortcut: S) -> Result<bool>
    where
        S::Error: std::fmt::Display,
    {
        let Some(native_id) = self.host.callbacks.native_id(&self.id(convert(shortcut)?)) else {
            return Ok(false);
        };
        self.host
            .client
            .apply(json!({"op":"shortcuts.isRegistered","id":native_id}))
            .await?
            .as_bool()
            .ok_or_else(|| Error::new("invalid shortcut query response"))
    }
    pub async fn unregister_all(&self) -> Result<()> {
        let scope = match self.scope {
            ShortcutScope::Application => 0,
            ShortcutScope::System => -1,
            ShortcutScope::Window(id) => id.0,
        };
        let snapshot: Vec<_> = self
            .host
            .callbacks
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|(_, record)| record.scope == self.scope)
            .map(|(key, record)| (key.clone(), record.native_id.clone()))
            .collect();
        self.host
            .client
            .apply(json!({"op":"shortcuts.unregisterAll","scope":scope.to_string()}))
            .await?;
        for (key, native_id) in snapshot {
            self.host.callbacks.remove(&key, &native_id);
        }
        Ok(())
    }
}
fn convert<S: TryInto<Shortcut>>(value: S) -> Result<Shortcut>
where
    S::Error: std::fmt::Display,
{
    value.try_into().map_err(|e| Error::new(e.to_string()))
}
fn parse(accelerator: &str) -> Result<(u16, u8)> {
    let mut modifiers = 0;
    let mut key = None;
    for part in accelerator.split('+') {
        let modifier = match part {
            "Shift" => Some(2),
            "Ctrl" | "Control" => Some(4),
            "Alt" => Some(8),
            "Cmd" | "Command" | "Meta" | "Super" => Some(128),
            "CmdOrCtrl" | "CmdOrControl" | "CommandOrControl" => {
                Some(if cfg!(target_os = "macos") { 128 } else { 4 })
            }
            _ => None,
        };
        if let Some(value) = modifier {
            if key.is_some() || modifiers & value != 0 {
                return Err(Error::new("invalid shortcut modifiers"));
            }
            modifiers |= value;
            continue;
        }
        let code = match part {
            "Enter" => 13,
            "Escape" => 27,
            "Space" => 32,
            "Tab" => 9,
            "Backspace" => 8,
            "Delete" => 46,
            "Insert" => 45,
            "Home" => 36,
            "End" => 35,
            "PageUp" => 33,
            "PageDown" => 34,
            "ArrowLeft" => 37,
            "ArrowUp" => 38,
            "ArrowRight" => 39,
            "ArrowDown" => 40,
            value if value.len() == 1 && value.as_bytes()[0].is_ascii_alphanumeric() => {
                u16::from(value.as_bytes()[0].to_ascii_uppercase())
            }
            value if value.starts_with('F') => value[1..]
                .parse::<u16>()
                .ok()
                .filter(|v| (1..=24).contains(v))
                .map(|v| 111 + v)
                .ok_or_else(|| Error::new("invalid function key"))?,
            _ => return Err(Error::new("unknown shortcut key")),
        };
        if key.replace(code).is_some() {
            return Err(Error::new("shortcut requires exactly one key"));
        }
    }
    key.map(|key| (key, modifiers))
        .ok_or_else(|| Error::new("shortcut requires a key"))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Native {
        registrations: Mutex<std::collections::BTreeSet<String>>,
        reject: std::sync::atomic::AtomicBool,
    }
    unsafe extern "C" fn apply(
        native: *mut std::ffi::c_void,
        bytes: tuffite_abi::ByteSlice,
        data: *mut std::ffi::c_void,
        reply: Option<tuffite_abi::UtilityReplyFn>,
    ) -> tuffite_abi::Status {
        // SAFETY: The test pins Native and supplies readable bytes until completion.
        let native = unsafe { &*native.cast::<Native>() };
        if native.reject.load(std::sync::atomic::Ordering::Relaxed) {
            return tuffite_abi::Status::InvalidState;
        }
        // SAFETY: Client borrows the complete serialized request for this synchronous call.
        let request: serde_json::Value =
            serde_json::from_slice(unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) })
                .unwrap();
        let id = request["id"].as_str().unwrap_or_default();
        let result = {
            let mut registrations = native.registrations.lock().unwrap();
            match request["op"].as_str().unwrap() {
                "shortcuts.register" => {
                    assert!(registrations.insert(id.into()));
                    serde_json::Value::String(id.into())
                }
                "shortcuts.unregister" => {
                    registrations.remove(id);
                    serde_json::Value::Null
                }
                "shortcuts.unregisterAll" => {
                    let prefix = format!("{}/", request["scope"].as_str().unwrap());
                    registrations.retain(|id| !id.starts_with(&prefix));
                    serde_json::Value::Null
                }
                "shortcuts.isRegistered" => serde_json::Value::Bool(registrations.contains(id)),
                _ => panic!("unexpected shortcut operation"),
            }
        };
        let bytes = serde_json::to_vec(&result).unwrap();
        // SAFETY: Accepted submission owns one live reply and borrows response bytes synchronously.
        unsafe {
            reply.unwrap()(
                data,
                tuffite_abi::Status::Ok,
                tuffite_abi::ByteSlice::from_slice(&bytes),
            );
        }
        tuffite_abi::Status::Ok
    }
    #[test]
    fn scoped_registration_callbacks_queries_and_cleanup_use_the_native_registry() {
        let mut native = Box::<Native>::default();
        let client = crate::contexts::Client::for_test(tuffite_abi::Contexts {
            header: tuffite_abi::Header::for_type::<tuffite_abi::Contexts>(),
            user_data: std::ptr::from_mut(native.as_mut()).cast(),
            apply: Some(apply),
        });
        let host = Shortcuts::new(client, Callbacks::default());
        let called = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let shortcut = Shortcut::try_from("Control+Shift+P").unwrap();
        let keys = host.application();
        futures_executor::block_on(keys.on_shortcut(shortcut, {
            let called = called.clone();
            move |event| {
                assert_eq!(event.state, ShortcutState::Activated);
                called.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }))
        .unwrap();
        assert!(futures_executor::block_on(keys.is_registered(shortcut)).unwrap());
        assert!(futures_executor::block_on(keys.register(shortcut)).is_err());
        let deliver = |id: String, window: &str| {
            host.deliver(&serde_json::to_vec(&json!({"id":id,"window":window})).unwrap())
        };
        assert!(
            deliver(host.callbacks.native_id(&keys.id(shortcut)).unwrap(), "7")
                .unwrap()
                .is_none()
        );
        assert_eq!(called.load(std::sync::atomic::Ordering::Relaxed), 1);
        let window = host.window(WindowId(7));
        futures_executor::block_on(window.register(shortcut)).unwrap();
        assert_eq!(
            deliver(host.callbacks.native_id(&window.id(shortcut)).unwrap(), "7")
                .unwrap()
                .unwrap()
                .scope,
            ShortcutScope::Window(WindowId(7))
        );
        native
            .registrations
            .lock()
            .unwrap()
            .remove(&host.callbacks.native_id(&window.id(shortcut)).unwrap());
        deliver(
            host.callbacks.native_id(&window.id(shortcut)).unwrap(),
            "-1",
        )
        .unwrap();
        assert!(!futures_executor::block_on(window.is_registered(shortcut)).unwrap());
        futures_executor::block_on(keys.unregister_all()).unwrap();
        assert!(!futures_executor::block_on(keys.is_registered(shortcut)).unwrap());
        futures_executor::block_on(keys.register(shortcut)).unwrap();
        let old = host.callbacks.native_id(&keys.id(shortcut)).unwrap();
        futures_executor::block_on(keys.unregister(shortcut)).unwrap();
        futures_executor::block_on(keys.register(shortcut)).unwrap();
        deliver(old, "-1").unwrap();
        assert!(futures_executor::block_on(keys.is_registered(shortcut)).unwrap());
        futures_executor::block_on(keys.unregister_all()).unwrap();
        native
            .reject
            .store(true, std::sync::atomic::Ordering::Relaxed);
        assert!(futures_executor::block_on(keys.on_shortcut(shortcut, |_| {})).is_err());
        assert!(host.callbacks.0.lock().unwrap().is_empty());
    }
    #[test]
    fn parses_modifiers_and_rejects_ambiguous_keys() {
        assert_eq!(parse("Ctrl+Shift+p").unwrap(), (80, 6));
        assert_eq!(parse("Alt+F24").unwrap(), (135, 8));
        for invalid in ["", "Ctrl+", "Ctrl+Ctrl+K", "K+P", "K+Ctrl", "F25"] {
            assert!(parse(invalid).is_err(), "{invalid}");
        }
    }
}
