//! Owns the dev invocation, process runtime, and restart/stop decisions.
use super::Options;
use super::application::{DevelopmentApplication, Launch};
use super::configuration::Configuration;
use super::server::{DevelopmentToken, WebServer};
use super::watch::SourceWatcher;
use processkit::RunningProcess;
use std::net::TcpListener;
use std::path::Path;
use std::time::Duration;
use tokio::runtime::{Builder, Runtime};

pub(super) struct DevelopmentSession<'a> {
    framework: Option<std::sync::Arc<crate::build::framework::Distribution>>,
    processes: std::sync::Arc<dyn super::super::processes::Processes>,
    options: &'a Options,
    configuration: Configuration,
    runtime: Runtime,
}

enum DevExit {
    Application(processkit::Result<processkit::Outcome>),
    Web(processkit::Result<processkit::Outcome>),
    Interrupted(std::io::Result<()>),
    SourcesChanged,
    WaitFailed(processkit::Error),
}

enum SessionExit {
    Finished,
    SourcesChanged,
}

impl<'a> DevelopmentSession<'a> {
    pub(super) fn new(
        framework: Option<std::sync::Arc<crate::build::framework::Distribution>>,
        processes: std::sync::Arc<dyn super::super::processes::Processes>,
        options: &'a Options,
    ) -> Result<Self, String> {
        let configuration = Configuration::load(options)?;
        if options.force_download_framework && configuration.manifest.is_none() {
            return Err("--force-download-framework requires a tuffite.json manifest so the development package can be rebuilt".to_owned());
        }
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| format!("create development session runtime: {error}"))?;
        Ok(Self {
            framework,
            processes,
            options,
            configuration,
            runtime,
        })
    }

    pub(super) fn run(mut self) -> Result<(), String> {
        let mut application = DevelopmentApplication::prepare(
            self.framework.as_deref(),
            self.processes.as_ref(),
            &self.configuration,
            self.options.release,
        )?;
        if self.options.prepare_only {
            tracing::info!(package = %application.package.display(), "development package is ready");
            return Ok(());
        }
        if let Some(port) = self.configuration.devtools_port {
            ensure_loopback_port_available(port)?;
            tracing::info!(endpoint = %format_args!("http://127.0.0.1:{port}/"),
                targets = %format_args!("http://127.0.0.1:{port}/json/list"),
                inspect = "edge://inspect/#devices", "DevTools is enabled");
        }
        let token = DevelopmentToken::new()?;
        let mut server = WebServer::start(&self.runtime, &self.configuration.server, &token)?;
        tracing::info!(url = %self.configuration.server.url, "development Web server is ready");
        loop {
            let watch_root = (!self.options.no_watch && self.configuration.manifest.is_some())
                .then_some(self.configuration.root.as_path());
            let watcher = SourceWatcher::new(watch_root);
            tracing::info!(executable = %application.executable.display(), "launching development application");
            let command = application.command(&Launch {
                url: &self.configuration.server.url,
                token: &token,
                args: &self.options.app_arg,
                devtools_port: self.configuration.devtools_port,
            });
            let mut process = self
                .runtime
                .block_on(command.start())
                .map_err(|error| format!("launch {}: {error}", application.executable.display()))?;
            let outcome = self.wait(&mut process, &mut server, &application.executable, &watcher);
            if matches!(outcome, Ok(SessionExit::SourcesChanged)) {
                // Reap the old process tree before replacing its loaded library.
                self.runtime
                    .block_on(process.shutdown(Duration::from_millis(200)))
                    .map_err(|error| format!("stop application before rebuilding: {error}"))?;
            } else {
                drop(process);
            }
            match outcome? {
                SessionExit::Finished => return Ok(()),
                SessionExit::SourcesChanged => {
                    tracing::info!(root = %self.configuration.root.display(), "project sources changed; rebuilding");
                    application = DevelopmentApplication::prepare(
                        self.framework.as_deref(),
                        self.processes.as_ref(),
                        &self.configuration,
                        self.options.release,
                    )?;
                }
            }
        }
    }

    fn wait(
        &mut self,
        application: &mut RunningProcess,
        server: &mut WebServer,
        executable: &Path,
        watcher: &SourceWatcher,
    ) -> Result<SessionExit, String> {
        let exit = self.runtime.block_on(async {
            let mut processes = [application, &mut server.process];
            tokio::select! {
                outcome = processkit::wait_any(&mut processes) => match outcome {
                    Ok((0, outcome)) => DevExit::Application(Ok(outcome)),
                    Ok((1, outcome)) => DevExit::Web(Ok(outcome)),
                    Ok((index, _)) => unreachable!("unexpected process index {index}"),
                    Err(error) => DevExit::WaitFailed(error),
                },
                signal = tokio::signal::ctrl_c() => DevExit::Interrupted(signal),
                () = watcher.wait() => DevExit::SourcesChanged,
            }
        });
        match exit {
            DevExit::Application(Ok(outcome)) if outcome.code() == Some(0) => {
                Ok(SessionExit::Finished)
            }
            DevExit::Application(Ok(outcome)) => {
                let log_path = executable.with_file_name("tuffite.log");
                let details = std::fs::read_to_string(&log_path)
                    .ok()
                    .map(|log| {
                        log.lines()
                            .filter(|line| line.contains(":ERROR:") || line.contains(":FATAL:"))
                            .rev()
                            .take(5)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .filter(|details| !details.is_empty());
                Err(format!(
                    "development application exited with {outcome:?}; native log: {}{}",
                    log_path.display(),
                    details.map_or_else(String::new, |details| format!("\n{details}"))
                ))
            }
            DevExit::Application(Err(error)) => {
                Err(format!("wait for {}: {error}", executable.display()))
            }
            DevExit::Web(Ok(outcome)) => Err(format!(
                "Web development server exited before the application with {outcome:?}"
            )),
            DevExit::Web(Err(error)) => Err(format!("Web development server failed: {error}")),
            DevExit::Interrupted(Ok(())) => Err("development session interrupted".to_owned()),
            DevExit::Interrupted(Err(error)) => Err(format!("listen for interruption: {error}")),
            DevExit::SourcesChanged => Ok(SessionExit::SourcesChanged),
            DevExit::WaitFailed(error) => Err(format!(
                "wait for development application or Web server: {error}"
            )),
        }
    }
}

fn ensure_loopback_port_available(port: u16) -> Result<(), String> {
    TcpListener::bind(("127.0.0.1", port))
        .map(drop)
        .map_err(|error| {
            format!(
                "DevTools port {port} is unavailable on 127.0.0.1: {error}; choose another with --devtools-port"
            )
        })
}
