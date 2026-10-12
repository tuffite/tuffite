use super::workflow;
use crate::ioc::{FactoryContext, Service};
use anyhow::Context as _;
use std::{path::PathBuf, sync::Arc};

#[derive(clap::Args, Debug)]
pub struct Options {
    /// Empty directory to initialize.
    pub directory: PathBuf,
    /// Cargo package name; defaults to the directory name.
    #[arg(long)]
    pub name: Option<String>,
}

pub(super) struct InitService {
    options: Arc<Options>,
}
impl InitService {
    pub(super) fn new(build: &mut FactoryContext<'_, Options>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self {
            options: build.depends_on::<Service<Options>>()?,
        }))
    }

    pub(super) fn run(&self) -> anyhow::Result<()> {
        workflow::init(&self.options.directory, self.options.name.as_deref())
            .map_err(anyhow::Error::msg)
            .with_context(|| format!("initialize {}", self.options.directory.display()))
    }
}
