//! Synchronous external process boundary used by build and package services.
use std::io;
use std::process::{Child, Command, ExitStatus, Output};

pub(super) trait Processes: Send + Sync {
    fn status(&self, command: &mut Command) -> io::Result<ExitStatus>;
    fn output(&self, command: &mut Command) -> io::Result<Output>;
    fn spawn(&self, command: &mut Command) -> io::Result<Child>;
}

pub(super) struct NativeProcesses;
impl Processes for NativeProcesses {
    fn status(&self, command: &mut Command) -> io::Result<ExitStatus> {
        command.status()
    }
    fn output(&self, command: &mut Command) -> io::Result<Output> {
        command.output()
    }
    fn spawn(&self, command: &mut Command) -> io::Result<Child> {
        command.spawn()
    }
}
