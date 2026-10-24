//! Invocation-scoped composition; workflows receive explicit service handles.
use super::processes::{NativeProcesses, Processes};
use crate::build::framework::{Distribution, Resolver};
use crate::ioc::{Factory, FactoryContext, Registry, Service};
use std::sync::Arc;

pub(super) struct Invocation {
    refresh_framework: bool,
    manifest: Option<std::path::PathBuf>,
}

trait FrameworkSource: Send + Sync {
    fn resolve(
        &self,
        refresh: bool,
        manifest: Option<&std::path::Path>,
    ) -> Result<Distribution, String>;
}
struct NativeFrameworkSource;
impl FrameworkSource for NativeFrameworkSource {
    fn resolve(
        &self,
        refresh: bool,
        manifest: Option<&std::path::Path>,
    ) -> Result<Distribution, String> {
        let mut resolver = Resolver::from_env()?;
        if let Some(manifest) = manifest {
            resolver = resolver.manifest(manifest)?;
        }
        if refresh {
            tracing::info!("forcing a fresh Tuffite Framework Release download");
            resolver = resolver.managed().auto_download(true).force_download(true);
        }
        resolver.resolve()
    }
}

struct FrameworkFactory;
impl Factory<Invocation> for FrameworkFactory {
    type Target = Service<Distribution>;
    fn build(
        &self,
        build: &mut FactoryContext<'_, Invocation>,
    ) -> Result<Arc<Distribution>, String> {
        let invocation = build.context();
        let source = build.depends_on::<Service<dyn FrameworkSource>>()?;
        source
            .resolve(invocation.refresh_framework, invocation.manifest.as_deref())
            .map(Arc::new)
    }
}

pub(super) fn registry_for_manifest(
    refresh: bool,
    manifest: Option<std::path::PathBuf>,
) -> Result<Registry<Invocation>, String> {
    registry_with_manifest_sources(
        refresh,
        manifest,
        Arc::new(NativeFrameworkSource),
        Arc::new(NativeProcesses),
    )
}
#[cfg(test)]
fn registry_with_sources(
    refresh_framework: bool,
    source: Arc<dyn FrameworkSource>,
    processes: Arc<dyn Processes>,
) -> Result<Registry<Invocation>, String> {
    registry_with_manifest_sources(refresh_framework, None, source, processes)
}
fn registry_with_manifest_sources(
    refresh_framework: bool,
    manifest: Option<std::path::PathBuf>,
    source: Arc<dyn FrameworkSource>,
    processes: Arc<dyn Processes>,
) -> Result<Registry<Invocation>, String> {
    let mut registry = Registry::new(Arc::new(Invocation {
        refresh_framework,
        manifest,
    }));
    registry.inject(source)?;
    registry.inject(processes)?;
    registry.register(FrameworkFactory)?;
    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct UnavailableSource(Arc<AtomicUsize>);
    impl FrameworkSource for UnavailableSource {
        fn resolve(
            &self,
            refresh: bool,
            _manifest: Option<&std::path::Path>,
        ) -> Result<Distribution, String> {
            assert!(refresh);
            self.0.fetch_add(1, Ordering::SeqCst);
            Err("offline fixture".to_owned())
        }
    }

    #[test]
    fn invocation_providers_are_lazy_and_failures_remain_retryable() {
        let calls = Arc::new(AtomicUsize::new(0));
        let source: Arc<dyn FrameworkSource> = Arc::new(UnavailableSource(Arc::clone(&calls)));
        let mut registry = registry_with_sources(true, source, Arc::new(NativeProcesses)).unwrap();
        registry.freeze();
        let context = registry;
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        for _ in 0..2 {
            assert!(
                context
                    .get::<Service<Distribution>>()
                    .unwrap_err()
                    .contains("offline fixture")
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        let processes = context.get::<Service<dyn Processes>>().unwrap();
        assert!(Arc::ptr_eq(
            &processes,
            &context.get::<Service<dyn Processes>>().unwrap()
        ));
    }

    struct UnavailableProcesses(Arc<AtomicUsize>);
    impl Processes for UnavailableProcesses {
        fn status(
            &self,
            _: &mut std::process::Command,
        ) -> std::io::Result<std::process::ExitStatus> {
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(std::io::Error::other("fixture process runner"))
        }
        fn output(&self, _: &mut std::process::Command) -> std::io::Result<std::process::Output> {
            Err(std::io::Error::other("fixture process runner"))
        }
        fn spawn(&self, _: &mut std::process::Command) -> std::io::Result<std::process::Child> {
            Err(std::io::Error::other("fixture process runner"))
        }
    }

    #[test]
    fn hooks_use_the_invocation_process_provider() {
        let calls = Arc::new(AtomicUsize::new(0));
        let processes: Arc<dyn Processes> = Arc::new(UnavailableProcesses(Arc::clone(&calls)));
        let mut registry =
            registry_with_sources(false, Arc::new(NativeFrameworkSource), processes).unwrap();
        registry.freeze();
        let context = registry;
        let root = std::env::temp_dir().join(format!("tuffite-ioc-hook-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let manifest = root.join("tuffite.json");
        std::fs::write(
            &manifest,
            r#"{"build":{"beforeBuildCommand":"fixture must never execute"}}"#,
        )
        .unwrap();
        let processes = context.get::<Service<dyn Processes>>().unwrap();
        let error = super::super::hooks::run(&manifest, "beforeBuildCommand", processes.as_ref())
            .unwrap_err();
        assert!(error.contains("fixture process runner"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn command_scopes_validate_before_resolving_native_framework() {
        use super::super::Cli;
        use clap::Parser as _;
        let calls = Arc::new(AtomicUsize::new(0));
        let source: Arc<dyn FrameworkSource> = Arc::new(UnavailableSource(Arc::clone(&calls)));
        let mut parent = registry_with_sources(true, source, Arc::new(NativeProcesses)).unwrap();
        parent.freeze();
        let root =
            std::env::temp_dir().join(format!("tuffite-command-scopes-{}", std::process::id()));
        let directory = root.to_str().unwrap();
        let init = Cli::try_parse_from(["tuff", "init", directory]).unwrap();
        init.command.run(&parent).unwrap();
        let manifest = root.join("tuffite.json");
        let check =
            Cli::try_parse_from(["tuff", "check", "--manifest", manifest.to_str().unwrap()])
                .unwrap();
        check.command.run(&parent).unwrap();
        let verify = Cli::try_parse_from(["tuff", "framework", "verify", directory]).unwrap();
        assert!(verify.command.run(&parent).is_err());
        let invalid = root.join("invalid.json");
        std::fs::write(&invalid, r#"{"app":false}"#).unwrap();
        for command in ["build", "doctor", "dev", "package"] {
            let parsed =
                Cli::try_parse_from(["tuff", command, "--manifest", invalid.to_str().unwrap()])
                    .unwrap();
            let error = parsed.command.run(&parent).unwrap_err();
            assert!(
                !error.to_string().contains("offline fixture"),
                "{command}: {error}"
            );
        }
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let build =
            Cli::try_parse_from(["tuff", "build", "--manifest", manifest.to_str().unwrap()])
                .unwrap();
        let error = build.command.run(&parent).unwrap_err();
        assert!(error.to_string().contains("offline fixture"));
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_dev_package_does_not_resolve_framework() {
        use super::super::Cli;
        use clap::Parser as _;
        let calls = Arc::new(AtomicUsize::new(0));
        let source: Arc<dyn FrameworkSource> = Arc::new(UnavailableSource(Arc::clone(&calls)));
        let mut parent = registry_with_sources(true, source, Arc::new(NativeProcesses)).unwrap();
        parent.freeze();
        let root =
            std::env::temp_dir().join(format!("tuffite-existing-dev-{}", std::process::id()));
        let parsed = Cli::try_parse_from([
            "tuff",
            "dev",
            "--package",
            root.to_str().unwrap(),
            "--url",
            "http://127.0.0.1:5173/",
            "--prepare-only",
        ])
        .unwrap();
        let error = parsed.command.run(&parent).unwrap_err();
        assert!(!error.to_string().contains("offline fixture"));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
