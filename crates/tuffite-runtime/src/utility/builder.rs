use crate::{Command, CommandDescriptor, CommandResult};
/// Generated typed interface binding for a Rust Utility.
#[doc(hidden)]
pub trait InterfaceFor<S> {
    fn interface() -> Interface<S>;
}
#[doc(hidden)]
pub struct Interface<S> {
    pub(super) name: &'static str,
    pub(super) dispatch: for<'a> fn(&S, Command<'a>) -> CommandResult,
    pub(super) descriptors: &'static [CommandDescriptor],
}
impl<S> Interface<S> {
    #[doc(hidden)]
    pub const fn new(
        name: &'static str,
        dispatch: for<'a> fn(&S, Command<'a>) -> CommandResult,
        descriptors: &'static [CommandDescriptor],
    ) -> Self {
        Self {
            name,
            dispatch,
            descriptors,
        }
    }
}
pub struct InterfaceRegistry<S> {
    pub(super) interfaces: Vec<Interface<S>>,
}
impl<S> Default for InterfaceRegistry<S> {
    fn default() -> Self {
        Self {
            interfaces: Vec::new(),
        }
    }
}
impl<S> InterfaceRegistry<S> {
    #[must_use]
    pub fn register<I: InterfaceFor<S>>(mut self) -> Self {
        self.interfaces.push(I::interface());
        self
    }
}
use crate::{Error, Result};

/// Owned startup configuration for one Rust Utility.
///
/// Framework supplies empty startup data before `setup`.
/// The returned builder is validated and frozen before constructing service state.
pub struct Builder<S = ()> {
    pub(crate) context_factory: crate::context_data::Factory<crate::UtilityContext>,
    pub(super) configuration: Vec<u8>,
    pub(super) interfaces: InterfaceRegistry<S>,
    pub(super) name: Option<String>,
    pub(super) startup: crate::Startup,
}

impl<S> Default for Builder<S> {
    fn default() -> Self {
        Self {
            context_factory: crate::context_data::Factory::default(),
            configuration: Vec::new(),
            interfaces: InterfaceRegistry::default(),
            name: None,
            startup: crate::Startup::default(),
        }
    }
}
impl<S> Builder<S> {
    #[must_use]
    pub fn context<C: std::any::Any + Send + Sync>(
        mut self,
        factory: impl Fn(crate::UtilityContext) -> crate::Result<C> + Send + Sync + 'static,
    ) -> Self {
        self.context_factory = crate::context_data::Factory::new(factory);
        self
    }

    #[must_use]
    pub fn startup(&self) -> &crate::Startup {
        &self.startup
    }
    #[must_use]
    pub fn service_name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub(super) fn named(mut self, name: &str) -> Self {
        self.name = Some(name.into());
        self
    }

    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    #[must_use]
    pub fn interfaces(
        mut self,
        register: impl FnOnce(InterfaceRegistry<S>) -> InterfaceRegistry<S>,
    ) -> Self {
        self.interfaces = register(self.interfaces);
        self
    }
    /// Replaces service-local startup data. This does not grant native capabilities.
    #[must_use]
    pub fn configuration(mut self, configuration: impl Into<Vec<u8>>) -> Self {
        self.configuration = configuration.into();
        self
    }

    /// Reads the current startup data, initially empty.
    #[must_use]
    pub fn configuration_bytes(&self) -> &[u8] {
        &self.configuration
    }

    pub(super) fn finish(self) -> Result<Vec<u8>> {
        if self.configuration.len() > 64 * 1024 {
            return Err(Error::new("Utility configuration exceeds 64 KiB"));
        }
        Ok(self.configuration)
    }
}

impl<S> std::fmt::Debug for Builder<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Builder")
            .field("name", &self.name)
            .field("configuration_bytes", &self.configuration.len())
            .field("interfaces", &self.interfaces.interfaces.len())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Utility, UtilityContext, UtilityRequest};

    struct ConfiguredUtility;
    impl Utility for ConfiguredUtility {
        fn setup(builder: Builder<Self>) -> Result<Builder<Self>> {
            assert_eq!(builder.configuration_bytes(), b"manifest");
            Ok(builder
                .configuration(b"effective".to_vec())
                .context(|base| {
                    assert_eq!(base.configuration(), b"effective");
                    Ok((base, String::from("injected")))
                }))
        }
        async fn new(context: UtilityContext) -> Result<Self> {
            assert_eq!(context.configuration(), b"effective");
            assert_eq!(
                context.downcast::<(UtilityContext, String)>()?.1,
                "injected"
            );
            Ok(Self)
        }
        async fn invoke(&self, _: UtilityRequest) -> crate::CommandResult {
            Ok(Vec::new())
        }
    }

    #[test]
    fn setup_receives_manifest_and_constructor_receives_frozen_result() {
        let context = UtilityContext::new_for_test("configured", b"manifest".to_vec());
        assert!(
            futures_executor::block_on(super::super::create_utility::<ConfiguredUtility>(context))
                .is_ok()
        );
    }

    struct FactoryRejected;
    impl Utility for FactoryRejected {
        fn setup(builder: Builder<Self>) -> Result<Builder<Self>> {
            Ok(builder.context(|_| Err::<(), _>(Error::new("factory rejected"))))
        }
        async fn new(_: UtilityContext) -> Result<Self> {
            panic!("factory failure must precede new")
        }
    }
    #[test]
    fn context_factory_failure_rejects_before_backend_construction() {
        let context = UtilityContext::new_for_test("rejected", Vec::new());
        let error =
            futures_executor::block_on(super::super::create_utility::<FactoryRejected>(context))
                .err()
                .unwrap();
        assert!(error.to_string().contains("factory rejected"));
    }

    struct InvalidUtility;
    impl Utility for InvalidUtility {
        fn setup(builder: Builder<Self>) -> Result<Builder<Self>> {
            Ok(builder.configuration(vec![0; 64 * 1024 + 1]))
        }
        async fn new(_: UtilityContext) -> Result<Self> {
            panic!("invalid setup must not construct service state")
        }
        async fn invoke(&self, _: UtilityRequest) -> crate::CommandResult {
            unreachable!()
        }
    }

    #[test]
    fn invalid_setup_is_rejected_before_constructor() {
        let context = UtilityContext::new_for_test("invalid", Vec::new());
        assert!(
            futures_executor::block_on(super::super::create_utility::<InvalidUtility>(context))
                .is_err()
        );
    }
}
