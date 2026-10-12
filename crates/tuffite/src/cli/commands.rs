//! Composition root: each subcommand has its own Options context and implementation.

use super::{
    Command, build, check, context::Invocation, dev, doctor, framework, init, package,
    processes::Processes,
};
use crate::{
    build::framework::Distribution,
    ioc::{FactoryContext, Service, container::Container, factory},
};
use anyhow::Context as _;

#[derive(Clone, Copy)]
pub(super) enum Dependencies {
    None,
    Processes,
    Native,
}

pub(super) fn service<
    P: Send + Sync + 'static,
    C: Send + Sync + 'static,
    T: Send + Sync + 'static,
    F: Fn(&mut FactoryContext<'_, C>) -> Result<std::sync::Arc<T>, String> + Send + Sync + 'static,
>(
    parent: &mut Container<P>,
    options: C,
    construct: F,
    dependencies: Dependencies,
) -> anyhow::Result<std::sync::Arc<T>> {
    let mut registry = parent.scope(options);
    if matches!(dependencies, Dependencies::Processes | Dependencies::Native) {
        registry
            .import::<Service<dyn Processes>>()
            .map_err(anyhow::Error::msg)?;
    }
    if matches!(dependencies, Dependencies::Native) {
        registry
            .import::<Service<Distribution>>()
            .map_err(anyhow::Error::msg)?;
    }
    registry
        .register(factory::<Service<T>, C, F>(construct))
        .map_err(anyhow::Error::msg)?;
    let mut scope = registry.freeze();
    scope.get::<Service<T>>().map_err(anyhow::Error::msg)
}

pub(super) fn run(command: Command, parent: &mut Container<Invocation>) -> anyhow::Result<()> {
    match command {
        Command::Init(options) => {
            service(parent, options, init::InitService::new, Dependencies::None)?.run()
        }
        Command::Check(options) => service(
            parent,
            options,
            check::CheckService::new,
            Dependencies::None,
        )?
        .run(),
        Command::Build(options) => {
            options.validate().map_err(anyhow::Error::msg)?;
            service(
                parent,
                options,
                build::BuildService::new,
                Dependencies::Native,
            )?
            .run()
        }
        Command::Doctor(options) => {
            options.validate().map_err(anyhow::Error::msg)?;
            service(
                parent,
                options,
                doctor::DoctorService::new,
                Dependencies::Native,
            )?
            .run()
        }
        Command::Dev(options) => {
            let dependencies = if options.requires_framework().map_err(anyhow::Error::msg)? {
                Dependencies::Native
            } else {
                Dependencies::Processes
            };
            service(parent, options, dev::DevService::new, dependencies)?.run()
        }
        Command::Package(options) => {
            options.validate().map_err(anyhow::Error::msg)?;
            service(
                parent,
                options,
                package::PackageService::new,
                Dependencies::Native,
            )?
            .run()
        }
        Command::Framework(options) => {
            framework::run(options, parent).context("run framework command")
        }
    }
}
