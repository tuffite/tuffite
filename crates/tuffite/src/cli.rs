mod config;
mod context;
mod dev;
mod filesystem;
mod framework;
mod package;
mod workflow;

use std::path::PathBuf;

use context::Context;

use anyhow::Context as _;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "tuff",
    version,
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
    Init {
        /// Empty directory to initialize.
        directory: PathBuf,
        /// Cargo package name; defaults to the directory name.
        #[arg(long)]
        name: Option<String>,
    },
    /// Validate the complete application configuration without building it.
    Check {
        #[command(flatten)]
        manifest: config::ManifestOptions,
    },
    /// Generate bindings and compile the application library.
    Build {
        #[command(flatten)]
        manifest: config::ManifestOptions,
        /// Use Cargo's release profile.
        #[arg(long)]
        release: bool,
    },
    /// Report configuration, toolchain, target, and Distribution identity.
    Doctor {
        #[command(flatten)]
        manifest: config::ManifestOptions,
    },
    /// Refresh and launch a native package against a loopback Web development server.
    Dev(dev::Options),
    /// Package an application as a directory, ZIP, macOS bundle/DMG or Windows installer.
    Package(package::Options),
    /// Maintain the Tuffite framework and prebuilt Framework Distributions.
    Framework(framework::Options),
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
        let mut context = Context::new(matches!(
            &self.command,
            Command::Dev(options) if options.force_download_framework
        ));
        match self.command {
            Command::Init { directory, name } => workflow::init(&directory, name.as_deref())
                .map_err(anyhow::Error::msg)
                .with_context(|| format!("initialize {}", directory.display())),
            Command::Check { manifest } => {
                let manifest = manifest.resolve().map_err(anyhow::Error::msg)?;
                config::check(&manifest)
                    .map_err(anyhow::Error::msg)
                    .with_context(|| format!("check {}", manifest.display()))?;
                tracing::info!(manifest = %manifest.display(), "configuration is valid");
                Ok(())
            }
            Command::Build { manifest, release } => {
                let manifest = manifest.resolve().map_err(anyhow::Error::msg)?;
                workflow::build(&mut context, &manifest, release)
                    .map_err(anyhow::Error::msg)
                    .with_context(|| format!("build {}", manifest.display()))
            }
            Command::Doctor { manifest } => {
                let manifest = manifest.resolve().map_err(anyhow::Error::msg)?;
                workflow::doctor(&mut context, &manifest)
                    .map_err(anyhow::Error::msg)
                    .with_context(|| format!("diagnose {}", manifest.display()))
            }
            Command::Dev(options) => options
                .run(&mut context)
                .map_err(anyhow::Error::msg)
                .context("launch development application"),
            Command::Package(options) => options
                .run(&mut context)
                .map_err(anyhow::Error::msg)
                .context("package application"),
            Command::Framework(options) => framework::run(&options)
                .map_err(anyhow::Error::msg)
                .context("run framework command"),
        }
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
