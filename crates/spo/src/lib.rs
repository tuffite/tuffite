//! Git-backed patch series. Files are ordered mailboxes, not permanent commit IDs.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod ui;

pub const UPSTREAM_REF: &str = "refs/spo/upstream";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigTarget {
    name: Option<String>,
    repo: PathBuf,
    patch_dir: PathBuf,
    base: Option<String>,
    base_ref: Option<String>,
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {
    pub name: String,
    pub repo: PathBuf,
    pub patch_dir: PathBuf,
    pub base: String,
    pub exclude: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Group {
    pub name: String,
    pub commits: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Snapshot {
    base: String,
    groups: Vec<Group>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Operation {
    kind: String,
    original: String,
    base: String,
    phase: String,
    cursor: usize,
    groups: Vec<Group>,
    tips: Vec<(String, String)>,
    stash: Option<String>,
    staging: Option<PathBuf>,
    backup: Option<PathBuf>,
    files: BTreeMap<String, String>,
}

fn is_patch_file(value: &str) -> bool {
    Path::new(value)
        .extension()
        .is_some_and(|extension| extension == "patch")
}

#[derive(Debug)]
pub struct Status {
    pub head: String,
    pub commits: usize,
    pub files: usize,
    pub dirty: bool,
    pub pending: Option<String>,
    pub current: bool,
}

#[derive(Clone, Debug)]
pub struct PlanEntry {
    pub action: String,
    pub commit: String,
    pub group: String,
}

#[must_use]
pub fn unique() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}

pub fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = git_bytes(repo, args, None)?;
    Ok(String::from_utf8(output)?.trim().to_owned())
}

pub fn git_bytes(repo: &Path, args: &[&str], input: Option<&[u8]>) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    command
        .arg("-C")
        .arg(repo)
        .args(args)
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("start Git")?;
    if let Some(data) = input {
        child.stdin.take().context("Git stdin")?.write_all(data)?;
    }
    let output = child.wait_with_output()?;
    ensure!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

pub fn run_git(repo: &Path, args: &[&str]) -> Result<()> {
    ensure!(
        Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()?
            .success(),
        "Git stopped. Resolve the current operation, then run `cargo spo continue {}`.",
        repo.display()
    );
    Ok(())
}

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    fs::create_dir_all(path.parent().context("state parent")?)?;
    let temporary = path.with_extension("tmp");
    let mut file = File::create(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn scoped_path(root: &Path, value: &Path) -> Result<PathBuf> {
    ensure!(
        !value.is_absolute()
            && value
                .components()
                .all(|c| matches!(c, Component::Normal(_))),
        "unsafe workspace path: {}",
        value.display()
    );
    let path = root.join(value);
    // Resolve the closest existing ancestor too, including during interrupted publication.
    let mut ancestor = path.as_path();
    while !ancestor.exists() {
        ancestor = ancestor.parent().context("workspace path parent")?;
    }
    ensure!(
        ancestor.canonicalize()?.starts_with(root.canonicalize()?),
        "path escapes workspace: {}",
        value.display()
    );
    Ok(path)
}

pub fn load_targets(workspace: &Path, config: &Path) -> Result<Vec<Target>> {
    load_targets_with_bases(workspace, config, &BTreeMap::new())
}

/// Integration layers supply desired bases without teaching spo their dependency format.
pub fn load_targets_with_bases(
    workspace: &Path,
    config: &Path,
    bases: &BTreeMap<String, String>,
) -> Result<Vec<Target>> {
    let entries: Vec<ConfigTarget> = serde_json::from_slice(
        &fs::read(config)
            .with_context(|| format!("read patch configuration {}", config.display()))?,
    )?;
    ensure!(!entries.is_empty(), "empty patch target configuration");
    let mut names = BTreeSet::new();
    let mut repos = BTreeSet::new();
    let mut directories = BTreeSet::new();
    let mut targets = Vec::new();
    for entry in entries {
        let repo = scoped_path(workspace, &entry.repo)?;
        let patch_dir = scoped_path(workspace, &entry.patch_dir)?;
        // An interrupted export may temporarily move the directory to its backup.
        // Otherwise reject stale checkout paths before baseline repair or any mutation.
        if !patch_dir.is_dir() {
            let target = Target {
                name: entry.name.clone().unwrap_or_default(),
                repo: repo.clone(),
                patch_dir: patch_dir.clone(),
                base: String::new(),
                exclude: entry.exclude.clone(),
            };
            ensure!(
                target.pending()?.is_some(),
                "configured patch directory does not exist: {} (config: {}). Workspace-relative paths must match the checkout directory; after a project rename, migrate the checkout layout and .gclient together. No patches were applied",
                patch_dir.display(),
                config.display()
            );
        }
        let name = entry.name.unwrap_or_else(|| {
            entry
                .patch_dir
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        });
        ensure!(
            !name.is_empty() && names.insert(name.clone()),
            "duplicate or empty target name"
        );
        ensure!(
            repos.insert(repo.canonicalize()?) && directories.insert(patch_dir.clone()),
            "targets must have distinct repositories and patch directories"
        );
        ensure!(
            entry.base.is_none() || entry.base_ref.is_none(),
            "choose base or base_ref, not both"
        );
        let value = bases
            .get(&name)
            .map(String::as_str)
            .or(entry.base.as_deref())
            .or(entry.base_ref.as_deref())
            .unwrap_or(UPSTREAM_REF);
        let base = if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            value.to_owned()
        } else {
            let target = Target {
                name: name.clone(),
                repo: repo.clone(),
                patch_dir: patch_dir.clone(),
                base: String::new(),
                exclude: entry.exclude.clone(),
            };
            if value == UPSTREAM_REF {
                target.upstream_base()?.with_context(|| format!(
                    "missing base {value} for {name}; cannot verify a baseline from local state or the checked-in series. Run the integration bootstrap, or initialize a known upstream commit with `cargo spo --base {name}=<upstream-commit> status {name}`. Do not use patched HEAD as the base"
                ))?
            } else {
                git(
                    &repo,
                    &["rev-parse", "--verify", &format!("{value}^{{commit}}")],
                )
                .with_context(|| {
                    format!(
                        "missing base {value} for {name}; configure an existing commit or reference"
                    )
                })?
            }
        };
        targets.push(Target {
            name,
            repo,
            patch_dir,
            base,
            exclude: entry.exclude,
        });
    }
    Ok(targets)
}

pub fn mailbox_messages(data: &[u8]) -> Result<Vec<&[u8]>> {
    let mut offsets = Vec::new();
    let mut position = 0;
    for line in data.split_inclusive(|b| *b == b'\n') {
        if line.len() >= 46
            && line.starts_with(b"From ")
            && line[5..45].iter().all(u8::is_ascii_hexdigit)
            && line[45] == b' '
        {
            offsets.push(position);
        }
        position += line.len();
    }
    ensure!(
        offsets.first() == Some(&0),
        "patch must be a Git format-patch mailbox"
    );
    offsets.push(data.len());
    Ok(offsets
        .windows(2)
        .map(|pair| &data[pair[0]..pair[1]])
        .collect())
}

fn patch_ids(repo: &Path, data: &[u8]) -> Result<Vec<String>> {
    mailbox_messages(data)?
        .into_iter()
        .map(|message| {
            let result = git_bytes(repo, &["patch-id", "--stable"], Some(message))?;
            Ok(String::from_utf8(result)?
                .split_whitespace()
                .next()
                .unwrap_or("empty")
                .to_owned())
        })
        .collect()
}

pub fn valid_name(value: &str) -> Result<()> {
    ensure!(
        is_patch_file(value)
            && !value.starts_with('.')
            && !value.contains(['/', '\\', '\n', '\r', '\0'])
            && value != ".patch",
        "invalid patch filename: {value}"
    );
    Ok(())
}

fn slug(subject: &str) -> String {
    let mut value = String::new();
    for c in subject.chars() {
        if c.is_ascii_alphanumeric() || c == '-' {
            value.push(c.to_ascii_lowercase());
        } else if !value.ends_with('_') {
            value.push('_');
        }
    }
    format!("{}.patch", value.trim_matches('_'))
}

impl Target {
    /// Recover a baseline from Git state or a fully matching checked-in stack.
    /// This method is read-only; HEAD is never used as an unverified fallback.
    pub fn upstream_base(&self) -> Result<Option<String>> {
        if let Ok(base) = git(
            &self.repo,
            &[
                "rev-parse",
                "--verify",
                &format!("{UPSTREAM_REF}^{{commit}}"),
            ],
        ) {
            return Ok(Some(base));
        }
        if let Some(operation) = self.operation()? {
            return git(
                &self.repo,
                &[
                    "rev-parse",
                    "--verify",
                    &format!("{}^{{commit}}", operation.base),
                ],
            )
            .map(Some)
            .context("interrupted operation's base commit is missing");
        }
        let snapshot = self.state_dir()?.join("stack.json");
        if snapshot.exists() {
            let snapshot: Snapshot = serde_json::from_slice(&fs::read(snapshot)?)?;
            let base = git(
                &self.repo,
                &[
                    "rev-parse",
                    "--verify",
                    &format!("{}^{{commit}}", snapshot.base),
                ],
            )?;
            self.commits(&base)
                .context("recorded baseline is no longer an ancestor of HEAD")?;
            return Ok(Some(base));
        }
        if let Ok(base) = git(
            &self.repo,
            &[
                "rev-parse",
                "--verify",
                "refs/tuffite/upstream-head^{commit}",
            ],
        ) {
            self.commits(&base)?;
            return Ok(Some(base));
        }
        if self.native_operation()?.is_some() {
            return Ok(None);
        }
        let expected = self.expected_ids()?;
        if expected.is_empty() {
            return Ok(None);
        }
        let Ok(base) = git(
            &self.repo,
            &[
                "rev-parse",
                "--verify",
                &format!("HEAD~{}^{{commit}}", expected.len()),
            ],
        ) else {
            return Ok(None);
        };
        let candidate = Self {
            base: base.clone(),
            ..self.clone()
        };
        if candidate.status()?.current {
            return Ok(Some(base));
        }
        Ok(None)
    }

    /// Initialize a missing derived reference from an explicit or verified base.
    pub fn initialize_upstream(&self, base: &str) -> Result<()> {
        let base = git(
            &self.repo,
            &["rev-parse", "--verify", &format!("{base}^{{commit}}")],
        )
        .context("upstream commit is unavailable; sync/fetch that exact commit first")?;
        if let Ok(existing) = git(&self.repo, &["rev-parse", "--verify", UPSTREAM_REF]) {
            ensure!(
                existing == base,
                "{UPSTREAM_REF} already records {existing}; finish the current roll before changing it"
            );
            return Ok(());
        }
        ensure!(
            self.pending()?.is_none() && self.native_operation()?.is_none(),
            "finish the interrupted Git operation before initializing the base"
        );
        self.commits(&base).context(
            "upstream must be an ancestor of HEAD; do not initialize from a different checkout",
        )?;
        git(
            &self.repo,
            &[
                "update-ref",
                UPSTREAM_REF,
                &base,
                "0000000000000000000000000000000000000000",
            ],
        )?;
        eprintln!("{}: initialized {UPSTREAM_REF} at {base}", self.name);
        Ok(())
    }

    /// Persist recovered metadata, without interfering with an active operation.
    pub fn repair_upstream(&self) -> Result<()> {
        if git(&self.repo, &["rev-parse", "--verify", UPSTREAM_REF]).is_err()
            && self.pending()?.is_none()
            && self.native_operation()?.is_none()
            && let Some(base) = self.upstream_base()?
        {
            self.initialize_upstream(&base)?;
        }
        Ok(())
    }

    pub fn state_dir(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(git(
            &self.repo,
            &["rev-parse", "--path-format=absolute", "--git-path", "spo"],
        )?))
    }

    fn lock(&self) -> Result<File> {
        let root = self.state_dir()?;
        fs::create_dir_all(&root)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("lock"))?;
        file.try_lock().map_err(|error| {
            anyhow::anyhow!("another spo operation owns {}: {error}", self.name)
        })?;
        Ok(file)
    }

    fn operation(&self) -> Result<Option<Operation>> {
        let path = self.state_dir()?.join("operation.json");
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
    }

    fn save_operation(&self, operation: &Operation) -> Result<()> {
        write_json(&self.state_dir()?.join("operation.json"), operation)
    }

    fn save_snapshot(&self, base: &str, groups: &[Group]) -> Result<()> {
        write_json(
            &self.state_dir()?.join("stack.json"),
            &Snapshot {
                base: base.into(),
                groups: groups.to_vec(),
            },
        )
    }

    pub fn record_stack(&self, base: &str, groups: &[Group]) -> Result<()> {
        ensure!(
            self.commits(base)?
                == groups
                    .iter()
                    .flat_map(|g| g.commits.clone())
                    .collect::<Vec<_>>(),
            "stack association does not cover HEAD"
        );
        self.save_snapshot(base, groups)
    }

    pub fn pending(&self) -> Result<Option<String>> {
        if let Some(operation) = self.operation()? {
            return Ok(Some(format!("{}: {}", operation.kind, operation.phase)));
        }
        self.native_operation()
    }

    pub fn native_operation(&self) -> Result<Option<String>> {
        for (path, name) in [
            ("rebase-apply", "am"),
            ("rebase-merge", "rebase"),
            ("MERGE_HEAD", "merge"),
            ("CHERRY_PICK_HEAD", "cherry-pick"),
        ] {
            let path = PathBuf::from(git(
                &self.repo,
                &["rev-parse", "--path-format=absolute", "--git-path", path],
            )?);
            if path.exists() {
                return Ok(Some(name.into()));
            }
        }
        Ok(None)
    }

    /// Repository-root pathspecs shared by WIP inspection and preservation.
    pub fn work_paths(&self) -> Result<Vec<String>> {
        let mut paths = vec![".".into()];
        for path in &self.exclude {
            ensure!(
                !path.is_empty()
                    && !Path::new(path).is_absolute()
                    && Path::new(path)
                        .components()
                        .all(|part| matches!(part, Component::Normal(_))),
                "exclude paths must be repository-relative: {path}"
            );
            paths.push(format!(":(top,literal,exclude){path}"));
        }
        Ok(paths)
    }

    /// Enumerate literal owned paths, including independent index/worktree changes.
    pub fn changed_paths(&self) -> Result<Vec<String>> {
        let paths = self.work_paths()?;
        let mut changed = BTreeSet::new();
        for mut args in [
            vec!["diff", "--no-renames", "--name-only", "-z", "--"],
            vec![
                "diff",
                "--cached",
                "--no-renames",
                "--name-only",
                "-z",
                "--",
            ],
            vec!["ls-files", "--others", "--exclude-standard", "-z", "--"],
        ] {
            args.extend(paths.iter().map(String::as_str));
            for path in git_bytes(&self.repo, &args, None)?.split(|byte| *byte == 0) {
                if !path.is_empty() {
                    changed.insert(String::from_utf8(path.to_vec())?);
                }
            }
        }
        Ok(changed.into_iter().collect())
    }

    pub fn dirty(&self) -> Result<bool> {
        let paths = self.work_paths()?;
        let mut args = vec!["status", "--porcelain", "--untracked-files=normal", "--"];
        args.extend(paths.iter().map(String::as_str));
        Ok(!git(&self.repo, &args)?.is_empty())
    }

    pub fn commits(&self, base: &str) -> Result<Vec<String>> {
        ensure!(
            Command::new("git")
                .arg("-C")
                .arg(&self.repo)
                .args(["merge-base", "--is-ancestor", base, "HEAD"])
                .status()?
                .success(),
            "{} is not based on {base}; roll or choose the correct base",
            self.name
        );
        let range = format!("{base}..HEAD");
        ensure!(
            git(&self.repo, &["rev-list", "--merges", &range])?.is_empty(),
            "spo currently supports linear stacks only"
        );
        Ok(git(&self.repo, &["rev-list", "--reverse", &range])?
            .lines()
            .map(str::to_owned)
            .collect())
    }

    pub fn filenames(&self) -> Result<Vec<String>> {
        let path = self.patch_dir.join(".patches");
        let manifest = fs::read_to_string(&path).with_context(|| {
            format!(
                "read patch manifest {}; check patch_dir and the checkout layout (a missing manifest is not an empty series)",
                path.display()
            )
        })?;
        let mut names = BTreeSet::new();
        let mut result = Vec::new();
        for line in manifest
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            valid_name(line)?;
            ensure!(
                names.insert(line.to_owned()),
                "duplicate patch in .patches: {line}"
            );
            let path = self.patch_dir.join(line);
            ensure!(
                path.is_file()
                    && path.canonicalize()?.parent()
                        == Some(self.patch_dir.canonicalize()?.as_path()),
                "missing or unsafe patch: {line}"
            );
            result.push(line.into());
        }
        Ok(result)
    }

    pub fn format_commit(&self, commit: &str) -> Result<Vec<u8>> {
        for path in &self.exclude {
            let files = git(
                &self.repo,
                &[
                    "diff-tree",
                    "--no-commit-id",
                    "--name-only",
                    "-r",
                    commit,
                    "--",
                    path,
                ],
            )?;
            ensure!(
                files.is_empty(),
                "commit {commit} includes separately owned files under {path}; move them into their owning repository"
            );
        }
        git_bytes(
            &self.repo,
            &[
                "-c",
                "diff.renames=true",
                "format-patch",
                "-1",
                "--always",
                "--stdout",
                "--binary",
                "--keep-subject",
                "--no-stat",
                "--no-signature",
                "--zero-commit",
                "--full-index",
                commit,
            ],
            None,
        )
    }

    fn expected_ids(&self) -> Result<Vec<String>> {
        let mut ids = Vec::new();
        for name in self.filenames()? {
            ids.extend(patch_ids(
                &self.repo,
                &fs::read(self.patch_dir.join(name))?,
            )?);
        }
        Ok(ids)
    }

    pub fn groups(&self, base: &str) -> Result<Vec<Group>> {
        let commits = self.commits(base)?;
        let snapshot_path = self.state_dir()?.join("stack.json");
        let snapshot: Option<Snapshot> = if snapshot_path.exists() {
            Some(serde_json::from_slice(&fs::read(snapshot_path)?)?)
        } else {
            None
        };
        let mut membership = BTreeMap::new();
        if let Some(snapshot) = snapshot.filter(|s| s.base == base) {
            let known: BTreeSet<_> = snapshot.groups.iter().flat_map(|g| &g.commits).collect();
            ensure!(
                known.iter().all(|sha| commits.contains(sha)),
                "stack history changed outside spo; use `spo group --commits …` to explicitly reassociate commits, or import the checked-in series in a fresh branch"
            );
            for group in snapshot.groups {
                for commit in group.commits {
                    membership.insert(commit, group.name.clone());
                }
            }
        } else {
            let actual = commits
                .iter()
                .map(|c| patch_ids(&self.repo, &self.format_commit(c)?).map(|v| v[0].clone()))
                .collect::<Result<Vec<_>>>()?;
            let mut cursor = 0;
            for name in self.filenames()? {
                let expected = patch_ids(&self.repo, &fs::read(self.patch_dir.join(&name))?)?;
                if actual.get(cursor..cursor + expected.len()) != Some(expected.as_slice()) {
                    break;
                }
                for commit in &commits[cursor..cursor + expected.len()] {
                    membership.insert(commit.clone(), name.clone());
                }
                cursor += expected.len();
            }
        }
        let mut groups: Vec<Group> = Vec::new();
        let mut finished = BTreeSet::new();
        for commit in commits {
            let name = if let Some(name) = membership.get(&commit) {
                name.clone()
            } else {
                slug(&git(&self.repo, &["show", "-s", "--format=%s", &commit])?)
            };
            valid_name(&name)?;
            if let Some(last) = groups.last_mut().filter(|group| group.name == name) {
                last.commits.push(commit);
            } else {
                ensure!(
                    finished.insert(name.clone()),
                    "non-contiguous or duplicate patch group {name}; assign an explicit group"
                );
                groups.push(Group {
                    name,
                    commits: vec![commit],
                });
            }
        }
        Ok(groups)
    }

    pub fn status(&self) -> Result<Status> {
        let head = git(&self.repo, &["rev-parse", "HEAD"])?;
        let pending = self.pending()?;
        let filenames = self.filenames()?;
        let expected = self.expected_ids()?;
        let commits = self.commits(&self.base).unwrap_or_default();
        let actual = commits
            .iter()
            .map(|c| patch_ids(&self.repo, &self.format_commit(c)?).map(|v| v[0].clone()))
            .collect::<Result<Vec<_>>>()?;
        Ok(Status {
            head,
            commits: commits.len(),
            files: filenames.len(),
            dirty: self.dirty()?,
            pending,
            current: !expected.is_empty() && actual == expected,
        })
    }

    pub fn group(&self, name: &str, commits: &[String]) -> Result<()> {
        let _lock = self.lock()?;
        ensure!(
            self.pending()?.is_none(),
            "finish the current operation first"
        );
        valid_name(name)?;
        ensure!(!commits.is_empty(), "select at least one commit");
        let stack = self.commits(&self.base)?;
        let selected = commits
            .iter()
            .map(|sha| git(&self.repo, &["rev-parse", "--verify", sha]))
            .collect::<Result<Vec<_>>>()?;
        let first = stack
            .iter()
            .position(|sha| sha == &selected[0])
            .context("commit is outside the stack")?;
        ensure!(
            stack.get(first..first + selected.len()) == Some(selected.as_slice()),
            "group commits must be consecutive and ordered"
        );
        // Explicit reassociation remains possible after an external rebase.
        let mut groups = self.groups(&self.base).unwrap_or_else(|_| {
            stack
                .iter()
                .enumerate()
                .map(|(i, sha)| Group {
                    name: format!("unassigned_{i}.patch"),
                    commits: vec![sha.clone()],
                })
                .collect()
        });
        let mut assignments = BTreeMap::new();
        for group in groups.drain(..) {
            for commit in group.commits {
                assignments.insert(commit, group.name.clone());
            }
        }
        for commit in selected {
            assignments.insert(commit, name.into());
        }
        let mut grouped: Vec<Group> = Vec::new();
        let mut names = BTreeSet::new();
        for commit in stack {
            let name = assignments.remove(&commit).context("unassigned commit")?;
            if let Some(last) = grouped.last_mut().filter(|g| g.name == name) {
                last.commits.push(commit);
            } else {
                ensure!(
                    names.insert(name.clone()),
                    "group {name} would be non-contiguous"
                );
                grouped.push(Group {
                    name,
                    commits: vec![commit],
                });
            }
        }
        self.save_snapshot(&self.base, &grouped)
    }

    fn begin(&self, kind: &str, groups: Vec<Group>, stash: bool) -> Result<Operation> {
        ensure!(
            self.pending()?.is_none(),
            "unfinished operation; run `cargo spo continue {}` or `cargo spo abort {}`",
            self.name,
            self.name
        );
        ensure!(
            stash || !self.dirty()?,
            "working tree has changes; choose --dirty stash to preserve WIP"
        );
        let mut files = BTreeMap::new();
        for name in self.filenames()? {
            files.insert(
                name.clone(),
                git(
                    &self.repo,
                    &[
                        "hash-object",
                        self.patch_dir
                            .join(name)
                            .to_str()
                            .context("patch path encoding")?,
                    ],
                )?,
            );
        }
        let operation = Operation {
            kind: kind.into(),
            original: git(&self.repo, &["rev-parse", "HEAD"])?,
            base: self.base.clone(),
            phase: "prepared".into(),
            cursor: 0,
            groups,
            tips: Vec::new(),
            stash: None,
            staging: None,
            backup: None,
            files,
        };
        self.save_operation(&operation)?;
        git(
            &self.repo,
            &[
                "update-ref",
                &format!("refs/spo/backups/{}", unique()),
                &operation.original,
            ],
        )?;
        let mut operation = operation;
        if self.dirty()? {
            ensure!(
                stash,
                "working tree has changes; choose `--dirty stash` to preserve WIP"
            );
            // Write the intent first; a crash while saving WIP is diagnosed, never blindly repeated.
            operation.phase = "saving-wip".into();
            self.save_operation(&operation)?;
            operation.stash = self.stash()?;
            operation.phase = "prepared".into();
            self.save_operation(&operation)?;
        }
        Ok(operation)
    }

    pub fn stash(&self) -> Result<Option<String>> {
        if !self.dirty()? {
            return Ok(None);
        }
        // Negative pathspecs make stash's internal git-add reject ignored exclusions.
        // Send only literal owned paths, NUL-delimited so names and large WIP are safe.
        let paths = self.changed_paths()?;
        ensure!(
            !paths.is_empty(),
            "working tree changed while preparing the stash"
        );
        let mut input = Vec::new();
        for path in paths {
            input.extend_from_slice(format!(":(top,literal){path}").as_bytes());
            input.push(0);
        }
        let message = format!("spo-{}", unique());
        git_bytes(
            &self.repo,
            &[
                "stash",
                "push",
                "--include-untracked",
                "--message",
                &message,
                "--pathspec-from-file=-",
                "--pathspec-file-nul",
            ],
            Some(&input),
        )?;
        let oid = git(&self.repo, &["rev-parse", "refs/stash"])?;
        git(
            &self.repo,
            &["update-ref", &format!("refs/spo/wip/{oid}"), &oid],
        )?;
        Ok(Some(oid))
    }

    pub fn restore_stash(&self, oid: &str) -> Result<()> {
        run_git(&self.repo, &["stash", "apply", "--index", oid])
    }

    pub fn drop_stash(&self, oid: &str) -> Result<()> {
        for line in git(&self.repo, &["stash", "list", "--format=%gd %H"])?.lines() {
            if let Some((reference, sha)) = line.split_once(' ')
                && sha == oid
            {
                git(&self.repo, &["stash", "drop", reference])?;
                break;
            }
        }
        git(
            &self.repo,
            &["update-ref", "-d", &format!("refs/spo/wip/{oid}")],
        )?;
        Ok(())
    }

    fn finish(&self, operation: &mut Operation) -> Result<()> {
        if let Some(oid) = operation.stash.clone() {
            if operation.phase != "restored" {
                ensure!(
                    operation.phase != "restoring-wip",
                    "WIP restoration was interrupted; resolve or inspect it, then use `continue --wip-restored`. Saved WIP: {oid}"
                );
                ensure!(
                    !self.dirty()?,
                    "save new working changes before restoring WIP {oid}"
                );
                operation.phase = "restoring-wip".into();
                self.save_operation(operation)?;
                self.restore_stash(&oid)?;
                operation.phase = "restored".into();
                self.save_operation(operation)?;
            }
            self.drop_stash(&oid)?;
        }
        if let Some(backup) = &operation.backup
            && backup.exists()
        {
            fs::remove_dir_all(backup)?;
        }
        if let Some(staging) = &operation.staging
            && staging.exists()
        {
            fs::remove_dir_all(staging)?;
        }
        fs::remove_file(self.state_dir()?.join("operation.json"))?;
        Ok(())
    }

    pub fn apply(&self, stash: bool) -> Result<()> {
        let _lock = self.lock()?;
        ensure!(
            self.pending()?.is_none(),
            "unfinished operation; continue or abort first"
        );
        let head = git(&self.repo, &["rev-parse", "HEAD"])?;
        if head != self.base {
            let commits = self.commits(&self.base)?;
            let ids = commits
                .iter()
                .map(|c| patch_ids(&self.repo, &self.format_commit(c)?).map(|v| v[0].clone()))
                .collect::<Result<Vec<_>>>()?;
            let expected = self.expected_ids()?;
            ensure!(
                !expected.is_empty() && ids.starts_with(&expected),
                "{} contains a different local stack; export it or use roll instead of overwriting it",
                self.name
            );
            // Replay checks file modes and binary content as well as patch-id.
            self.verify_apply(&self.patch_dir, Some(&commits[expected.len() - 1]))?;
            self.save_snapshot(&self.base, &self.groups(&self.base)?)?;
            git(&self.repo, &["update-ref", UPSTREAM_REF, &self.base])?;
            println!("{}: patches already applied", self.name);
            return Ok(());
        }
        let mut groups = Vec::new();
        for name in self.filenames()? {
            groups.push(Group {
                name,
                commits: Vec::new(),
            });
        }
        let mut operation = self.begin("apply", groups, stash)?;
        self.resume_apply(&mut operation)
    }

    fn unchanged_files(&self, operation: &Operation) -> Result<()> {
        ensure!(
            self.filenames()?
                == operation
                    .groups
                    .iter()
                    .map(|g| g.name.clone())
                    .collect::<Vec<_>>(),
            "patch manifest changed during the operation"
        );
        for (name, sha) in &operation.files {
            ensure!(
                git(
                    &self.repo,
                    &[
                        "hash-object",
                        self.patch_dir.join(name).to_str().context("patch path")?
                    ]
                )? == *sha,
                "patch {name} changed during the operation"
            );
        }
        Ok(())
    }

    fn resume_apply(&self, operation: &mut Operation) -> Result<()> {
        self.unchanged_files(operation)?;
        if operation.phase == "restoring-wip" || operation.phase == "restored" {
            return self.finish(operation);
        }
        while operation.cursor < operation.groups.len() {
            let before = operation
                .tips
                .last()
                .map_or(operation.original.clone(), |(_, sha)| sha.clone());
            let name = operation.groups[operation.cursor].name.clone();
            if operation.phase == "applying" {
                match self.native_operation()?.as_deref() {
                    Some("am") => run_git(&self.repo, &["am", "--continue"])?,
                    None => {
                        ensure!(
                            git(&self.repo, &["rev-parse", "HEAD"])? != before,
                            "application did not start or was aborted; run `spo abort` before retrying"
                        );
                    }
                    _ => bail!("a different Git operation is active"),
                }
            } else {
                ensure!(!self.dirty()?, "working tree changed during application");
                operation.phase = "applying".into();
                self.save_operation(operation)?;
                run_git(
                    &self.repo,
                    &[
                        "-c",
                        "commit.gpgsign=false",
                        "am",
                        "--3way",
                        "--empty=keep",
                        "--keep-cr",
                        "--keep-non-patch",
                        self.patch_dir.join(&name).to_str().context("patch path")?,
                    ],
                )?;
            }
            let tip = git(&self.repo, &["rev-parse", "HEAD"])?;
            let commits: Vec<_> = git(
                &self.repo,
                &["rev-list", "--reverse", &format!("{before}..{tip}")],
            )?
            .lines()
            .map(str::to_owned)
            .collect();
            let expected = patch_ids(&self.repo, &fs::read(self.patch_dir.join(&name))?)?;
            ensure!(
                commits.len() == expected.len(),
                "applied commit count changed; skipping patches requires explicit reassociation"
            );
            operation.groups[operation.cursor].commits = commits;
            operation.tips.push((name, tip));
            operation.cursor += 1;
            operation.phase = "prepared".into();
            self.save_operation(operation)?;
        }
        self.save_snapshot(&operation.base, &operation.groups)?;
        git(&self.repo, &["update-ref", UPSTREAM_REF, &operation.base])?;
        println!(
            "{}: applied {} patch files",
            self.name,
            operation.groups.len()
        );
        self.finish(operation)
    }

    pub fn verify_apply(&self, directory: &Path, expected: Option<&String>) -> Result<()> {
        let worktree = self.state_dir()?.join(format!("verify-{}", unique()));
        git(
            &self.repo,
            &[
                "worktree",
                "add",
                "--detach",
                "--no-checkout",
                worktree.to_str().context("worktree path")?,
                &self.base,
            ],
        )?;
        let result = (|| {
            let contents = fs::read_to_string(directory.join(".patches"))?;
            let patterns = self.replay_patterns(directory, &contents)?;
            git_bytes(
                &worktree,
                &["sparse-checkout", "set", "--no-cone", "--stdin"],
                Some(patterns.as_bytes()),
            )?;
            git(&worktree, &["checkout", "--detach", &self.base])?;
            for name in contents
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
            {
                valid_name(name)?;
                git(
                    &worktree,
                    &[
                        "-c",
                        "user.name=spo",
                        "-c",
                        "user.email=spo@localhost",
                        "-c",
                        "commit.gpgsign=false",
                        "am",
                        "--3way",
                        "--empty=keep",
                        "--keep-cr",
                        "--keep-non-patch",
                        directory.join(name).to_str().context("patch path")?,
                    ],
                )?;
            }
            if let Some(expected) = expected {
                ensure!(
                    git(&worktree, &["rev-parse", "HEAD^{tree}"])?
                        == git(&self.repo, &["rev-parse", &format!("{expected}^{{tree}}")])?,
                    "replayed series produces a different tree"
                );
            }
            Ok(())
        })();
        let cleanup = git(
            &self.repo,
            &[
                "worktree",
                "remove",
                "--force",
                worktree.to_str().context("worktree path")?,
            ],
        );
        result?;
        cleanup?;
        Ok(())
    }

    fn replay_patterns(&self, directory: &Path, contents: &str) -> Result<String> {
        let mut paths = BTreeSet::new();
        for name in contents
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let data = fs::read(directory.join(name))?;
            for message in mailbox_messages(&data)? {
                let stats = git_bytes(
                    &self.repo,
                    &["apply", "--allow-empty", "--numstat", "-z"],
                    Some(message),
                )?;
                let mut records = stats
                    .split(|byte| *byte == 0)
                    .filter(|record| !record.is_empty());
                while let Some(record) = records.next() {
                    let mut fields = record.splitn(3, |byte| *byte == b'\t');
                    fields.next();
                    fields.next();
                    let path = fields.next().context("patch file statistics")?;
                    if path.is_empty() {
                        paths.insert(String::from_utf8(
                            records.next().context("rename source")?.to_vec(),
                        )?);
                        paths.insert(String::from_utf8(
                            records.next().context("rename destination")?.to_vec(),
                        )?);
                    } else {
                        paths.insert(String::from_utf8(path.to_vec())?);
                    }
                }
            }
        }
        let mut patterns = String::new();
        for path in paths {
            ensure!(
                !path.contains(['\n', '\r'])
                    && !Path::new(&path).is_absolute()
                    && Path::new(&path)
                        .components()
                        .all(|part| matches!(part, Component::Normal(_))),
                "unsupported patch path {path:?}"
            );
            let escaped = path
                .replace('\\', "\\\\")
                .replace('*', "\\*")
                .replace('?', "\\?")
                .replace('[', "\\[");
            writeln!(patterns, "/{escaped}")?;
        }
        Ok(patterns)
    }

    fn formatted_groups(&self, groups: &[Group]) -> Result<BTreeMap<String, Vec<u8>>> {
        let mut result = BTreeMap::new();
        for group in groups {
            valid_name(&group.name)?;
            ensure!(!group.commits.is_empty(), "empty group {}", group.name);
            let mut data = Vec::new();
            for commit in &group.commits {
                data.extend(self.format_commit(commit)?);
            }
            ensure!(
                result.insert(group.name.clone(), data).is_none(),
                "duplicate group {}",
                group.name
            );
        }
        let mut manifest = String::new();
        for group in groups {
            writeln!(manifest, "{}", group.name)?;
        }
        result.insert(".patches".into(), manifest.into_bytes());
        Ok(result)
    }

    pub fn check(&self, strict: bool) -> Result<()> {
        ensure!(
            self.pending()?.is_none(),
            "finish the current operation first"
        );
        let groups = self.groups(&self.base)?;
        let expected = self.formatted_groups(&groups)?;
        if strict {
            let actual: BTreeSet<_> = fs::read_dir(&self.patch_dir)?
                .collect::<std::io::Result<Vec<_>>>()?
                .into_iter()
                .filter_map(|entry| {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    (is_patch_file(&name) || name == ".patches").then_some(name)
                })
                .collect();
            ensure!(
                actual == expected.keys().cloned().collect(),
                "patch file set differs; export the stack"
            );
            for (name, data) in &expected {
                ensure!(
                    fs::read(self.patch_dir.join(name))? == *data,
                    "{name} is not canonical; run `cargo spo export {}`",
                    self.name
                );
            }
        } else {
            let names: Vec<_> = groups.iter().map(|g| g.name.clone()).collect();
            ensure!(
                names == self.filenames()?,
                "patch groups or order differ; export the stack"
            );
            for (name, data) in &expected {
                if name == ".patches" {
                    continue;
                }
                ensure!(
                    patch_ids(&self.repo, data)?
                        == patch_ids(&self.repo, &fs::read(self.patch_dir.join(name))?)?,
                    "patch content differs: {name}"
                );
            }
        }
        self.verify_apply(
            &self.patch_dir,
            Some(&git(&self.repo, &["rev-parse", "HEAD"])?),
        )?;
        println!(
            "{}: verified {} patch files ({} commits)",
            self.name,
            groups.len(),
            groups.iter().map(|g| g.commits.len()).sum::<usize>()
        );
        Ok(())
    }

    pub fn export(&self, verify: bool, stash: bool) -> Result<()> {
        let _lock = self.lock()?;
        let groups = self.groups(&self.base)?;
        let formatted = self.formatted_groups(&groups)?;
        // Export only reads HEAD, so WIP need not be stashed unless explicitly requested.
        ensure!(
            self.pending()?.is_none(),
            "finish the current operation first"
        );
        let mut operation = if stash {
            self.begin("export", groups, true)?
        } else {
            let op = Operation {
                kind: "export".into(),
                original: git(&self.repo, &["rev-parse", "HEAD"])?,
                base: self.base.clone(),
                phase: "prepared".into(),
                cursor: 0,
                groups,
                tips: vec![],
                stash: None,
                staging: None,
                backup: None,
                files: BTreeMap::new(),
            };
            self.save_operation(&op)?;
            op
        };
        let parent = self.patch_dir.parent().context("patch directory parent")?;
        let token = unique();
        let staging = parent.join(format!(".spo-stage-{token}"));
        let backup = parent.join(format!(".spo-backup-{token}"));
        operation.staging = Some(staging.clone());
        operation.backup = Some(backup);
        operation.phase = "staging".into();
        self.save_operation(&operation)?;
        fs::create_dir(&staging)?;
        for entry in fs::read_dir(&self.patch_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if name != ".patches" && !is_patch_file(&name) {
                ensure!(
                    entry.file_type()?.is_file(),
                    "unmanaged non-file in patch directory: {name}"
                );
                fs::copy(entry.path(), staging.join(name))?;
            }
        }
        for (name, data) in formatted {
            fs::write(staging.join(name), data)?;
        }
        if verify {
            self.verify_apply(&staging, Some(&operation.original))?;
        }
        operation.files = self.directory_hashes(&staging)?;
        operation.phase = "publishing".into();
        self.save_operation(&operation)?;
        self.publish(&mut operation)
    }

    fn directory_hashes(&self, directory: &Path) -> Result<BTreeMap<String, String>> {
        let mut hashes = BTreeMap::new();
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            ensure!(
                entry.file_type()?.is_file(),
                "unexpected export directory entry"
            );
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| anyhow::anyhow!("invalid export filename"))?;
            let hash = git(
                &self.repo,
                &["hash-object", entry.path().to_str().context("export path")?],
            )?;
            hashes.insert(name, hash);
        }
        Ok(hashes)
    }

    fn publish(&self, operation: &mut Operation) -> Result<()> {
        let staging = operation
            .staging
            .as_ref()
            .context("missing staging directory")?;
        let backup = operation
            .backup
            .as_ref()
            .context("missing backup directory")?;
        if staging.exists() {
            ensure!(
                self.directory_hashes(staging)? == operation.files,
                "staged export changed; preserve the edits before aborting"
            );
            if !backup.exists() {
                fs::rename(&self.patch_dir, backup)?;
            }
            ensure!(
                !self.patch_dir.exists(),
                "patch directory changed during publication"
            );
            fs::rename(staging, &self.patch_dir)?;
        }
        ensure!(self.patch_dir.exists(), "publication is incomplete");
        ensure!(
            self.directory_hashes(&self.patch_dir)? == operation.files,
            "published export changed; preserve the edits before continuing"
        );
        self.save_snapshot(&operation.base, &operation.groups)?;
        operation.phase = "published".into();
        self.save_operation(operation)?;
        println!(
            "{}: exported {} patch files ({} commits)",
            self.name,
            operation.groups.len(),
            operation
                .groups
                .iter()
                .map(|g| g.commits.len())
                .sum::<usize>()
        );
        self.finish(operation)
    }

    pub fn plan(&self) -> Result<String> {
        let mut result = String::new();
        for group in self.groups(&self.base)? {
            for commit in group.commits {
                writeln!(
                    result,
                    "pick {commit} {} # {}",
                    group.name,
                    git(&self.repo, &["show", "-s", "--format=%s", &commit])?
                )?;
            }
        }
        result.push_str("\n# Reorder lines; actions: pick, reword, edit, squash, fixup, drop.\n# Each commit must appear once. Use drop explicitly; group names must be contiguous.\n");
        Ok(result)
    }

    pub fn parse_plan(&self, text: &str) -> Result<Vec<PlanEntry>> {
        let commits = self.commits(&self.base)?;
        let mut seen = BTreeSet::new();
        let mut groups = BTreeSet::new();
        let mut previous: Option<String> = None;
        let mut result = Vec::new();
        for line in text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            let mut fields = line.split_whitespace();
            let action = fields.next().context("plan action")?;
            ensure!(
                ["pick", "reword", "edit", "squash", "fixup", "drop"].contains(&action),
                "unsupported plan action: {action}"
            );
            let commit = git(
                &self.repo,
                &[
                    "rev-parse",
                    "--verify",
                    fields.next().context("plan commit")?,
                ],
            )?;
            let group = fields.next().context("plan group")?;
            valid_name(group)?;
            ensure!(
                commits.contains(&commit) && seen.insert(commit.clone()),
                "unknown or duplicate plan commit: {commit}"
            );
            if action != "drop" {
                if previous.as_deref() != Some(group) {
                    ensure!(
                        action != "fixup" && action != "squash",
                        "fixup/squash must follow a retained commit in the same group"
                    );
                    ensure!(
                        groups.insert(group.to_owned()),
                        "non-contiguous plan group: {group}"
                    );
                }
                previous = Some(group.into());
            }
            result.push(PlanEntry {
                action: action.into(),
                commit,
                group: group.into(),
            });
        }
        ensure!(
            seen.len() == commits.len(),
            "plan omits commits; use drop explicitly"
        );
        Ok(result)
    }

    pub fn edit(
        &self,
        plan: &str,
        executable: &Path,
        stash: bool,
        non_interactive: bool,
    ) -> Result<()> {
        let _lock = self.lock()?;
        let entries = self.parse_plan(plan)?;
        ensure!(
            !non_interactive
                || entries
                    .iter()
                    .all(|e| e.action != "reword" && e.action != "squash"),
            "reword/squash need an editor; use an interactive session or pre-edit commit messages"
        );
        let mut operation = self.begin("edit", self.groups(&self.base)?, stash)?;
        let path = self.state_dir()?.join("rebase.todo");
        let mut todo = String::new();
        for (i, entry) in entries.iter().enumerate() {
            writeln!(todo, "{} {}", entry.action, entry.commit)?;
            if entry.action != "drop"
                && entries[i + 1..]
                    .iter()
                    .find(|e| e.action != "drop")
                    .is_none_or(|next| next.group != entry.group)
            {
                writeln!(
                    todo,
                    "exec {} checkpoint --repo {} --name {}",
                    shell_quote(executable.to_str().context("executable path")?),
                    shell_quote(self.repo.to_str().context("repo path")?),
                    shell_quote(&entry.group)
                )?;
            }
        }
        fs::write(&path, todo)?;
        operation.phase = "rebasing".into();
        self.save_operation(&operation)?;
        let editor = format!(
            "{} sequence-editor {}",
            shell_quote(executable.to_str().context("executable path")?),
            shell_quote(path.to_str().context("todo path")?)
        );
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(&self.repo)
            .args([
                "-c",
                "commit.gpgsign=false",
                "rebase",
                "--interactive",
                "--force-rebase",
                "--empty=keep",
                &self.base,
            ])
            .env("GIT_SEQUENCE_EDITOR", editor);
        if non_interactive {
            command.env("GIT_EDITOR", "true");
        }
        ensure!(
            command.status()?.success(),
            "rebase paused; resolve or edit, then run `cargo spo continue {}`",
            self.name
        );
        operation = self.operation()?.context("missing edit journal")?;
        self.complete_edit(&mut operation)
    }

    fn complete_edit(&self, operation: &mut Operation) -> Result<()> {
        ensure!(
            self.native_operation()?.is_none(),
            "Git rebase is still active"
        );
        let mut previous = operation.base.clone();
        let mut groups = Vec::new();
        for (name, tip) in &operation.tips {
            let commits: Vec<_> = git(
                &self.repo,
                &["rev-list", "--reverse", &format!("{previous}..{tip}")],
            )?
            .lines()
            .map(str::to_owned)
            .collect();
            ensure!(!commits.is_empty(), "empty edited group {name}");
            groups.push(Group {
                name: name.clone(),
                commits,
            });
            previous.clone_from(tip);
        }
        ensure!(
            previous == git(&self.repo, &["rev-parse", "HEAD"])?,
            "history changed after the last checkpoint; inspect before proceeding"
        );
        self.save_snapshot(&operation.base, &groups)?;
        operation.phase = "edited".into();
        self.save_operation(operation)?;
        self.finish(operation)
    }

    pub fn resume(&self, wip_restored: bool) -> Result<()> {
        let _lock = self.lock()?;
        let mut operation = self
            .operation()?
            .context("no spo operation to continue; inspect the native Git operation")?;
        if wip_restored {
            ensure!(
                operation.phase == "restoring-wip",
                "WIP restoration is not active"
            );
            ensure!(
                git(&self.repo, &["ls-files", "--unmerged"])?.is_empty(),
                "unresolved WIP conflicts remain"
            );
            operation.phase = "restored".into();
            self.save_operation(&operation)?;
            return self.finish(&mut operation);
        }
        ensure!(
            operation.phase != "saving-wip",
            "WIP save was interrupted; inspect refs/stash and abort or recover explicitly before continuing"
        );
        if operation.phase == "restoring-wip" || operation.phase == "restored" {
            return self.finish(&mut operation);
        }
        match operation.kind.as_str() {
            "apply" => self.resume_apply(&mut operation),
            "export" => {
                ensure!(
                    operation.phase == "publishing" || operation.phase == "published",
                    "export staging was interrupted; abort and export again"
                );
                self.publish(&mut operation)
            }
            "edit" => {
                if self.native_operation()?.as_deref() == Some("rebase") {
                    run_git(&self.repo, &["rebase", "--continue"])?;
                }
                // Checkpoints update the journal in child processes.
                operation = self.operation()?.context("missing edit journal")?;
                self.complete_edit(&mut operation)
            }
            _ => bail!("unknown operation {}", operation.kind),
        }
    }

    pub fn abort(&self) -> Result<()> {
        let _lock = self.lock()?;
        let mut operation = self.operation()?.context("no spo operation to abort")?;
        ensure!(
            operation.phase != "restoring-wip",
            "WIP restoration has changes; preserve or resolve them and use continue --wip-restored"
        );
        if let Some(native) = self.native_operation()? {
            ensure!(
                native == "am" || native == "rebase",
                "another Git operation is active"
            );
            run_git(&self.repo, &[&native, "--abort"])?;
        }
        if operation.kind != "export" {
            ensure!(
                !self.dirty()?,
                "new working changes must be preserved before aborting"
            );
            let head = git(&self.repo, &["rev-parse", "HEAD"])?;
            ensure!(
                head == operation.original || operation.tips.iter().any(|(_, sha)| sha == &head),
                "HEAD changed outside spo; original stack is preserved at {}",
                operation.original
            );
            git(
                &self.repo,
                &[
                    "update-ref",
                    &format!("refs/spo/aborted/{}", unique()),
                    &head,
                ],
            )?;
            run_git(&self.repo, &["reset", "--hard", &operation.original])?;
        }
        if let Some(backup) = &operation.backup
            && backup.exists()
        {
            if self.patch_dir.exists() {
                ensure!(
                    self.directory_hashes(&self.patch_dir)? == operation.files,
                    "export files changed after publication; preserve them before aborting"
                );
                fs::remove_dir_all(&self.patch_dir)?;
            }
            fs::rename(backup, &self.patch_dir)?;
        }
        operation.phase = "aborted".into();
        self.save_operation(&operation)?;
        self.finish(&mut operation)
    }
}

/// Called only by generated rebase exec lines. Native Git serializes checkpoints.
pub fn checkpoint(repo: &Path, name: &str) -> Result<()> {
    let target = Target {
        name: String::new(),
        repo: repo.into(),
        patch_dir: PathBuf::new(),
        base: String::new(),
        exclude: vec![],
    };
    let mut operation = target.operation()?.context("no edit journal")?;
    ensure!(
        operation.kind == "edit" && operation.phase == "rebasing",
        "checkpoint outside an edit operation"
    );
    valid_name(name)?;
    operation
        .tips
        .push((name.into(), git(repo, &["rev-parse", "HEAD"])?));
    target.save_operation(&operation)
}

#[must_use]
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
