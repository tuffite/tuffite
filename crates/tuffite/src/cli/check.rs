use super::config;
use crate::ioc::{FactoryContext, Service};
use anyhow::Context as _;
use std::sync::Arc;

#[derive(clap::Args, Debug)]
pub struct Options {
    #[command(flatten)]
    pub manifest: config::ManifestOptions,
}

pub(super) struct CheckService {
    options: Arc<Options>,
}
impl CheckService {
    pub(super) fn new(build: &mut FactoryContext<'_, Options>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self {
            options: build.depends_on::<Service<Options>>()?,
        }))
    }

    pub(super) fn run(&self) -> anyhow::Result<()> {
        let manifest = self
            .options
            .manifest
            .resolve()
            .map_err(anyhow::Error::msg)?;
        config::check(&manifest)
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("check {}", manifest.display()))?;
        tracing::info!(manifest = %manifest.display(), "configuration is valid");
        Ok(())
    }
}
