//! Development command boundary. Session and domain services live in separate modules.
mod application;
mod configuration;
mod server;
mod session;
mod watch;

use super::Context;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(clap::Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct Options {
    /// Application configuration; otherwise search the current directory and its parents.
    #[arg(long)]
    manifest: Option<PathBuf>,
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
    pub(super) fn run(&self, context: &mut Context) -> Result<(), String> {
        session::DevelopmentSession::new(context, self)?.run()
    }
}
