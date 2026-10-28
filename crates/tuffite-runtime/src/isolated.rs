//! Browser-owned registration of DOM isolated-world script snapshots.
//!
//! The application validates/installs extension packages, then registers the
//! immutable ESM package bytes. Dynamic registration never fetches remote URLs.
use std::ffi::c_void;
use tuffite_abi::{ByteSlice, Status};

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("native isolated registry is unavailable")]
    Unavailable,
    #[error("invalid or stale isolated registration")]
    InvalidArgument,
    #[error("isolated registry resource limit exceeded")]
    ResourceExhausted,
    #[error("native isolated registry failed")]
    Internal,
}
fn status_error(status: i32) -> Error {
    match status {
        1 => Error::InvalidArgument,
        4 => Error::Unavailable,
        5 => Error::ResourceExhausted,
        _ => Error::Internal,
    }
}

/// Immutable ESM package snapshot and exact allowed origins.
#[derive(Clone, Debug)]
pub struct WorldDefinition {
    name: String,
    source: String,
    origins: Vec<String>,
    entry: Option<String>,
    resources: std::collections::BTreeMap<String, String>,
}
impl WorldDefinition {
    #[must_use]
    pub fn new(name: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            origins: Vec::new(),
            entry: None,
            resources: std::collections::BTreeMap::new(),
        }
    }
    /// Defines an ESM entry relative to a package snapshot populated with `resource`.
    #[must_use]
    pub fn package(name: impl Into<String>, entry: impl Into<String>) -> Self {
        Self {
            entry: Some(entry.into()),
            ..Self::new(name, "")
        }
    }
    /// Adds one binary resource. Native validates paths, package size and entry existence.
    #[must_use]
    pub fn resource(mut self, path: impl Into<String>, bytes: impl AsRef<[u8]>) -> Self {
        use base64::Engine as _;
        self.resources.insert(
            path.into(),
            base64::engine::general_purpose::STANDARD.encode(bytes),
        );
        self
    }
    /// Snapshots a validated local package without following symlinks.
    /// Installation, signatures and authorization remain application responsibilities.
    pub fn from_directory(
        name: impl Into<String>,
        directory: impl AsRef<std::path::Path>,
        entry: impl Into<String>,
    ) -> std::io::Result<Self> {
        use std::{fs, io, path::Path};
        fn collect(
            mut definition: WorldDefinition,
            root: &Path,
            path: &Path,
            total: &mut u64,
        ) -> io::Result<WorldDefinition> {
            if fs::symlink_metadata(path)?.file_type().is_symlink() {
                return Err(io::Error::other(
                    "isolated packages cannot contain symlinks",
                ));
            }
            for child in fs::read_dir(path)? {
                let child = child?;
                let metadata = child.file_type()?;
                if metadata.is_symlink() {
                    return Err(io::Error::other(
                        "isolated packages cannot contain symlinks",
                    ));
                }
                if metadata.is_dir() {
                    definition = collect(definition, root, &child.path(), total)?;
                } else if metadata.is_file() {
                    let path = child.path();
                    let size = child.metadata()?.len();
                    *total += size;
                    if size > 8 * 1024 * 1024
                        || *total > 32 * 1024 * 1024
                        || definition.resources.len() >= 1024
                    {
                        return Err(io::Error::other("isolated package exceeds resource limits"));
                    }
                    let bytes = fs::read(&path)?;
                    definition = definition.resource(
                        path.strip_prefix(root)
                            .map_err(io::Error::other)?
                            .to_string_lossy()
                            .replace('\\', "/"),
                        bytes,
                    );
                } else {
                    return Err(io::Error::other("isolated packages require regular files"));
                }
            }
            Ok(definition)
        }
        let root = directory.as_ref();
        collect(Self::package(name, entry), root, root, &mut 0)
    }
    #[must_use]
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.origins.push(origin.into());
        self
    }
}

/// Identifies one committed generation. Clones do not own the registration;
/// dropping this value has no effect. Stale generations cannot replace/remove
/// a newer entry. Already created worlds remain pinned until host disposal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registration {
    name: String,
    generation: u64,
}
impl Registration {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }
}

/// Trusted native application control plane. No registration entry is exposed
/// to renderer JavaScript or isolated worlds. Operations complete on the browser
/// sequence before the callback is invoked; callbacks must not block that thread.
#[derive(Clone, Debug)]
pub struct Host {
    callbacks: Option<tuffite_abi::Isolated>,
}
impl Host {
    pub(crate) const fn new(callbacks: Option<tuffite_abi::Isolated>) -> Self {
        Self { callbacks }
    }
    /// False when the native Framework does not expose this optional capability.
    #[must_use]
    pub const fn is_available(&self) -> bool {
        matches!(self.callbacks, Some(table) if table.apply.is_some())
    }
    fn apply<F>(&self, request: &serde_json::Value, reply: F) -> Result<(), Error>
    where
        F: FnOnce(Result<u64, Error>) + Send + 'static,
    {
        let table = self.callbacks.ok_or(Error::Unavailable)?;
        let apply = table.apply.ok_or(Error::Unavailable)?;
        let bytes = serde_json::to_vec(request).map_err(|_| Error::InvalidArgument)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(Error::InvalidArgument);
        }
        let data = Box::into_raw(Box::new(ReplyContext(Box::new(reply)))).cast::<c_void>();
        // SAFETY: Accepted native tables live through Browser stop, copy bytes
        // before returning, and invoke exactly one reply only on acceptance.
        let status = unsafe {
            apply(
                table.user_data,
                ByteSlice::from_slice(&bytes),
                data,
                Some(on_reply),
            )
        };
        if status != Status::Ok {
            // SAFETY: Rejected calls retain context ownership without replying.
            drop(unsafe { Box::from_raw(data.cast::<ReplyContext>()) });
            return Err(status_error(status.0));
        }
        Ok(())
    }
    pub fn register<F>(&self, definition: WorldDefinition, reply: F) -> Result<(), Error>
    where
        F: FnOnce(Result<Registration, Error>) + Send + 'static,
    {
        self.write("register", definition, None, reply)
    }
    /// Atomically replaces the exact generation. The name must be unchanged.
    /// Bundled manifest entries cannot be replaced through this API.
    pub fn replace<F>(
        &self,
        previous: &Registration,
        definition: WorldDefinition,
        reply: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(Result<Registration, Error>) + Send + 'static,
    {
        if previous.name != definition.name {
            return Err(Error::InvalidArgument);
        }
        self.write("replace", definition, Some(previous.generation), reply)
    }
    fn write<F>(
        &self,
        op: &str,
        definition: WorldDefinition,
        generation: Option<u64>,
        reply: F,
    ) -> Result<(), Error>
    where
        F: FnOnce(Result<Registration, Error>) + Send + 'static,
    {
        if (definition.entry.is_none() && definition.source.is_empty())
            || (definition.entry.is_some() && definition.resources.is_empty())
            || definition.source.len() > 8 * 1024 * 1024
            || definition.origins.is_empty()
            || definition.origins.len() > 32
        {
            return Err(Error::InvalidArgument);
        }
        let name = definition.name;
        let request = serde_json::json!({
            "op": op, "name": name, "source": definition.source,
            "origins": definition.origins, "generation": generation.map(|v| v.to_string())
        });
        let mut request = request;
        if let Some(entry) = definition.entry {
            request["entry"] = entry.into();
            request["resources"] =
                serde_json::to_value(definition.resources).map_err(|_| Error::InvalidArgument)?;
        }
        self.apply(&request, move |result| {
            reply(result.map(|generation| Registration { name, generation }));
        })
    }
    /// Prevents future world creation. Does not terminate existing worlds;
    /// dispose their JavaScript handles and application request handlers first.
    pub fn unregister<F>(&self, registration: &Registration, reply: F) -> Result<(), Error>
    where
        F: FnOnce(Result<(), Error>) + Send + 'static,
    {
        self.apply(
            &serde_json::json!({
                "op": "unregister", "name": registration.name,
                "generation": registration.generation.to_string()
            }),
            move |result| reply(result.map(|_| ())),
        )
    }
}

// SAFETY: Native discovery validates this independently headed table; its
// callbacks/context are thread-safe and remain valid through Browser stop.
unsafe impl Send for Host {}
// SAFETY: The same accepted native contract permits concurrent submissions.
unsafe impl Sync for Host {}
struct ReplyContext(Box<dyn FnOnce(Result<u64, Error>) + Send>);
unsafe extern "C" fn on_reply(data: *mut c_void, status: Status, generation: u64) -> Status {
    if data.is_null() {
        return Status::InvalidArgument;
    }
    // SAFETY: Exactly one accepted completion owns this Box until consumption.
    let context = unsafe { Box::from_raw(data.cast::<ReplyContext>()) };
    let result = if status == Status::Ok {
        Ok(generation)
    } else {
        Err(status_error(status.0))
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (context.0)(result))) {
        Ok(()) => Status::Ok,
        Err(_) => Status::Internal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    fn definition() -> WorldDefinition {
        WorldDefinition::new("card", "source").allow_origin("https://example.com")
    }
    #[test]
    fn absent_capability_is_explicitly_unavailable() {
        let host = Host::new(None);
        assert!(!host.is_available());
        assert_eq!(
            host.register(definition(), |_| panic!("rejected call replied")),
            Err(Error::Unavailable)
        );
    }
    #[test]
    fn typed_reply_preserves_full_generation_and_consumes_callback() {
        unsafe extern "C" fn apply(
            _: *mut c_void,
            request: ByteSlice,
            data: *mut c_void,
            reply: Option<tuffite_abi::IsolatedReplyFn>,
        ) -> Status {
            // SAFETY: Host lends readable serialized request bytes for this call.
            let value: serde_json::Value = serde_json::from_slice(unsafe {
                std::slice::from_raw_parts(request.data, request.len)
            })
            .unwrap();
            assert_eq!(value["entry"], "entry.js");
            assert_eq!(value["resources"]["image.bin"], "AP8=");
            // SAFETY: Caller transfers one live ReplyContext when accepted.
            assert_eq!(
                unsafe { reply.unwrap()(data, Status::Ok, u64::MAX) },
                Status::Ok
            );
            Status::Ok
        }
        let host = Host::new(Some(tuffite_abi::Isolated {
            header: tuffite_abi::Header::for_type::<tuffite_abi::Isolated>(),
            user_data: std::ptr::null_mut(),
            apply: Some(apply),
        }));
        let result = Arc::new(Mutex::new(None));
        let output = result.clone();
        let package = WorldDefinition::package("card", "entry.js")
            .resource("entry.js", b"export {};")
            .resource("image.bin", [0, 255])
            .allow_origin("https://example.com");
        host.register(package, move |value| {
            *output.lock().unwrap() = Some(value);
        })
        .unwrap();
        let registration = result.lock().unwrap().take().unwrap().unwrap();
        assert_eq!(registration.generation(), u64::MAX);
        assert_eq!(Arc::strong_count(&result), 1);
    }
    #[test]
    fn rejected_call_reclaims_callback_ownership() {
        unsafe extern "C" fn reject(
            _: *mut c_void,
            _: ByteSlice,
            _: *mut c_void,
            _: Option<tuffite_abi::IsolatedReplyFn>,
        ) -> Status {
            Status::InvalidState
        }
        let host = Host::new(Some(tuffite_abi::Isolated {
            header: tuffite_abi::Header::for_type::<tuffite_abi::Isolated>(),
            user_data: std::ptr::null_mut(),
            apply: Some(reject),
        }));
        let lifetime = Arc::new(());
        let callback_lifetime = lifetime.clone();
        assert_eq!(
            host.register(definition(), move |_| {
                drop(callback_lifetime);
                panic!("rejected call replied");
            }),
            Err(Error::Unavailable)
        );
        assert_eq!(Arc::strong_count(&lifetime), 1);
    }
}
