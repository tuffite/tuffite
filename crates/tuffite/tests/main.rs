//! Public macro integration: explicit registration does not construct process state.

use std::sync::atomic::{AtomicUsize, Ordering};

use tuffite::{ApplicationResult, Browser, BrowserContext, Utility, UtilityRequest};

static BROWSER_STARTS: AtomicUsize = AtomicUsize::new(0);

struct Backend;
#[tuffite::browser]
impl Browser for Backend {
    fn new(_: BrowserContext) -> ApplicationResult<Self> {
        BROWSER_STARTS.fetch_add(1, Ordering::SeqCst);
        Ok(Self)
    }
}

struct Indexer;
#[tuffite::utility(name = "indexer")]
impl Utility for Indexer {
    async fn new(_: tuffite::UtilityContext) -> ApplicationResult<Self> {
        Ok(Self)
    }
    async fn invoke(&self, _: UtilityRequest) -> tuffite::CommandResult {
        Ok(Vec::new())
    }
}

struct Parser;
#[tuffite::utility(name = "parser")]
impl Utility for Parser {
    async fn new(_: tuffite::UtilityContext) -> ApplicationResult<Self> {
        Ok(Self)
    }
    async fn invoke(&self, _: UtilityRequest) -> tuffite::CommandResult {
        Ok(Vec::new())
    }
}

struct V8Parser;
#[tuffite::utility(name = "v8_parser")]
impl tuffite::utility::V8 for V8Parser {
    fn new(_: tuffite::V8Context) -> ApplicationResult<Self> {
        Ok(Self)
    }

    async fn resolve(
        &self,
        request: tuffite::utility::v8::ModuleRequest,
    ) -> Result<Option<tuffite::utility::v8::ModuleSource>, tuffite::utility::v8::Error> {
        let url = request.url();
        Ok(Some(tuffite::utility::v8::ModuleSource::new(
            url.to_owned(),
            "export default {ping: () => 'pong'};",
        )))
    }
}

struct AssetsParser;
#[tuffite::utility(name = "assets_parser")]
impl tuffite::utility::V8 for AssetsParser {
    fn new(_: tuffite::V8Context) -> ApplicationResult<Self> {
        Ok(Self)
    }
}

#[tuffite::main(browser = Backend, utilities(Indexer, Parser, V8Parser, AssetsParser))]
fn entry() -> tuffite::app::Builder<Backend> {
    tuffite::app::Builder::new()
}

#[test]
fn main_exports_only_explicit_named_utilities_and_a_stable_table() {
    let table = tuffite_get_application();
    assert!(!table.is_null());
    assert_eq!(table, tuffite_get_application());
    // SAFETY: The getter returns immutable static storage on success.
    let table = unsafe { &*table };
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
