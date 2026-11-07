//! Generated service traits execute in supervised Rust Utility processes.
use crate::shell_api::{self, MojomSample};
use std::sync::atomic::{AtomicU64, Ordering};
use tuffite::CommandError;

struct WorkerContext {
    base: tuffite::UtilityContext,
    initial_value: u64,
}

#[derive(Default)]
pub struct Worker {
    value: AtomicU64,
}

impl tuffite::Utility for Worker {
    fn setup(
        builder: tuffite::utility::Builder<Self>,
    ) -> tuffite::Result<tuffite::utility::Builder<Self>> {
        Ok(builder
            .context(|base| {
                Ok(WorkerContext {
                    base,
                    initial_value: 0,
                })
            })
            .interfaces(|registry| {
                registry
                    .register::<shell_api::interfaces::Echo>()
                    .register::<shell_api::interfaces::Aux>()
            }))
    }
    async fn new(base: tuffite::UtilityContext) -> tuffite::Result<Self> {
        let context = base.downcast::<WorkerContext>()?;
        assert_eq!(context.base.name(), base.name());
        Ok(Self {
            value: AtomicU64::new(context.initial_value),
        })
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
