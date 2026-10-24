//! Framework maintenance commands use independent typed child scopes.
use super::service;
use crate::{
    build::shell_api::{Generator, OutputMode},
    ioc::{FactoryContext, Registry, Service},
};
use clap::{Args, Subcommand};
use std::{path::PathBuf, sync::Arc};

#[derive(Args, Debug)]
pub struct Options {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Generate or verify SDK JavaScript bindings at an explicit destination.
    GenerateJs(GenerateOptions),
    /// Generate checked-in Rust bindings for the SDK native Shell API client.
    GenerateRust(GenerateOptions),
    /// Verify a local Distribution and its runtime hashes.
    Verify(VerifyOptions),
}
#[derive(Args, Debug, Clone)]
struct GenerateOptions {
    manifest: PathBuf,
    #[arg(long)]
    output: PathBuf,
    #[arg(long)]
    check: bool,
}
impl GenerateOptions {
    fn mode(&self) -> OutputMode {
        if self.check {
            OutputMode::Verify
        } else {
            OutputMode::Write
        }
    }
}
#[derive(Args, Debug, Clone)]
struct VerifyOptions {
    distribution: PathBuf,
}

pub(super) fn run<P: Send + Sync + 'static>(
    options: Options,
    parent: &Registry<P>,
) -> anyhow::Result<()> {
    let mut scope = parent.scope(options);
    scope.freeze();
    match scope.context().command.clone() {
        Command::GenerateJs(options) => service(&scope, options, GenerateJsService::new)?.run(),
        Command::GenerateRust(options) => service(&scope, options, GenerateRustService::new)?.run(),
        Command::Verify(options) => service(&scope, options, VerifyService::new)?.run(),
    }
}

struct GenerateJsService(Arc<GenerateOptions>);
impl GenerateJsService {
    fn new(build: &mut FactoryContext<'_, GenerateOptions>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self(
            build.depends_on::<Service<GenerateOptions>>()?,
        )))
    }

    fn run(&self) -> anyhow::Result<()> {
        Generator::new()
            .framework()
            .manifest(&self.0.manifest)
            .javascript(&self.0.output, self.0.mode())
            .generate()
            .map(|_| ())
            .map_err(anyhow::Error::msg)
    }
}
struct GenerateRustService(Arc<GenerateOptions>);
impl GenerateRustService {
    fn new(build: &mut FactoryContext<'_, GenerateOptions>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self(
            build.depends_on::<Service<GenerateOptions>>()?,
        )))
    }

    fn run(&self) -> anyhow::Result<()> {
        Generator::new()
            .framework()
            .manifest(&self.0.manifest)
            .rust_file(&self.0.output, self.0.mode())
            .generate()
            .map(|_| ())
            .map_err(anyhow::Error::msg)
    }
}
struct VerifyService(Arc<VerifyOptions>);
impl VerifyService {
    fn new(build: &mut FactoryContext<'_, VerifyOptions>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self(
            build.depends_on::<Service<VerifyOptions>>()?,
        )))
    }

    fn run(&self) -> anyhow::Result<()> {
        let value = crate::build::framework::Distribution::open(&self.0.distribution)
            .map_err(anyhow::Error::msg)?;
        value.runtime_files(true).map_err(anyhow::Error::msg)?;
        value.bootstrap(true).map_err(anyhow::Error::msg)?;
        Ok(())
    }
}
