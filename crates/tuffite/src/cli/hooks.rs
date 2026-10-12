//! Finite commands at build and package workflow boundaries.
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum BuildCommand {
    Script(String),
    Options {
        script: String,
        cwd: Option<PathBuf>,
    },
}

impl BuildCommand {
    pub(super) fn validate(&self, hook: &str) -> Result<(), String> {
        let script = match self {
            Self::Script(script) | Self::Options { script, .. } => script,
        };
        if script.trim().is_empty() {
            Err(format!("build.{hook}: script must not be empty"))
        } else {
            Ok(())
        }
    }

    pub(super) fn run(
        &self,
        root: &Path,
        hook: &str,
        processes: &dyn super::processes::Processes,
    ) -> Result<(), String> {
        self.validate(hook)?;
        let (script, cwd) = match self {
            Self::Script(script) => (script, root.to_path_buf()),
            Self::Options { script, cwd } => (
                script,
                cwd.as_ref()
                    .map_or_else(|| root.to_path_buf(), |cwd| root.join(cwd)),
            ),
        };
        tracing::info!(hook, cwd = %cwd.display(), "running application hook");
        let mut command = if cfg!(windows) {
            let mut command = Command::new("cmd");
            command.args(["/S", "/C", script]);
            command
        } else {
            let mut command = Command::new("/bin/sh");
            command.args(["-c", script]);
            command
        };
        let status = processes
            .status(command.current_dir(&cwd))
            .map_err(|error| format!("start build.{hook} in {}: {error}", cwd.display()))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("build.{hook} exited with {status}"))
        }
    }
}

pub(super) fn run(
    manifest: &Path,
    hook: &str,
    processes: &dyn super::processes::Processes,
) -> Result<(), String> {
    let value: serde_json::Value = serde_json::from_slice(
        &std::fs::read(manifest)
            .map_err(|error| format!("read {}: {error}", manifest.display()))?,
    )
    .map_err(|error| error.to_string())?;
    if let Some(command) = value["build"].get(hook) {
        let command: BuildCommand = serde_json::from_value(command.clone())
            .map_err(|error| format!("build.{hook}: {error}"))?;
        command.run(
            manifest
                .parent()
                .filter(|path| !path.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new(".")),
            hook,
            processes,
        )?;
    }
    Ok(())
}
