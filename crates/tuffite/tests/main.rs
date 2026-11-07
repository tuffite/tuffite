//! Public macro integration: explicit registration does not construct process state.

use std::sync::atomic::{AtomicUsize, Ordering};

use tuffite::{Browser, BrowserContext, Result, Utility, UtilityRequest};

static BROWSER_STARTS: AtomicUsize = AtomicUsize::new(0);

struct Backend;
impl Browser for Backend {
    fn new(_: BrowserContext) -> Result<Self> {
        BROWSER_STARTS.fetch_add(1, Ordering::SeqCst);
        Ok(Self)
    }
}

struct Indexer;
impl Utility for Indexer {
    fn setup(
        builder: tuffite::utility::Builder<Self>,
    ) -> tuffite::Result<tuffite::utility::Builder<Self>> {
        Ok(builder.name("indexer"))
    }
    async fn new(_: tuffite::UtilityContext) -> Result<Self> {
        Ok(Self)
    }
    async fn invoke(&self, _: UtilityRequest) -> tuffite::CommandResult {
        Ok(Vec::new())
    }
}

struct Parser;
impl Utility for Parser {
    fn setup(
        builder: tuffite::utility::Builder<Self>,
    ) -> tuffite::Result<tuffite::utility::Builder<Self>> {
        Ok(builder.name("parser"))
    }
    async fn new(_: tuffite::UtilityContext) -> Result<Self> {
        Ok(Self)
    }
    async fn invoke(&self, _: UtilityRequest) -> tuffite::CommandResult {
        Ok(Vec::new())
    }
}

struct V8Parser;
impl tuffite::utility::V8 for V8Parser {
    fn setup(
        builder: tuffite::utility::v8::Builder,
    ) -> tuffite::Result<tuffite::utility::v8::Builder> {
        Ok(builder.name("v8_parser"))
    }
    fn new(_: tuffite::V8Context) -> Result<Self> {
        Ok(Self)
    }

    async fn resolve(
        &self,
        _url: &tuffite::Url,
        _context: tuffite::ResolveContext,
    ) -> Result<tuffite::Resolution> {
        Ok(tuffite::Resolution::Resource(
            tuffite::Resource::bytes(b"export default {ping: () => 'pong'};".to_vec())
                .mime_type("text/javascript"),
        ))
    }
}

struct AssetsParser;
impl tuffite::utility::V8 for AssetsParser {
    fn setup(
        builder: tuffite::utility::v8::Builder,
    ) -> tuffite::Result<tuffite::utility::v8::Builder> {
        Ok(builder.name("assets_parser"))
    }
    fn new(_: tuffite::V8Context) -> Result<Self> {
        Ok(Self)
    }
}

struct App;
impl tuffite::Application for App {
    fn setup(builder: tuffite::app::Builder<Self>) -> tuffite::Result<tuffite::app::Builder<Self>> {
        Ok(builder.configuration(tuffite::configuration::Configuration::new(["test"])))
    }
    fn new(_: tuffite::ApplicationContext) -> Result<Self> {
        unreachable!()
    }
}
#[tuffite::main]
fn entry() -> tuffite::Embedding<App> {
    tuffite::Embedding::new()
        .browser::<Backend>()
        .utility::<Indexer>()
        .utility::<Parser>()
        .v8_utility::<V8Parser>()
        .v8_utility::<AssetsParser>()
}

unsafe extern "C" fn emit(
    _: *mut std::ffi::c_void,
    _: tuffite_abi::ByteSlice,
) -> tuffite_abi::Status {
    tuffite_abi::Status::Ok
}

#[test]
fn main_exports_only_explicit_named_utilities_and_a_stable_table() {
    let table = tuffite_get_application();
    assert!(!table.is_null());
    assert_eq!(table, tuffite_get_application());
    // SAFETY: The getter returns immutable static storage on success.
    let table = unsafe { &*table };
    let args = [tuffite_abi::ByteSlice::from_slice(b"test")];
    // SAFETY: Input bytes and the copying-free sink live through synchronous preparation.
    assert_eq!(
        unsafe {
            table.prepare_process.unwrap()(
                0,
                args.as_ptr(),
                args.len(),
                std::ptr::null_mut(),
                Some(emit),
            )
        },
        tuffite_abi::Status::Ok
    );
    for (name, expected) in [("indexer", 0), ("parser", 0), ("v8_parser", 1)] {
        let mut runtime = u32::MAX;
        // SAFETY: Name is readable and output remains writable for the call.
        assert_eq!(
            unsafe { table.utility_runtime.unwrap()(name.as_ptr(), name.len(), &raw mut runtime) },
            tuffite_abi::Status::Ok
        );
        assert_eq!(runtime, expected);
    }
    let name = "v8_parser";
    let mut handle = 0;
    // SAFETY: All outputs are writable; V8 must not enter the Rust state factory.
    assert_ne!(
        unsafe { table.utility.start.unwrap()(name.as_ptr(), name.len(), &raw mut handle) },
        tuffite_abi::Status::Ok
    );
    assert_eq!(handle, 0);
    for name in ["indexer", "parser"] {
        let mut handle = 0;
        // SAFETY: Service name and output are live for these synchronous calls.
        let status =
            unsafe { table.utility.start.unwrap()(name.as_ptr(), name.len(), &raw mut handle) };
        assert_eq!(status, tuffite_abi::Status::Ok);
        assert_ne!(handle, 0);
        // SAFETY: This test owns the live service and has no outstanding work.
        assert_eq!(
            unsafe { table.utility.stop.unwrap()(handle) },
            tuffite_abi::Status::Ok
        );
    }
    let name = b"unknown";
    // SAFETY: The static name is valid for this call.
    assert_eq!(
        unsafe { table.utility.is_registered.unwrap()(name.as_ptr(), name.len()) },
        tuffite_abi::Status::InvalidArgument
    );
    assert_eq!(BROWSER_STARTS.load(Ordering::SeqCst), 0);
}
