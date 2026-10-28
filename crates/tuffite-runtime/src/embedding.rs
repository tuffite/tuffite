//! Native embedding definitions and deferred process preparation.
use crate::application::builder::{Builder, FrozenRegistry, dispatch};
use crate::{
    Application, ApplicationContext, Browser, BrowserContext, Command, CommandResult, Error,
    Result, ShutdownContext, UtilityServiceRegistration,
};
use std::{any::Any, marker::PhantomData, sync::OnceLock};
use tuffite_abi::{Application as AbiApplication, ByteSlice, Handle, Host, Status};

type ExportFactory = fn() -> AbiApplication;
/// Selects Rust implementations exposed to the native Framework. No setup or state is constructed here.
pub struct Embedding<A: Application> {
    browser: Option<ExportFactory>,
    utilities: Vec<UtilityRegistrationFactory>,
    duplicate_browser: bool,
    marker: PhantomData<fn() -> A>,
}
impl<A: Application> Default for Embedding<A> {
    fn default() -> Self {
        Self::new()
    }
}
impl<A: Application> Embedding<A> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            browser: None,
            utilities: Vec::new(),
            duplicate_browser: false,
            marker: PhantomData,
        }
    }
    #[must_use]
    pub fn browser<B: Browser>(mut self) -> Self {
        self.duplicate_browser |= self.browser.replace(export::<A, B>).is_some();
        self
    }
    #[must_use]
    pub fn utility<U: crate::Utility>(mut self) -> Self {
        self.utilities.push(crate::utility::register::<U>);
        self
    }
    #[must_use]
    pub fn v8_utility<U: crate::utility::V8>(mut self) -> Self {
        self.utilities.push(crate::utility::register_v8::<U>);
        self
    }
    /// Called only by the generated native export.
    #[doc(hidden)]
    pub fn into_application(self) -> Result<AbiApplication> {
        if self.duplicate_browser {
            return Err(Error::new("Browser implementation selected more than once"));
        }
        let export = self
            .browser
            .ok_or_else(|| Error::new("Embedding requires a Browser implementation"))?;
        UTILITY_FACTORIES
            .set(self.utilities)
            .map_err(|_| Error::new("embedding already installed"))?;
        Ok(export())
    }
}
type UtilityRegistrationFactory = fn(crate::Startup) -> Result<UtilityServiceRegistration>;
static UTILITY_FACTORIES: OnceLock<Vec<UtilityRegistrationFactory>> = OnceLock::new();
static UTILITIES: OnceLock<Box<[UtilityServiceRegistration]>> = OnceLock::new();
pub(crate) fn utility_services() -> &'static [UtilityServiceRegistration] {
    UTILITIES.get().map_or(&[], AsRef::as_ref)
}
struct Registrations;
impl crate::application::Registrations for Registrations {
    fn utility_services() -> &'static [UtilityServiceRegistration] {
        utility_services()
    }
}
struct Prepared<A: Application, B: Browser> {
    registry: FrozenRegistry<A, B>,
    configuration: Vec<u8>,
    denied_origins: Vec<String>,
}
static STARTUP: OnceLock<crate::Startup> = OnceLock::new();
pub(crate) fn startup() -> Option<crate::Startup> {
    STARTUP.get().cloned()
}
static PREPARED: OnceLock<Box<dyn Any + Send + Sync>> = OnceLock::new();
fn prepared<A: Application, B: Browser>() -> Result<&'static Prepared<A, B>> {
    PREPARED
        .get()
        .and_then(|p| p.downcast_ref())
        .ok_or_else(|| Error::new("application setup has not completed"))
}
fn export<A: Application, B: Browser>() -> AbiApplication {
    let mut table =
        crate::application::for_application::<ApplicationBackend<A, B>, Registrations>();
    table.prepare_process = Some(prepare_process::<A, B>);
    table.configuration = Some(configuration::<A, B>);
    table.start_profile = Some(start_profile::<A, B>);
    table.command_scope = Some(command_scope::<A, B>);
    table
}

fn collect<A: Application, B: Browser>(
    startup: crate::Startup,
) -> Result<(Prepared<A, B>, crate::Startup)> {
    let builder = A::setup(Builder {
        startup,
        ..Builder::default()
    })?;
    // Collect static metadata without opening a profile or constructing its backend.
    let browser = B::setup(crate::browser::Builder::default())?;
    let mut utilities = Vec::new();
    for factory in UTILITY_FACTORIES.get().into_iter().flatten() {
        let registration = factory(builder.startup.clone())?;
        crate::utility_entry::validate_registration(&utilities, registration)
            .map_err(Error::new)?;
        utilities.push(registration);
    }
    UTILITIES
        .set(utilities.into_boxed_slice())
        .map_err(|_| Error::new("utility registration already frozen"))?;
    let registry = FrozenRegistry::new(builder.interfaces, browser.interfaces, utility_services())?;
    let config = builder
        .configuration
        .ok_or_else(|| Error::new("Application::setup must supply runtime configuration"))?;
    let configuration = config.to_bytes()?;
    Ok((
        Prepared {
            registry,
            configuration,
            denied_origins: browser.devtools_denied_origins,
        },
        builder.startup,
    ))
}
unsafe extern "C" fn prepare_process<A: Application, B: Browser>(
    kind: u32,
    args: *const ByteSlice,
    count: usize,
    user: *mut std::ffi::c_void,
    emit: Option<tuffite_abi::LaunchArgumentFn>,
) -> Status {
    // SAFETY: Native inputs and copying sink remain alive during this call.
    unsafe {
        crate::builder::prepare_with(kind, args, count, user, emit, |startup| {
            let (prepared, edited) = collect::<A, B>(startup.clone())?;
            PREPARED
                .set(Box::new(prepared))
                .map_err(|_| Error::new("process setup called more than once"))?;
            STARTUP
                .set(edited.clone())
                .map_err(|_| Error::new("startup already published"))?;
            *startup = edited;
            Ok(())
        })
    }
}
unsafe extern "C" fn configuration<A: Application, B: Browser>() -> ByteSlice {
    prepared::<A, B>().map_or_else(
        |_| ByteSlice::from_slice(&[]),
        |p| ByteSlice::from_slice(&p.configuration),
    )
}
unsafe extern "C" fn command_scope<A: Application, B: Browser>(id: u64) -> u32 {
    prepared::<A, B>()
        .ok()
        .and_then(|p| p.registry.scopes.get(&id))
        .copied()
        .unwrap_or(0)
}
pub(crate) struct ApplicationBackend<A, B>(A, PhantomData<fn() -> B>);
impl<A: Application, B: Browser> Browser for ApplicationBackend<A, B> {
    fn setup(mut builder: crate::browser::Builder<Self>) -> Result<crate::browser::Builder<Self>> {
        builder
            .devtools_denied_origins
            .clone_from(&prepared::<A, B>()?.denied_origins);
        Ok(builder)
    }
    fn new(context: BrowserContext) -> Result<Self> {
        if !context.context_client().is_available() {
            return Err(Error::new("Application requires multi-profile support"));
        }
        A::new(ApplicationContext::new(context)).map(|a| Self(a, PhantomData))
    }
    fn dispatch(&mut self, command: Command<'_>) -> CommandResult {
        dispatch(
            &mut self.0,
            &prepared::<A, B>()
                .map_err(|e| crate::CommandError::Failed(e.to_string()))?
                .registry
                .application,
            command,
        )
    }
    async fn initialize(&mut self) -> Result<()> {
        self.0.initialize().await
    }
    async fn shutdown(&mut self, context: ShutdownContext) -> Result<()> {
        self.0.shutdown(context).await
    }
}
struct ProfileBackend<A, B>(B, PhantomData<fn() -> A>);
impl<A: Application, B: Browser> Browser for ProfileBackend<A, B> {
    fn new(context: BrowserContext) -> Result<Self> {
        let setup = B::setup(crate::browser::Builder {
            profile_id: context.profile_id().cloned(),
            ..crate::browser::Builder::default()
        })?;
        let frozen = prepared::<A, B>()?;
        let actual: Vec<_> = setup
            .interfaces
            .commands
            .iter()
            .flat_map(crate::CommandRegistry::descriptors)
            .collect();
        let expected: Vec<_> = frozen
            .registry
            .browser
            .iter()
            .flat_map(crate::CommandRegistry::descriptors)
            .collect();
        if actual != expected || setup.devtools_denied_origins != frozen.denied_origins {
            return Err(Error::new(
                "profile setup cannot change frozen interfaces or native policy",
            ));
        }
        B::new(context).map(|b| Self(b, PhantomData))
    }
    fn dispatch(&mut self, command: Command<'_>) -> CommandResult {
        dispatch(
            &mut self.0,
            &prepared::<A, B>()
                .map_err(|e| crate::CommandError::Failed(e.to_string()))?
                .registry
                .browser,
            command,
        )
    }
    async fn initialize(&mut self) -> Result<()> {
        self.0.initialize().await
    }
    async fn shutdown(&mut self, context: ShutdownContext) -> Result<()> {
        self.0.shutdown(context).await
    }
}
pub(crate) unsafe extern "C" fn start_profile<A: Application, B: Browser>(
    parent: Handle,
    host: *const Host,
    configuration: ByteSlice,
    profile_id: ByteSlice,
    out: *mut Handle,
) -> Status {
    std::panic::catch_unwind(|| {
        if out.is_null() {
            return Status::InvalidArgument;
        }
        // SAFETY: Native supplies a writable output pointer.
        unsafe {
            out.write(0);
        }
        if profile_id.data.is_null() || profile_id.len > 128 {
            return Status::InvalidArgument;
        }
        // SAFETY: Native borrows readable profile bytes for this call.
        let bytes = unsafe { std::slice::from_raw_parts(profile_id.data, profile_id.len) };
        let Ok(id) = std::str::from_utf8(bytes) else {
            return Status::InvalidArgument;
        };
        let Ok(id) = crate::ProfileId::new(id) else {
            return Status::InvalidArgument;
        };
        let Ok(parent) = crate::abi::context_for_runtime(parent) else {
            return Status::InvalidHandle;
        };
        if parent.profile_id().is_some() {
            return Status::InvalidArgument;
        }
        // SAFETY: Forward the native startup contract to the shared context adapter.
        unsafe {
            crate::abi::start_context::<ProfileBackend<A, B>>(
                host,
                configuration.data,
                configuration.len,
                out,
                Some((ApplicationContext::new(parent), id)),
            )
        }
    })
    .unwrap_or(Status::Internal)
}

#[cfg(test)]
pub(crate) fn prepare_for_test<A: Application, B: Browser>() {
    let (prepared, _) = collect::<A, B>(crate::Startup::default()).unwrap();
    assert!(PREPARED.set(Box::new(prepared)).is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;
    struct App;
    struct Profile;
    impl Application for App {
        fn new(_: ApplicationContext) -> Result<Self> {
            panic!("metadata only")
        }
    }
    impl Browser for Profile {
        fn new(_: BrowserContext) -> Result<Self> {
            panic!("metadata only")
        }
    }
    #[test]
    fn missing_or_duplicate_browser_rejects_before_publishing_any_embedding() {
        assert!(
            Embedding::<App>::new()
                .into_application()
                .err()
                .unwrap()
                .to_string()
                .contains("requires a Browser")
        );
        assert!(
            Embedding::<App>::new()
                .browser::<Profile>()
                .browser::<Profile>()
                .into_application()
                .err()
                .unwrap()
                .to_string()
                .contains("more than once")
        );
    }
}
