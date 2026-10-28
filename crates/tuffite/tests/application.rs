use tuffite::{Application, ApplicationContext, Browser, BrowserContext, Result};
#[allow(dead_code, clippy::all, clippy::pedantic)]
mod api {
    include!("fixtures/mojom_generated.rs");
}
struct App;
struct Profile;
static SETUPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
impl Application for App {
    fn setup(mut builder: tuffite::app::Builder<Self>) -> Result<tuffite::app::Builder<Self>> {
        SETUPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        builder.argv_mut().push("--prepared-by-setup".into());
        Ok(builder
            .configuration(tuffite::app::Configuration::new(["test"]))
            .interfaces(
                tuffite::app::InterfaceRegistry::register::<api::interfaces::BrowserDiagnostics>,
            ))
    }

    fn new(_: ApplicationContext) -> Result<Self> {
        panic!("registration must not construct Application")
    }
}
impl api::BrowserDiagnostics for App {
    fn identity(&self) -> Result<String, tuffite::CommandError> {
        Ok(String::new())
    }
    fn increment(&self) -> Result<u32, tuffite::CommandError> {
        Ok(0)
    }
}
impl Browser for Profile {
    fn setup(builder: tuffite::browser::Builder<Self>) -> Result<tuffite::browser::Builder<Self>> {
        Ok(builder
            .disable_devtools_for_origin("https://private.example")
            .interfaces(tuffite::app::InterfaceRegistry::register::<api::interfaces::Diagnostics>))
    }
    fn new(_: BrowserContext) -> Result<Self> {
        panic!("registration must not construct Browser")
    }
}
impl api::Diagnostics for Profile {
    fn process_id(&self) -> Result<u32, tuffite::CommandError> {
        Ok(0)
    }
}
#[tuffite::main]
#[allow(clippy::redundant_closure_for_method_calls)] // Show the grouped public API.
fn entry() -> tuffite::Embedding<App> {
    tuffite::Embedding::new().browser::<Profile>()
}

unsafe extern "C" fn collect(
    data: *mut std::ffi::c_void,
    bytes: tuffite_abi::ByteSlice,
) -> tuffite_abi::Status {
    // SAFETY: Setup borrows the test vector and readable bytes synchronously.
    let origins = unsafe { &mut *data.cast::<Vec<String>>() };
    let bytes = unsafe { std::slice::from_raw_parts(bytes.data, bytes.len) };
    origins.push(std::str::from_utf8(bytes).unwrap().to_owned());
    tuffite_abi::Status::Ok
}

#[test]
fn application_export_freezes_both_scopes_without_starting_them() {
    let first = tuffite_get_application();
    assert!(!first.is_null());
    assert_eq!(first, tuffite_get_application());
    // SAFETY: The generated exporter retains this immutable table for the process.
    let table = unsafe { &*first };
    assert!(table.start_profile.is_some());
    assert_eq!(SETUPS.load(std::sync::atomic::Ordering::SeqCst), 0);
    let input = [
        tuffite_abi::ByteSlice::from_slice(b"application"),
        tuffite_abi::ByteSlice::from_slice(b"--type=utility"),
    ];
    let mut arguments = Vec::<String>::new();
    // SAFETY: Input bytes and copying sink are live for this call.
    assert_eq!(
        unsafe {
            table.prepare_process.unwrap()(
                1,
                input.as_ptr(),
                input.len(),
                std::ptr::from_mut(&mut arguments).cast(),
                Some(collect),
            )
        },
        tuffite_abi::Status::Ok
    );
    assert_eq!(arguments.last().unwrap(), "--prepared-by-setup");
    assert_eq!(
        tuffite::Startup::default().process_kind(),
        tuffite::ProcessKind::Utility
    );
    assert_eq!(SETUPS.load(std::sync::atomic::Ordering::SeqCst), 1);
    // SAFETY: The callback returns frozen library-lifetime data.
    let configuration = unsafe { table.configuration.unwrap()() };
    assert!(configuration.len > 0);
    let mut origins = Vec::<String>::new();
    // SAFETY: The copying sink and vector remain live during static setup.
    assert_eq!(
        unsafe {
            table.setup_browser.unwrap()(std::ptr::from_mut(&mut origins).cast(), Some(collect))
        },
        tuffite_abi::Status::Ok
    );
    assert_eq!(origins, ["https://private.example"]);
    let scope = table.command_scope.unwrap();
    let application = <App as api::BrowserDiagnostics>::__TUFFITE_COMMANDS[0].id();
    let browser = <Profile as api::Diagnostics>::__TUFFITE_COMMANDS[0].id();
    // SAFETY: Static lookups have no context or borrowed memory inputs.
    assert_eq!(unsafe { scope(application) }, 1);
    assert_eq!(unsafe { scope(browser) }, 2);
    assert_eq!(unsafe { scope(0) }, 0);
    // Generated interface registries accept either backend scope.
    let _ = api::browserDiagnostics::registry::<App>();
}
