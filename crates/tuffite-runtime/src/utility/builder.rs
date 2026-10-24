use crate::{ApplicationError, ApplicationResult};

/// Owned startup configuration for one Rust Utility.
///
/// Framework seeds this with the service's manifest configuration before `setup`.
/// The returned builder is validated and frozen before constructing service state.
#[derive(Debug, Default)]
pub struct Builder {
    configuration: Vec<u8>,
}

impl Builder {
    /// Replaces service-local startup data. This does not grant native capabilities.
    #[must_use]
    pub fn configuration(mut self, configuration: impl Into<Vec<u8>>) -> Self {
        self.configuration = configuration.into();
        self
    }

    /// Reads the current startup data, initially from `app.utilities.<name>.configuration`.
    #[must_use]
    pub fn configuration_bytes(&self) -> &[u8] {
        &self.configuration
    }

    pub(super) fn finish(self) -> ApplicationResult<Vec<u8>> {
        if self.configuration.len() > 64 * 1024 {
            return Err(ApplicationError::new(
                "Utility configuration exceeds 64 KiB",
            ));
        }
        Ok(self.configuration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Utility, UtilityContext, UtilityRequest};

    struct ConfiguredUtility;
    impl Utility for ConfiguredUtility {
        fn setup(builder: Builder) -> ApplicationResult<Builder> {
            assert_eq!(builder.configuration_bytes(), b"manifest");
            Ok(builder.configuration(b"effective".to_vec()))
        }
        async fn new(context: UtilityContext) -> ApplicationResult<Self> {
            assert_eq!(context.configuration(), b"effective");
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

    struct InvalidUtility;
    impl Utility for InvalidUtility {
        fn setup(builder: Builder) -> ApplicationResult<Builder> {
            Ok(builder.configuration(vec![0; 64 * 1024 + 1]))
        }
        async fn new(_: UtilityContext) -> ApplicationResult<Self> {
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
