use spo::{Target, git, mailbox_messages};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

struct Fixture {
    root: PathBuf,
    target: Target,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("spo-test-{}", spo::unique()));
        let repo = root.join("repo");
        let patches = root.join("patches");
        fs::create_dir_all(&repo).unwrap();
        fs::create_dir(&patches).unwrap();
        fs::write(patches.join(".patches"), "").unwrap();
        git(&repo, &["init", "--quiet", "-b", "main"]).unwrap();
        git(&repo, &["config", "user.name", "Test"]).unwrap();
        git(&repo, &["config", "user.email", "test@localhost"]).unwrap();
        git(&repo, &["config", "commit.gpgsign", "false"]).unwrap();
        fs::write(repo.join("file.txt"), "base\n").unwrap();
        fs::write(repo.join("other.txt"), "base\n").unwrap();
        git(&repo, &["add", "."]).unwrap();
        git(&repo, &["commit", "--quiet", "-m", "base"]).unwrap();
        let base = git(&repo, &["rev-parse", "HEAD"]).unwrap();
        fs::write(root.join("config.json"), serde_json::to_vec(&serde_json::json!([{ "name": "test", "repo": "repo", "patch_dir": "patches", "base": base }])).unwrap()).unwrap();
        Self {
            root,
            target: Target {
                name: "test".into(),
                repo,
                patch_dir: patches,
                base,
                exclude: vec![],
            },
        }
    }
    fn commit(&self, path: &str, contents: &str, message: &str) -> String {
        fs::write(self.target.repo.join(path), contents).unwrap();
        git(&self.target.repo, &["add", "--", path]).unwrap();
        git(&self.target.repo, &["commit", "--quiet", "-m", message]).unwrap();
        git(&self.target.repo, &["rev-parse", "HEAD"]).unwrap()
    }
    fn cli(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_spo"))
            .args([
                "--workspace",
                self.root.to_str().unwrap(),
                "--config",
                self.root.join("config.json").to_str().unwrap(),
                "--non-interactive",
            ])
            .args(args)
            .output()
            .unwrap()
    }
    fn success(&self, args: &[&str]) -> Output {
        let output = self.cli(args);
        assert!(
            output.status.success(),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn grouped_mailboxes_round_trip_and_keep_filenames() {
    let f = Fixture::new();
    let a = f.commit("file.txt", "one\n", "first");
    let b = f.commit("other.txt", "two\n", "second");
    f.target.group("feature.patch", &[a, b]).unwrap();
    f.target.export(true, false).unwrap();
    assert_eq!(
        mailbox_messages(&fs::read(f.target.patch_dir.join("feature.patch")).unwrap())
            .unwrap()
            .len(),
        2
    );
    let tree = git(&f.target.repo, &["rev-parse", "HEAD^{tree}"]).unwrap();
    git(&f.target.repo, &["reset", "--hard", &f.target.base]).unwrap();
    f.target.apply(false).unwrap();
    assert_eq!(
        git(&f.target.repo, &["rev-parse", "HEAD^{tree}"]).unwrap(),
        tree
    );
    f.target.check(true).unwrap();
    f.target.apply(false).unwrap();
    fs::write(
        f.target.patch_dir.join(".patches"),
        "# stack\n  feature.patch  \n",
    )
    .unwrap();
    f.target.check(false).unwrap();
    f.target.apply(false).unwrap();
    assert!(f.target.check(true).is_err());
}

#[test]
fn export_preserves_uncommitted_work_without_committing_it() {
    let f = Fixture::new();
    let commit = f.commit("file.txt", "committed\n", "change");
    fs::write(f.target.repo.join("file.txt"), "WIP\n").unwrap();
    fs::write(f.target.repo.join("untracked.txt"), "new\n").unwrap();
    f.target.export(true, false).unwrap();
    assert_eq!(git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(), commit);
    assert_eq!(
        fs::read_to_string(f.target.repo.join("file.txt")).unwrap(),
        "WIP\n"
    );
    assert!(f.target.repo.join("untracked.txt").exists());
}

#[test]
fn edit_reorders_groups_and_restores_staged_and_untracked_wip() {
    let f = Fixture::new();
    let a = f.commit("file.txt", "one\n", "first");
    let b = f.commit("other.txt", "two\n", "second");
    f.target.export(true, false).unwrap();
    fs::write(f.target.repo.join("wip.txt"), "staged\n").unwrap();
    git(&f.target.repo, &["add", "wip.txt"]).unwrap();
    fs::write(f.target.repo.join("untracked.txt"), "untracked\n").unwrap();
    let plan = f.root.join("plan");
    fs::write(
        &plan,
        format!("pick {b} second.patch\npick {a} first.patch\n"),
    )
    .unwrap();
    f.success(&[
        "edit",
        "test",
        "--plan",
        plan.to_str().unwrap(),
        "--dirty",
        "stash",
    ]);
    let groups = f.target.groups(&f.target.base).unwrap();
    assert_eq!(
        groups.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(),
        ["second.patch", "first.patch"]
    );
    assert_eq!(
        git(&f.target.repo, &["diff", "--cached", "--name-only"]).unwrap(),
        "wip.txt"
    );
    assert!(f.target.repo.join("untracked.txt").exists());
    assert!(f.target.pending().unwrap().is_none());
}

#[test]
fn edit_pause_can_continue_after_splitting_a_commit() {
    let f = Fixture::new();
    fs::write(f.target.repo.join("file.txt"), "one\n").unwrap();
    fs::write(f.target.repo.join("other.txt"), "two\n").unwrap();
    git(&f.target.repo, &["add", "."]).unwrap();
    git(&f.target.repo, &["commit", "--quiet", "-m", "feature"]).unwrap();
    let commit = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    let plan = f.root.join("plan");
    fs::write(&plan, format!("edit {commit} feature.patch\n")).unwrap();
    assert!(
        !f.cli(&["edit", "test", "--plan", plan.to_str().unwrap()])
            .status
            .success()
    );
    assert!(f.target.pending().unwrap().is_some());
    git(&f.target.repo, &["reset", "HEAD^"]).unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    git(&f.target.repo, &["commit", "--quiet", "-m", "first half"]).unwrap();
    git(&f.target.repo, &["add", "other.txt"]).unwrap();
    git(&f.target.repo, &["commit", "--quiet", "-m", "second half"]).unwrap();
    f.success(&["continue", "test"]);
    let groups = f.target.groups(&f.target.base).unwrap();
    assert_eq!(groups[0].name, "feature.patch");
    assert_eq!(groups[0].commits.len(), 2);
    f.target.export(true, false).unwrap();
    f.target.check(true).unwrap();
}

#[test]
fn invalid_plan_never_modifies_history() {
    let f = Fixture::new();
    let a = f.commit("file.txt", "one\n", "first");
    let b = f.commit("other.txt", "two\n", "second");
    assert!(
        f.target
            .parse_plan(&format!("pick {a} first.patch\n"))
            .is_err()
    );
    assert!(
        f.target
            .parse_plan(&format!("pick {a} first.patch\npick {a} first.patch\n"))
            .is_err()
    );
    assert!(
        f.target
            .parse_plan(&format!("exec {a} first.patch\npick {b} second.patch\n"))
            .is_err()
    );
    assert!(
        f.target
            .parse_plan(&format!("fixup {a} first.patch\npick {b} second.patch\n"))
            .is_err()
    );
    assert_eq!(git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(), b);
    assert!(f.target.pending().unwrap().is_none());
}

fn conflicting_target(f: &Fixture) -> Target {
    f.commit("file.txt", "patch content\n", "feature");
    f.target.export(true, false).unwrap();
    git(&f.target.repo, &["checkout", "--detach", &f.target.base]).unwrap();
    let base = f.commit("file.txt", "upstream content\n", "upstream");
    Target {
        base,
        ..f.target.clone()
    }
}

#[test]
fn apply_conflict_continues_with_original_group_name() {
    let f = Fixture::new();
    let t = conflicting_target(&f);
    assert!(t.apply(false).is_err());
    fs::write(t.repo.join("file.txt"), "resolved\n").unwrap();
    git(&t.repo, &["add", "file.txt"]).unwrap();
    t.resume(false).unwrap();
    assert_eq!(t.groups(&t.base).unwrap()[0].name, "feature.patch");
    assert!(t.pending().unwrap().is_none());
    t.export(true, false).unwrap();
    t.check(true).unwrap();
}

#[test]
fn apply_conflict_abort_restores_original_stack_and_wip() {
    let f = Fixture::new();
    let t = conflicting_target(&f);
    fs::write(t.repo.join("wip.txt"), "keep\n").unwrap();
    assert!(t.apply(true).is_err());
    t.abort().unwrap();
    assert_eq!(git(&t.repo, &["rev-parse", "HEAD"]).unwrap(), t.base);
    assert_eq!(
        fs::read_to_string(t.repo.join("wip.txt")).unwrap(),
        "keep\n"
    );
    assert!(t.pending().unwrap().is_none());
}

#[test]
fn semantic_check_tolerates_metadata_but_strict_check_detects_it() {
    let f = Fixture::new();
    f.commit("file.txt", "one\n", "first");
    f.target.export(true, false).unwrap();
    let path = f.target.patch_dir.join("first.patch");
    fs::write(
        &path,
        fs::read_to_string(&path)
            .unwrap()
            .replace("From: Test", "From: Another Author"),
    )
    .unwrap();
    f.target.check(false).unwrap();
    assert!(f.target.check(true).is_err());
}

#[test]
fn new_only_commits_selected_files_and_refresh_keeps_other_wip() {
    let f = Fixture::new();
    fs::write(f.target.repo.join("file.txt"), "one\n").unwrap();
    fs::write(f.target.repo.join("other.txt"), "other WIP\n").unwrap();
    git(&f.target.repo, &["add", "other.txt"]).unwrap();
    f.success(&[
        "new",
        "test",
        "--message",
        "feature",
        "--paths",
        "file.txt",
        "--export",
    ]);
    assert_eq!(
        git(&f.target.repo, &["diff", "--cached", "--name-only"]).unwrap(),
        "other.txt"
    );
    fs::write(f.target.repo.join("file.txt"), "two\n").unwrap();
    f.success(&[
        "refresh",
        "test",
        "--patch",
        "feature.patch",
        "--paths",
        "file.txt",
        "--export",
        "--dirty",
        "stash",
    ]);
    assert_eq!(f.target.commits(&f.target.base).unwrap().len(), 1);
    assert_eq!(
        git(&f.target.repo, &["diff", "--cached", "--name-only"]).unwrap(),
        "other.txt"
    );
    assert_eq!(
        fs::read_to_string(f.target.repo.join("file.txt")).unwrap(),
        "two\n"
    );
    f.target.check(true).unwrap();
}

#[test]
fn check_rejects_duplicate_manifest_and_ownership_violations() {
    let f = Fixture::new();
    f.commit("file.txt", "one\n", "first");
    f.target.export(true, false).unwrap();
    fs::write(
        f.target.patch_dir.join(".patches"),
        "first.patch\nfirst.patch\n",
    )
    .unwrap();
    assert!(f.target.check(false).is_err());
    let t = Target {
        exclude: vec!["file.txt".into()],
        ..f.target.clone()
    };
    assert!(
        t.format_commit(&git(&t.repo, &["rev-parse", "HEAD"]).unwrap())
            .is_err()
    );
}

#[test]
fn dry_run_and_noninteractive_menu_do_not_modify_repository() {
    let f = Fixture::new();
    f.commit("file.txt", "one\n", "first");
    let head = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    f.success(&["export", "test", "--dry-run"]);
    assert_eq!(
        fs::read_to_string(f.target.patch_dir.join(".patches")).unwrap(),
        ""
    );
    assert!(!f.cli(&[]).status.success());
    assert_eq!(git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(), head);
}

#[test]
fn multiple_targets_export_and_check_independently() {
    let a = Fixture::new();
    let b = Fixture::new();
    a.commit("file.txt", "first\n", "first");
    b.commit("other.txt", "second\n", "second");
    let root = a.root.join("second");
    fs::rename(&b.root, &root).unwrap();
    fs::write(a.root.join("config.json"), serde_json::to_vec(&serde_json::json!([
        {"name":"first", "repo":"repo", "patch_dir":"patches", "base":a.target.base},
        {"name":"second", "repo":"second/repo", "patch_dir":"second/patches", "base":b.target.base}
    ])).unwrap()).unwrap();
    a.success(&["export", "--all", "--verify-apply"]);
    a.success(&["check", "first", "second", "--strict"]);
}

#[test]
fn interrupted_export_refuses_to_discard_later_patch_edits() {
    let f = Fixture::new();
    f.commit("file.txt", "changed\n", "feature");
    f.target.export(false, false).unwrap();
    let groups = f.target.groups(&f.target.base).unwrap();
    let backup = f.root.join("backup");
    fs::create_dir(&backup).unwrap();
    fs::write(backup.join(".patches"), "").unwrap();
    let mut hashes = std::collections::BTreeMap::new();
    for entry in fs::read_dir(&f.target.patch_dir).unwrap() {
        let entry = entry.unwrap();
        hashes.insert(
            entry.file_name().to_str().unwrap().to_owned(),
            git(
                &f.target.repo,
                &["hash-object", entry.path().to_str().unwrap()],
            )
            .unwrap(),
        );
    }
    let journal = f.target.state_dir().unwrap().join("operation.json");
    fs::write(&journal, serde_json::to_vec(&serde_json::json!({
        "kind": "export", "original": git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(),
        "base": f.target.base, "phase": "published", "cursor": 0, "groups": groups,
        "tips": [], "stash": null, "staging": f.root.join("stage"), "backup": backup, "files": hashes,
    })).unwrap()).unwrap();
    let patch = f.target.patch_dir.join(&groups[0].name);
    let original = fs::read(&patch).unwrap();
    fs::write(&patch, "later user edit\n").unwrap();
    assert!(f.target.resume(false).is_err());
    assert!(f.target.abort().is_err());
    assert_eq!(fs::read_to_string(&patch).unwrap(), "later user edit\n");
    assert!(backup.exists());
    fs::write(&patch, original).unwrap();
    f.target.abort().unwrap();
    assert_eq!(
        fs::read_to_string(f.target.patch_dir.join(".patches")).unwrap(),
        ""
    );
    assert!(!journal.exists());
}

#[test]
fn excluded_nested_checkout_is_not_wip_and_survives_stash() {
    let mut f = Fixture::new();
    f.target.exclude = vec!["nested repo".into()];
    let nested = f.target.repo.join("nested repo");
    fs::create_dir_all(&nested).unwrap();
    git(&nested, &["init", "--quiet"]).unwrap();
    fs::write(nested.join("local.txt"), "nested WIP\n").unwrap();
    assert!(!f.target.dirty().unwrap());
    assert!(f.target.stash().unwrap().is_none());
    fs::write(f.target.repo.join(".git/info/exclude"), "/nested repo/\n").unwrap();
    fs::write(f.target.repo.join("file.txt"), "tracked WIP\n").unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    fs::write(f.target.repo.join("outside.txt"), "untracked WIP\n").unwrap();
    assert!(f.target.dirty().unwrap());
    let oid = f.target.stash().unwrap().unwrap();
    assert!(!f.target.dirty().unwrap());
    assert_eq!(
        fs::read_to_string(nested.join("local.txt")).unwrap(),
        "nested WIP\n"
    );
    let files = git(
        &f.target.repo,
        &["stash", "show", "--include-untracked", "--name-only", &oid],
    )
    .unwrap();
    assert!(!files.contains("nested repo"));
    assert!(!f.target.repo.join("outside.txt").exists());
    f.target.restore_stash(&oid).unwrap();
    f.target.drop_stash(&oid).unwrap();
    assert_eq!(
        fs::read_to_string(f.target.repo.join("outside.txt")).unwrap(),
        "untracked WIP\n"
    );
    assert!(
        git(&f.target.repo, &["diff", "--cached", "--name-only"])
            .unwrap()
            .contains("file.txt")
    );
}

#[test]
fn literal_exclusions_do_not_hide_similarly_named_files_or_all_wip() {
    let mut f = Fixture::new();
    f.target.exclude = vec!["other.txt".into(), "brackets[1]".into()];
    fs::write(f.target.repo.join("other.txt"), "excluded WIP\n").unwrap();
    fs::create_dir(f.target.repo.join("brackets[1]")).unwrap();
    fs::write(f.target.repo.join("brackets[1]/local.txt"), "excluded\n").unwrap();
    assert!(!f.target.dirty().unwrap());
    fs::write(f.target.repo.join("brackets1"), "owned WIP\n").unwrap();
    assert!(f.target.dirty().unwrap());
    let oid = f.target.stash().unwrap().unwrap();
    assert!(!f.target.dirty().unwrap());
    assert_eq!(
        fs::read_to_string(f.target.repo.join("other.txt")).unwrap(),
        "excluded WIP\n"
    );
    assert!(f.target.repo.join("brackets[1]/local.txt").exists());
    f.target.restore_stash(&oid).unwrap();
    f.target.drop_stash(&oid).unwrap();
    assert!(f.target.repo.join("brackets1").exists());
}

#[test]
fn missing_ref_recovers_from_multi_commit_mailbox_without_snapshot() {
    let f = Fixture::new();
    let first = f.commit("file.txt", "one\n", "first");
    let second = f.commit("other.txt", "two\n", "second");
    f.target.group("group.patch", &[first, second]).unwrap();
    f.target.export(true, false).unwrap();
    fs::remove_file(f.target.state_dir().unwrap().join("stack.json")).unwrap();
    fs::write(f.root.join("config.json"), serde_json::to_vec(&serde_json::json!([{ "name":"test", "repo":"repo", "patch_dir":"patches", "base_ref":spo::UPSTREAM_REF }])).unwrap()).unwrap();
    f.success(&["--dry-run", "status", "test"]);
    assert!(
        git(
            &f.target.repo,
            &["rev-parse", "--verify", spo::UPSTREAM_REF]
        )
        .is_err()
    );
    f.success(&["status", "test"]);
    assert_eq!(
        git(&f.target.repo, &["rev-parse", spo::UPSTREAM_REF]).unwrap(),
        f.target.base
    );
    assert_ne!(
        git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(),
        f.target.base
    );
    f.target.check(true).unwrap();
}

#[test]
fn missing_ref_does_not_guess_with_unexported_commits_and_accepts_explicit_base() {
    let f = Fixture::new();
    f.commit("file.txt", "patch\n", "feature");
    f.target.export(true, false).unwrap();
    fs::remove_file(f.target.state_dir().unwrap().join("stack.json")).unwrap();
    f.commit("other.txt", "local commit\n", "not exported");
    fs::write(f.root.join("config.json"), serde_json::to_vec(&serde_json::json!([{ "name":"test", "repo":"repo", "patch_dir":"patches", "base_ref":spo::UPSTREAM_REF }])).unwrap()).unwrap();
    let output = f.cli(&["status", "test"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--base test="));
    assert!(
        git(
            &f.target.repo,
            &["rev-parse", "--verify", spo::UPSTREAM_REF]
        )
        .is_err()
    );
    let explicit = format!("test={}", f.target.base);
    f.success(&["--base", &explicit, "status", "test"]);
    assert_eq!(
        git(&f.target.repo, &["rev-parse", spo::UPSTREAM_REF]).unwrap(),
        f.target.base
    );
}

#[test]
fn stash_preserves_index_when_worktree_reverts_staged_changes() {
    let mut f = Fixture::new();
    f.target.exclude = vec!["nested".into()];
    let original = fs::read(f.target.repo.join("file.txt")).unwrap();
    fs::write(f.target.repo.join("file.txt"), "staged WIP\n").unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    fs::write(f.target.repo.join("file.txt"), &original).unwrap();
    fs::write(f.target.repo.join("line\nbreak.txt"), "untracked\n").unwrap();
    let oid = f.target.stash().unwrap().unwrap();
    assert!(!f.target.dirty().unwrap());
    f.target.restore_stash(&oid).unwrap();
    f.target.drop_stash(&oid).unwrap();
    assert_eq!(fs::read(f.target.repo.join("file.txt")).unwrap(), original);
    assert_eq!(
        git(&f.target.repo, &["show", ":file.txt"]).unwrap(),
        "staged WIP"
    );
    assert!(f.target.repo.join("line\nbreak.txt").exists());
}

#[test]
fn stale_checkout_path_is_reported_by_cli_and_bridge_without_mutation() {
    let f = Fixture::new();
    f.commit("file.txt", "patch\n", "feature");
    f.target.export(true, false).unwrap();
    let head = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    let config = f.root.join("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!([{
            "name": "test", "repo": "repo", "patch_dir": "renamed-framework/patches",
            "base": f.target.base,
        }]))
        .unwrap(),
    )
    .unwrap();
    for action in ["status", "check"] {
        let output = f.cli(&[action, "test"]);
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains("configured patch directory does not exist"),
            "{error}"
        );
        assert!(error.contains("renamed-framework"));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("Patch files: 0"));
    }
    let request = f.root.join("request.json");
    let response = f.root.join("response.json");
    fs::write(
        &request,
        serde_json::to_vec(&serde_json::json!({
            "operation": "load", "workspace": f.root, "config": config,
            "bases": {"test": f.target.base},
        }))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_spo"))
        .arg("bridge")
        .arg(&request)
        .arg(&response)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("configured patch directory does not exist")
    );
    assert!(!response.exists());
    assert_eq!(git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(), head);
    assert!(
        git(
            &f.target.repo,
            &["rev-parse", "--verify", spo::UPSTREAM_REF]
        )
        .is_err()
    );
}

#[test]
fn status_reports_missing_or_invalid_manifest_instead_of_an_empty_series() {
    let f = Fixture::new();
    let manifest = f.target.patch_dir.join(".patches");
    fs::remove_file(&manifest).unwrap();
    let error = format!("{:#}", f.target.status().unwrap_err());
    assert!(error.contains("read patch manifest"));
    assert!(error.contains(".patches"));
    fs::write(&manifest, "missing.patch\n").unwrap();
    let error = format!("{:#}", f.target.status().unwrap_err());
    assert!(error.contains("missing or unsafe patch: missing.patch"));
}

#[test]
fn cli_and_bridge_accept_patch_paths_with_spaces() {
    let mut f = Fixture::new();
    f.commit("file.txt", "patch\n", "feature");
    f.target.export(true, false).unwrap();
    let directory = f.root.join("patch directory with spaces");
    fs::rename(&f.target.patch_dir, &directory).unwrap();
    f.target.patch_dir = directory;
    let config = f.root.join("config.json");
    fs::write(
        &config,
        serde_json::to_vec(&serde_json::json!([{
            "name": "test", "repo": "repo", "patch_dir": "patch directory with spaces",
            "base": f.target.base,
        }]))
        .unwrap(),
    )
    .unwrap();
    f.success(&["check", "test", "--strict"]);
    let request = f.root.join("request with spaces.json");
    let response = f.root.join("response with spaces.json");
    fs::write(
        &request,
        serde_json::to_vec(&serde_json::json!({
            "operation": "load", "workspace": f.root, "config": config,
        }))
        .unwrap(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_spo"))
        .arg("bridge")
        .arg(&request)
        .arg(&response)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let targets: Vec<Target> = serde_json::from_slice(&fs::read(response).unwrap()).unwrap();
    assert_eq!(targets[0].patch_dir, f.target.patch_dir);
    assert_eq!(targets[0].status().unwrap().files, 1);
}
