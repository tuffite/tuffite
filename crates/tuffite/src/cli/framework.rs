//! Application-facing Distribution commands and the GN code-generation entry.
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Options {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate or verify SDK JavaScript bindings at an explicit destination.
    GenerateJs {
        manifest: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        check: bool,
    },
    /// Generate the C++ builtin protocol consumed by Chromium GN.
    GenerateCpp {
        manifest: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },
    /// Generate checked-in Rust bindings for the SDK native Shell API client.
    GenerateRust {
        manifest: PathBuf,
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        check: bool,
    },
    /// Verify a local Distribution and its runtime hashes.
    Verify { distribution: PathBuf },
}

pub fn run(options: &Options) -> Result<(), String> {
    match &options.command {
        Command::GenerateJs {
            manifest,
            output,
            check,
        } => crate::build::shell_api::Generator::new()
            .manifest(manifest)
            .javascript(
                output,
                if *check {
                    crate::build::shell_api::OutputMode::Verify
                } else {
                    crate::build::shell_api::OutputMode::Write
                },
            )
            .generate()
            .map(|_| ()),
        Command::GenerateCpp { manifest, output } => crate::build::shell_api::Generator::new()
            .manifest(manifest)
            .cpp(output, crate::build::shell_api::OutputMode::Write)
            .generate()
            .map(|_| ()),
        Command::GenerateRust {
            manifest,
            output,
            check,
        } => crate::build::shell_api::Generator::new()
            .manifest(manifest)
            .rust_file(
                output,
                if *check {
                    crate::build::shell_api::OutputMode::Verify
                } else {
                    crate::build::shell_api::OutputMode::Write
                },
            )
            .generate()
            .map(|_| ()),
        Command::Verify { distribution } => {
            let value = crate::build::framework::Distribution::open(distribution)?;
            value.runtime_files(true)?;
            value.bootstrap(true)?;
            Ok(())
        }
    }
}
