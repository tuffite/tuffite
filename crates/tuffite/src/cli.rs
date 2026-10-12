mod build;
mod check;
mod config;
mod context;
mod dev;
mod doctor;
mod filesystem;
mod framework;
mod hooks;
mod init;
mod package;
mod processes;
mod workflow;

use crate::ioc::{FactoryContext, Registry, Service, factory};
use anyhow::Context as _;
use context::Invocation;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tuff",
    version = env!("TUFFITE_CLI_VERSION"),
    about = "Build and package Tuffite desktop applications"
)]
pub struct Cli {
    /// Increase logging detail (-v debug, -vv trace, -vvv trace with source locations).
    #[arg(short = 'v', action = clap::ArgAction::Count, global = true)]
    verbose: u8,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a minimal typed Tuffite application.
    Init(init::Options),
    /// Validate the complete application configuration without building it.
    Check(check::Options),
    /// Generate bindings and compile the application library.
    Build(build::Options),
    /// Report configuration, toolchain, target, and Distribution identity.
    Doctor(doctor::Options),
    /// Refresh and launch a native package against a loopback Web development server.
    Dev(dev::Options),
    /// Package an application as a directory, ZIP, macOS bundle/DMG or Windows installer.
    Package(package::Options),
    /// Maintain the Tuffite framework and prebuilt Framework Distributions.
    Framework(framework::Options),
}

impl Command {
    fn run(self, parent: &Registry<Invocation>) -> anyhow::Result<()> {
        match self {
            Command::Init(options) => service(parent, options, init::InitService::new)?.run(),
            Command::Check(options) => service(parent, options, check::CheckService::new)?.run(),
            Command::Build(options) => {
                options.validate().map_err(anyhow::Error::msg)?;
                service(parent, options, build::BuildService::new)?.run()
            }
            Command::Doctor(options) => {
                options.validate().map_err(anyhow::Error::msg)?;
                service(parent, options, doctor::DoctorService::new)?.run()
            }
            Command::Dev(options) => service(parent, options, dev::DevService::new)?.run(),
            Command::Package(options) => {
                options.validate().map_err(anyhow::Error::msg)?;
                service(parent, options, package::PackageService::new)?.run()
            }
            Command::Framework(options) => {
                framework::run(options, parent).context("run framework command")
            }
        }
    }
}

fn service<
    P: Send + Sync + 'static,
    C: Send + Sync + 'static,
    T: Send + Sync + 'static,
    F: Fn(&mut FactoryContext<'_, C>) -> Result<std::sync::Arc<T>, String> + Send + Sync + 'static,
>(
    parent: &Registry<P>,
    options: C,
    construct: F,
) -> anyhow::Result<std::sync::Arc<T>> {
    let mut scope = parent.scope(options);
    scope
        .register(factory::<Service<T>, C, F>(construct))
        .map_err(anyhow::Error::msg)?;
    scope.freeze();
    scope.get::<Service<T>>().map_err(anyhow::Error::msg)
}

pub fn main() {
    if let Err(error) = Cli::parse().run() {
        tracing::error!(error = %format_args!("{error:#}"), "command failed");
        std::process::exit(2);
    }
}

impl Cli {
    pub fn run(self) -> anyhow::Result<()> {
        Self::init_tracing(self.verbose);
        tracing::debug!(command = ?self.command, "running Tuffite CLI command");
        let mut context = context::registry(matches!(
            &self.command,
            Command::Dev(options) if options.force_download_framework
        ))
        .map_err(anyhow::Error::msg)?;
        context.freeze();
        self.command.run(&context)
    }

    fn init_tracing(verbosity: u8) {
        let filter = if verbosity == 0 {
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        } else {
            tracing_subscriber::EnvFilter::new(if verbosity == 1 { "debug" } else { "trace" })
        };
        let builder = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_ansi(false)
            .with_writer(std::io::stderr);
        let _ = if verbosity >= 3 {
            builder
                .with_target(true)
                .with_file(true)
                .with_line_number(true)
                .with_thread_ids(true)
                .try_init()
        } else {
            builder.with_target(false).try_init()
        };
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser as _;

    use super::{Cli, Command};

    #[test]
    fn version_reports_the_build_commit() {
        let error = Cli::try_parse_from(["tuff", "--version"]).err().unwrap();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
        assert_eq!(
            error.to_string(),
            format!("tuff {}\n", env!("TUFFITE_CLI_VERSION"))
        );
        assert!(env!("TUFFITE_CLI_VERSION").starts_with(concat!(env!("CARGO_PKG_VERSION"), " (")));
    }

    #[test]
    fn verbosity_is_global_and_counted() {
        assert_eq!(
            Cli::try_parse_from(["tuff", "-v", "check"])
                .unwrap()
                .verbose,
            1
        );
        assert_eq!(
            Cli::try_parse_from(["tuff", "check", "-vv"])
                .unwrap()
                .verbose,
            2
        );
        assert_eq!(
            Cli::try_parse_from(["tuff", "-vvv", "doctor"])
                .unwrap()
                .verbose,
            3
        );
    }

    #[test]
    fn bindings_and_downloads_have_no_standalone_commands() {
        for arguments in [
            vec!["tuff", "generate"],
            vec!["tuff", "download"],
            vec!["tuff", "framework", "download"],
        ] {
            assert!(Cli::try_parse_from(arguments).is_err());
        }
        assert!(Cli::try_parse_from(["tuff", "dev", "--prepare-only"]).is_ok());
    }

    #[test]
    fn dev_accepts_forced_framework_download() {
        let cli = Cli::try_parse_from(["tuff", "dev", "--force-download-framework"]).unwrap();
        let Command::Dev(options) = cli.command else {
            panic!("expected dev command");
        };
        assert!(options.force_download_framework);
    }

    #[test]
    fn application_commands_accept_discovery_and_manifest_overrides() {
        for command in ["check", "build", "doctor", "dev", "package"] {
            assert!(Cli::try_parse_from(["tuff", command]).is_ok());
            assert!(
                Cli::try_parse_from(["tuff", command, "--manifest", "project/tuffite.json"])
                    .is_ok()
            );
        }
        assert!(Cli::try_parse_from(["tuff", "package", "--format", "app"]).is_ok());
        for command in ["check", "build", "doctor"] {
            assert!(Cli::try_parse_from(["tuff", command, "project/tuffite.json"]).is_ok());
            assert!(
                Cli::try_parse_from([
                    "tuff",
                    command,
                    "project/tuffite.json",
                    "--manifest",
                    "other/tuffite.json"
                ])
                .is_err()
            );
        }
    }
}
