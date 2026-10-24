//! Development command boundary. Session and domain services live in separate modules.
mod application;
mod configuration;
mod server;
mod session;
mod watch;

use anyhow::Context as _;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::Arc;

use super::processes::Processes;
use crate::{
    build::framework::Distribution,
    ioc::{FactoryContext, Service},
};

#[derive(clap::Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Options {
    /// Application configuration; otherwise search the current directory and its parents.
    #[arg(long)]
    pub(super) manifest: Option<PathBuf>,
    /// Unpacked development package to create or refresh.
    #[arg(long)]
    package: Option<PathBuf>,
    /// Loopback HTTP Vite or other development-server entry URL.
    #[arg(long)]
    url: Option<String>,
    /// Web development server executable.
    #[arg(long)]
    web_command: Option<PathBuf>,
    /// Argument forwarded to the Web development server; repeat as needed.
    #[arg(long, allow_hyphen_values = true)]
    web_arg: Vec<OsString>,
    /// Working directory for the Web development server.
    #[arg(long)]
    web_cwd: Option<PathBuf>,
    /// Argument forwarded to the application executable; repeat as needed.
    #[arg(long, allow_hyphen_values = true)]
    app_arg: Vec<OsString>,
    /// Loopback Chrome `DevTools` Protocol port. Overrides `build.devtoolsPort`.
    #[arg(long)]
    devtools_port: Option<u16>,
    /// Refresh and validate the native development package without launching it.
    #[arg(long)]
    prepare_only: bool,
    /// Compile the application Rust library with Cargo's release profile.
    #[arg(long)]
    release: bool,
    /// Ignore local and cached Framework selections and download the Release artifact again.
    #[arg(long)]
    pub(super) force_download_framework: bool,
    /// Disable automatic Rust and Shell API rebuilds when project sources change.
    #[arg(long)]
    no_watch: bool,
}

impl Options {
    pub(super) fn requires_framework(&self) -> Result<bool, String> {
        let configuration = configuration::Configuration::load(self)?;
        if self.force_download_framework && configuration.manifest.is_none() {
            return Err("--force-download-framework requires an application manifest".to_owned());
        }
        Ok(configuration.manifest.is_some())
    }
}

pub(super) struct DevService {
    options: Arc<Options>,
    framework: Option<Arc<Distribution>>,
    processes: Arc<dyn Processes>,
}
impl DevService {
    pub(super) fn new(build: &mut FactoryContext<'_, Options>) -> Result<Arc<Self>, String> {
        Ok(Arc::new(Self {
            options: build.depends_on::<Service<Options>>()?,
            framework: if build.context().requires_framework()? {
                Some(build.depends_on::<Service<Distribution>>()?)
            } else {
                None
            },
            processes: build.depends_on::<Service<dyn Processes>>()?,
        }))
    }

    pub(super) fn run(&self) -> anyhow::Result<()> {
        session::DevelopmentSession::new(
            self.framework.clone(),
            self.processes.clone(),
            &self.options,
        )
        .and_then(session::DevelopmentSession::run)
        .map_err(anyhow::Error::msg)
        .context("launch development application")
    }
}
