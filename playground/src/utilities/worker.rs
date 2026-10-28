//! Generated service traits execute in supervised Rust Utility processes.
use crate::shell_api::{self, MojomSample};
use std::sync::atomic::{AtomicU64, Ordering};
use tuffite::CommandError;

#[derive(Default)]
pub struct Worker {
    value: AtomicU64,
}

impl tuffite::Utility for Worker {
    fn setup(
        builder: tuffite::utility::Builder<Self>,
    ) -> tuffite::Result<tuffite::utility::Builder<Self>> {
        Ok(builder.interfaces(|registry| {
            registry
                .register::<shell_api::interfaces::Echo>()
                .register::<shell_api::interfaces::Aux>()
        }))
    }
    async fn new(_: tuffite::UtilityContext) -> tuffite::Result<Self> {
        Ok(Self::default())
    }
}

impl shell_api::Echo for Worker {
    fn round_trip(&self, value: MojomSample) -> Result<MojomSample, CommandError> {
        self.value.store(value.large, Ordering::SeqCst);
        Ok(value)
    }
    fn restricted(&self, value: MojomSample) -> Result<MojomSample, CommandError> {
        Ok(value)
    }
}

impl shell_api::Aux for Worker {
    fn round_trip(&self, value: MojomSample) -> Result<MojomSample, CommandError> {
        Ok(value)
    }
    fn read_state(&self) -> Result<u64, CommandError> {
        Ok(self.value.load(Ordering::SeqCst))
    }
    fn process_id(&self) -> Result<u32, CommandError> {
        Ok(std::process::id())
    }
}
