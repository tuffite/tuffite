//! Typed application interfaces and startup configuration.
use crate::{
    Application, Browser, Command, CommandRegistry, CommandResult, Error, Result,
    UtilityServiceRegistration,
};
use std::collections::{BTreeMap, BTreeSet};
#[doc(hidden)]
pub trait InterfaceFor<T> {
    fn registry() -> CommandRegistry<T>;
}
pub struct InterfaceRegistry<T> {
    pub(crate) commands: Vec<CommandRegistry<T>>,
}
impl<T> Default for InterfaceRegistry<T> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
        }
    }
}
impl<T> InterfaceRegistry<T> {
    #[must_use]
    pub fn register<I: InterfaceFor<T>>(mut self) -> Self {
        self.commands.push(I::registry());
        self
    }
    #[must_use]
    pub fn commands(mut self, commands: CommandRegistry<T>) -> Self {
        self.commands.push(commands);
        self
    }
}
pub(crate) struct FrozenRegistry<A: Application, B: Browser> {
    pub(crate) application: Vec<CommandRegistry<A>>,
    pub(crate) browser: Vec<CommandRegistry<B>>,
    pub(crate) scopes: BTreeMap<u64, u32>,
}
impl<A: Application, B: Browser> FrozenRegistry<A, B> {
    pub(crate) fn new(
        application: InterfaceRegistry<A>,
        browser: InterfaceRegistry<B>,
        utilities: &[UtilityServiceRegistration],
    ) -> Result<Self> {
        let mut scopes = BTreeMap::new();
        let mut channels = BTreeSet::new();
        let mut interface_scopes = BTreeMap::new();
        let descriptors = application
            .commands
            .iter()
            .flat_map(CommandRegistry::descriptors)
            .map(|d| (1, d))
            .chain(
                browser
                    .commands
                    .iter()
                    .flat_map(CommandRegistry::descriptors)
                    .map(|d| (2, d)),
            );
        for (scope, entry) in descriptors {
            if entry.id() >= crate::url_handler::SHORTCUT_COMMAND {
                return Err(Error::new("command ID is reserved by Framework"));
            }
            if entry.channel().is_empty() || !entry.channel().contains('.') {
                return Err(Error::new("registered command channel is invalid"));
            }
            if scopes.insert(entry.id(), scope).is_some() || !channels.insert(entry.channel()) {
                return Err(Error::new(
                    "duplicate interface or command across backend scopes",
                ));
            }
            let (interface, _) = entry.channel().rsplit_once('.').expect("validated channel");
            if interface_scopes
                .insert(interface, scope)
                .is_some_and(|previous| previous != scope)
            {
                return Err(Error::new(
                    "interface registered in multiple backend scopes",
                ));
            }
            if utilities
                .iter()
                .any(|u| interface == u.name() || u.interfaces().contains(&interface))
            {
                return Err(Error::new(
                    "interface registered in both backend and utility",
                ));
            }
        }
        Ok(Self {
            application: application.commands,
            browser: browser.commands,
            scopes,
        })
    }
}
/// Process startup configuration. Framework freezes it before entering Chromium.
pub struct Builder<A: Application> {
    pub(crate) interfaces: InterfaceRegistry<A>,
    pub(crate) configuration: Option<crate::app::Configuration>,
    pub(crate) startup: crate::Startup,
    pub(crate) context_factory: crate::context_data::Factory<crate::ApplicationContext>,
}
impl<A: Application> Default for Builder<A> {
    fn default() -> Self {
        Self::new()
    }
}
impl<A: Application> Builder<A> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            interfaces: InterfaceRegistry::default(),
            configuration: None,
            startup: crate::Startup::default(),
            context_factory: crate::context_data::Factory::default(),
        }
    }
    /// Registers a business context factory, without executing it during setup.
    #[must_use]
    pub fn context<C: std::any::Any + Send + Sync>(
        mut self,
        factory: impl Fn(crate::ApplicationContext) -> Result<C> + Send + Sync + 'static,
    ) -> Self {
        self.context_factory = crate::context_data::Factory::new(factory);
        self
    }
    #[must_use]
    pub fn configuration(mut self, configuration: crate::app::Configuration) -> Self {
        self.configuration = Some(configuration);
        self
    }
    #[must_use]
    pub fn interfaces(
        mut self,
        register: impl FnOnce(InterfaceRegistry<A>) -> InterfaceRegistry<A>,
    ) -> Self {
        self.interfaces
            .commands
            .extend(register(InterfaceRegistry::default()).commands);
        self
    }
    #[must_use]
    pub fn startup(&self) -> &crate::Startup {
        &self.startup
    }
    pub fn argv_mut(&mut self) -> &mut Vec<String> {
        self.startup.argv_mut()
    }
}
pub(crate) fn dispatch<T>(
    target: &mut T,
    registries: &[CommandRegistry<T>],
    command: Command<'_>,
) -> CommandResult {
    for registry in registries {
        if registry.contains(command.id()) {
            return registry.dispatch_to(target, command);
        }
    }
    Err(crate::CommandError::UnknownCommand)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ApplicationContext, BrowserContext, CommandDescriptor};

    struct RejectedApplication;
    impl Application for RejectedApplication {
        fn setup(_: Builder<Self>) -> Result<Builder<Self>> {
            Err(Error::new("setup rejected"))
        }
        fn new(_: ApplicationContext) -> Result<Self> {
            panic!("setup must precede backend construction")
        }
    }
    #[test]
    fn application_setup_errors_stop_before_registry_and_backend_creation() {
        assert!(
            RejectedApplication::setup(Builder::new())
                .err()
                .unwrap()
                .to_string()
                .contains("setup rejected")
        );
    }

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
        fn dispatch(&mut self, _: Command<'_>) -> CommandResult {
            panic!("unregistered fallback")
        }
    }
    struct First;
    struct Second;
    impl<T> InterfaceFor<T> for First {
        fn registry() -> CommandRegistry<T> {
            const COMMANDS: &[CommandDescriptor] =
                &[CommandDescriptor::new("test.First.Call", 1, 0)];
            CommandRegistry::new(|_, _| Ok(vec![1]), COMMANDS)
        }
    }
    impl<T> InterfaceFor<T> for Second {
        fn registry() -> CommandRegistry<T> {
            const COMMANDS: &[CommandDescriptor] =
                &[CommandDescriptor::new("test.Second.Call", 2, 0)];
            CommandRegistry::new(|_, _| Ok(vec![2]), COMMANDS)
        }
    }
    #[test]
    #[allow(clippy::redundant_closure_for_method_calls)] // Exercise the grouped registration DSL.
    fn groups_append_and_frozen_scopes_share_dispatch_metadata() {
        let builder = Builder::<App>::new()
            .interfaces(|r| r.register::<First>())
            .interfaces(|r| r)
            .interfaces(|_| InterfaceRegistry::default());
        let browser = InterfaceRegistry::default().register::<Second>();
        let frozen = FrozenRegistry::<App, Profile>::new(builder.interfaces, browser, &[]).unwrap();
        assert_eq!(frozen.scopes.get(&1), Some(&1));
        assert_eq!(frozen.scopes.get(&2), Some(&2));
        assert_eq!(frozen.scopes.get(&3), None);
        assert_eq!(
            dispatch(&mut App, &frozen.application, Command::new(1, 0, &[])).unwrap(),
            [1]
        );
        assert_eq!(
            dispatch(&mut Profile, &frozen.browser, Command::new(2, 0, &[])).unwrap(),
            [2]
        );
        assert!(matches!(
            dispatch(&mut Profile, &frozen.browser, Command::new(1, 0, &[])),
            Err(crate::CommandError::UnknownCommand)
        ));
    }
    #[test]
    fn duplicate_interfaces_and_cross_scope_commands_are_rejected() {
        assert!(
            FrozenRegistry::<App, Profile>::new(
                InterfaceRegistry::default()
                    .register::<First>()
                    .register::<First>(),
                InterfaceRegistry::default(),
                &[]
            )
            .is_err()
        );
        assert!(
            FrozenRegistry::<App, Profile>::new(
                InterfaceRegistry::default().register::<First>(),
                InterfaceRegistry::default().register::<First>(),
                &[]
            )
            .is_err()
        );
    }

    #[test]
    fn explicit_handlers_cannot_split_one_interface_between_scopes() {
        const FIRST: &[CommandDescriptor] = &[CommandDescriptor::new("test.Shared.First", 1, 0)];
        const SECOND: &[CommandDescriptor] = &[CommandDescriptor::new("test.Shared.Second", 2, 0)];
        let app = InterfaceRegistry::default()
            .commands(CommandRegistry::new(|_, _| Ok(Vec::new()), FIRST));
        let browser = InterfaceRegistry::default()
            .commands(CommandRegistry::new(|_, _| Ok(Vec::new()), SECOND));
        assert!(FrozenRegistry::<App, Profile>::new(app, browser, &[]).is_err());
    }
}
