use super::processes::Processes;
use super::{config, workflow};
use crate::build::framework::Distribution;
use crate::ioc::{FactoryContext, Service};
use anyhow::Context as _;
use std::sync::Arc;

#[derive(clap::Args, Debug)]
pub struct Options {
    #[command(flatten)]
    pub manifest: config::ManifestOptions,
}
impl Options {
    pub(super) fn validate(&self) -> Result<(), String> {
        config::check(&self.manifest.resolve()?).map(|_| ())
    }
}

pub(super) struct DoctorService {
    options: Arc<Options>,
    framework: Arc<Distribution>,
    processes: Arc<dyn Processes>,
}
impl DoctorService {
    pub(super) fn new(build: &mut FactoryContext<'_, Options>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self {
            options: build.depends_on::<Service<Options>>()?,
            framework: build.depends_on::<Service<Distribution>>()?,
            processes: build.depends_on::<Service<dyn Processes>>()?,
        }))
    }

    pub(super) fn run(&self) -> anyhow::Result<()> {
        let manifest = self
            .options
            .manifest
            .resolve()
            .map_err(anyhow::Error::msg)?;
        workflow::doctor(&self.framework, self.processes.as_ref(), &manifest)
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("diagnose {}", manifest.display()))
    }
}
