//! Source change detection with an explicit baseline per application generation.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

type Snapshot = BTreeMap<PathBuf, (u64, Option<SystemTime>)>;

pub(super) struct SourceWatcher {
    root: Option<PathBuf>,
    baseline: Snapshot,
}

impl SourceWatcher {
    pub(super) fn new(root: Option<&Path>) -> Self {
        Self {
            root: root.map(Path::to_path_buf),
            baseline: root.map(project_snapshot).unwrap_or_default(),
        }
    }

    pub(super) async fn wait(&self) {
        let Some(root) = self.root.as_deref() else {
            std::future::pending::<()>().await;
            return;
        };
        loop {
            tokio::time::sleep(Duration::from_millis(300)).await;
            if project_snapshot(root) != self.baseline {
                tokio::time::sleep(Duration::from_millis(200)).await;
                return;
            }
        }
    }
}

fn project_snapshot(root: &Path) -> BTreeMap<PathBuf, (u64, Option<std::time::SystemTime>)> {
    fn visit(
        root: &Path,
        directory: &Path,
        snapshot: &mut BTreeMap<PathBuf, (u64, Option<std::time::SystemTime>)>,
    ) {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = std::fs::symlink_metadata(&path) else {
                continue;
            };
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                let ignored = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        matches!(
                            name,
                            ".git" | "dist" | "generated" | "node_modules" | "target"
                        )
                    });
                if !ignored {
                    visit(root, &path, snapshot);
                }
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let watched = matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("rs" | "toml" | "mojom")
            ) || matches!(name, "build.rs" | "Cargo.lock" | "tuffite.json");
            if watched {
                let relative = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
                snapshot.insert(relative, (metadata.len(), metadata.modified().ok()));
            }
        }
    }

    let mut snapshot = BTreeMap::new();
    visit(root, root, &mut snapshot);
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_early_changes_and_ignores_build_outputs() {
        let root =
            std::env::temp_dir().join(format!("tuffite-source-watch-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("target")).unwrap();
        std::fs::write(root.join("lib.rs"), "before").unwrap();
        let watcher = SourceWatcher::new(Some(&root));
        std::fs::write(root.join("target/generated.rs"), "ignored").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&root, root.join("cycle")).unwrap();
        assert_eq!(project_snapshot(&root), watcher.baseline);
        std::fs::write(root.join("lib.rs"), "changed before launch").unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), watcher.wait())
                .await
                .unwrap();
        });
        std::fs::remove_dir_all(root).unwrap();
    }
}
