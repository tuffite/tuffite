#![cfg(unix)]
use spo::{Target, git};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};

fn framework_command() -> Command {
    let mut command = Command::new("python3");
    command.arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../x.py"));
    command.env("TUFFITE_SPO", env!("CARGO_BIN_EXE_spo"));
    command
}

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    target: Target,
    new_base: String,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("xtask-test-{}", spo::unique()));
        let repo = root.join("src");
        let source = repo.join("tuffite");
        fs::create_dir_all(source.join("patches/chromium")).unwrap();
        fs::create_dir_all(source.join("scripts")).unwrap();
        fs::write(source.join("scripts/framework.py"), "# test driver\n").unwrap();
        fs::write(source.join("patches/chromium/.patches"), "").unwrap();
        git(&repo, &["init", "--quiet", "-b", "main"]).unwrap();
        git(&repo, &["config", "user.name", "Test"]).unwrap();
        git(&repo, &["config", "user.email", "test@localhost"]).unwrap();
        git(&repo, &["config", "commit.gpgsign", "false"]).unwrap();
        // A fresh gclient checkout does not ignore its nested Framework repository.
        fs::write(repo.join(".gitignore"), "").unwrap();
        fs::write(repo.join("file.txt"), "base\n").unwrap();
        git(&repo, &["add", ".gitignore", "file.txt"]).unwrap();
        git(&repo, &["commit", "--quiet", "-m", "base"]).unwrap();
        let base = git(&repo, &["rev-parse", "HEAD"]).unwrap();
        git(&repo, &["checkout", "--detach", &base]).unwrap();
        fs::write(repo.join("upstream.txt"), "upstream\n").unwrap();
        git(&repo, &["add", "upstream.txt"]).unwrap();
        git(&repo, &["commit", "--quiet", "-m", "upstream"]).unwrap();
        let new_base = git(&repo, &["rev-parse", "HEAD"]).unwrap();
        git(&repo, &["checkout", "main"]).unwrap();
        fs::write(repo.join("file.txt"), "patch\n").unwrap();
        git(&repo, &["add", "file.txt"]).unwrap();
        git(&repo, &["commit", "--quiet", "-m", "feature"]).unwrap();
        let target = Target {
            name: "chromium".into(),
            repo: repo.clone(),
            patch_dir: source.join("patches/chromium"),
            base: base.clone(),
            exclude: vec!["tuffite".into()],
        };
        target.export(true, false).unwrap();
        git(&repo, &["update-ref", spo::UPSTREAM_REF, &base]).unwrap();
        fs::write(root.join(".gclient"), "solutions = [{'name': 'src/tuffite', 'url': 'https://example.test/framework.git@framework-pin'}]\n").unwrap();
        fs::write(
            source.join("DEPS"),
            format!("vars = {{'chromium_revision': '{base}'}}\n"),
        )
        .unwrap();
        fs::write(source.join("patches/config.json"), serde_json::to_vec(&serde_json::json!([{ "name":"chromium", "repo":"src", "patch_dir":"src/tuffite/patches/chromium", "base_ref":spo::UPSTREAM_REF, "exclude":["tuffite"] }])).unwrap()).unwrap();
        git(&source, &["init", "--quiet", "-b", "framework-feature"]).unwrap();
        git(&source, &["config", "user.name", "Test"]).unwrap();
        git(&source, &["config", "user.email", "test@localhost"]).unwrap();
        git(&source, &["config", "commit.gpgsign", "false"]).unwrap();
        git(&source, &["add", "."]).unwrap();
        git(&source, &["commit", "--quiet", "-m", "framework"]).unwrap();
        let fake = root.join("gclient");
        fs::write(&fake, r"#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys
root = pathlib.Path.cwd()
assert os.environ['TUFFITE_SKIP_PATCHES'] == '1'
with (root / 'calls.jsonl').open('a') as output:
    output.write(json.dumps(sys.argv[1:]) + '\n')
if (root / 'fail-sync').exists() and sys.argv[1] == 'sync':
    sys.exit(7)
if sys.argv[1] == 'sync':
    assert '--nohooks' in sys.argv
    args = iter(sys.argv[2:])
    for arg in args:
        if arg == '--revision':
            repo, revision = next(args).rsplit('@', 1)
            if revision != 'unmanaged':
                subprocess.check_call(['git', '-C', str(root / repo), 'checkout', '--detach', revision])
").unwrap();
        fs::set_permissions(fake, fs::Permissions::from_mode(0o755)).unwrap();
        Self {
            root,
            source,
            target,
            new_base,
        }
    }
    fn cli(&self, args: &[&str]) -> Output {
        framework_command()
            .args([
                "--source",
                self.source.to_str().unwrap(),
                "--non-interactive",
            ])
            .args(args)
            .env("TUFFITE_GCLIENT", self.root.join("gclient"))
            .env("TUFFITE_SPO", env!("CARGO_BIN_EXE_spo"))
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
fn roll_changes_base_preserves_framework_branch_and_restores_wip() {
    let f = Fixture::new();
    let framework_head = git(&f.source, &["rev-parse", "HEAD"]).unwrap();
    fs::write(f.target.repo.join("wip.txt"), "keep\n").unwrap();
    f.success(&[
        "roll",
        "--chromium-revision",
        &f.new_base,
        "--dirty",
        "stash",
    ]);
    assert_eq!(
        git(&f.source, &["symbolic-ref", "--short", "HEAD"]).unwrap(),
        "framework-feature"
    );
    assert_eq!(
        git(&f.source, &["rev-parse", "HEAD"]).unwrap(),
        framework_head
    );
    assert_eq!(
        git(&f.target.repo, &["rev-parse", "HEAD^"]).unwrap(),
        f.new_base
    );
    assert_eq!(
        fs::read_to_string(f.target.repo.join("file.txt")).unwrap(),
        "patch\n"
    );
    assert_eq!(
        fs::read_to_string(f.target.repo.join("wip.txt")).unwrap(),
        "keep\n"
    );
    assert!(
        fs::read_to_string(f.source.join("DEPS"))
            .unwrap()
            .contains(&f.new_base)
    );
    assert!(
        fs::read_to_string(f.root.join(".gclient"))
            .unwrap()
            .contains("framework-pin")
    );
    let calls = fs::read_to_string(f.root.join("calls.jsonl")).unwrap();
    assert!(calls.contains("src/tuffite@unmanaged"));
    assert!(calls.contains("runhooks"));
    assert!(f.target.native_operation().unwrap().is_none());
}

#[test]
fn interrupted_sync_continues_without_repeating_preparation() {
    let f = Fixture::new();
    fs::write(f.root.join("fail-sync"), "").unwrap();
    let original = git(&f.target.repo, &["rev-parse", "main"]).unwrap();
    fs::write(f.target.repo.join("wip.txt"), "keep\n").unwrap();
    assert!(
        !f.cli(&[
            "roll",
            "--chromium-revision",
            &f.new_base,
            "--dirty",
            "stash"
        ])
        .status
        .success()
    );
    assert!(!f.target.repo.join("wip.txt").exists());
    fs::remove_file(f.root.join("fail-sync")).unwrap();
    f.success(&["roll", "--continue"]);
    assert_eq!(
        git(&f.target.repo, &["rev-parse", "main"]).unwrap(),
        original
    );
    assert_eq!(
        fs::read_to_string(f.target.repo.join("wip.txt")).unwrap(),
        "keep\n"
    );
    assert!(git(&f.target.repo, &["stash", "list"]).unwrap().is_empty());
}

#[test]
fn interrupted_roll_can_abort_and_restore_original_declarations() {
    let f = Fixture::new();
    fs::write(f.root.join("fail-sync"), "").unwrap();
    let original = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    let deps = fs::read_to_string(f.source.join("DEPS")).unwrap();
    let gclient = fs::read_to_string(f.root.join(".gclient")).unwrap();
    fs::write(f.target.repo.join("wip.txt"), "keep\n").unwrap();
    assert!(
        !f.cli(&[
            "roll",
            "--chromium-revision",
            &f.new_base,
            "--dirty",
            "stash"
        ])
        .status
        .success()
    );
    f.success(&["roll", "--abort"]);
    assert_eq!(
        git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(),
        original
    );
    assert_eq!(
        git(&f.target.repo, &["symbolic-ref", "--short", "HEAD"]).unwrap(),
        "main"
    );
    assert_eq!(fs::read_to_string(f.source.join("DEPS")).unwrap(), deps);
    assert_eq!(
        fs::read_to_string(f.root.join(".gclient")).unwrap(),
        gclient
    );
    assert_eq!(
        fs::read_to_string(f.target.repo.join("wip.txt")).unwrap(),
        "keep\n"
    );
}

#[test]
fn dry_run_does_not_launch_gclient_or_edit_declarations() {
    let f = Fixture::new();
    let original = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    let deps = fs::read_to_string(f.source.join("DEPS")).unwrap();
    let output = f.success(&["roll", "--chromium-revision", &f.new_base, "--dry-run"]);
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!("src@{}", f.new_base)));
    assert!(!f.root.join("calls.jsonl").exists());
    assert_eq!(
        git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(),
        original
    );
    assert_eq!(fs::read_to_string(f.source.join("DEPS")).unwrap(), deps);
}

#[test]
fn build_run_plan_uses_local_distribution_and_no_sync() {
    let f = Fixture::new();
    let result = f.success(&[
        "build",
        "--run",
        "--profile",
        "release",
        "--target-cpu",
        "x64",
        "--dry-run",
    ]);
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains("Package Framework:"));
    assert!(output.contains("tuffite-framework-") && output.contains("-x64"));
    assert!(output.contains("dev --manifest"));
    assert!(!output.contains("gclient"));
    assert!(!f.source.join("dist").exists());
}

#[test]
fn native_plan_resolves_parallel_jobs_and_explicit_override() {
    let f = Fixture::new();
    let args = [
        "build",
        "--framework",
        "--profile",
        "release",
        "--target-cpu",
        "x64",
        "--dry-run",
    ];
    let output = String::from_utf8(f.success(&args).stdout).unwrap();
    assert!(output.contains("Ninja -C"));
    assert!(output.contains("tuffite/framework:distribution"));
    let mut explicit = args.to_vec();
    explicit.extend(["-j", "3"]);
    let output = String::from_utf8(f.success(&explicit).stdout).unwrap();
    assert!(output.contains("-j 3 "));
    let mut invalid = args.to_vec();
    invalid.extend(["-j", "0"]);
    assert!(!f.cli(&invalid).status.success());
}

#[test]
fn dirty_rejection_and_noninteractive_missing_choices_do_not_mutate() {
    let f = Fixture::new();
    let original = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    fs::write(f.target.repo.join("wip.txt"), "keep\n").unwrap();
    assert!(!f.cli(&["roll"]).status.success());
    assert!(!f.cli(&["package", "--dry-run"]).status.success());
    assert!(f.cli(&[]).status.success());
    assert_eq!(
        git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(),
        original
    );
    assert!(!f.root.join("calls.jsonl").exists());
}

#[test]
fn unchanged_roll_is_a_noop_even_with_wip() {
    let f = Fixture::new();
    f.success(&["roll"]);
    let calls = fs::read_to_string(f.root.join("calls.jsonl")).unwrap();
    fs::write(f.target.repo.join("wip.txt"), "keep\n").unwrap();
    f.success(&["roll"]);
    assert_eq!(
        fs::read_to_string(f.root.join("calls.jsonl")).unwrap(),
        calls
    );
    assert_eq!(
        fs::read_to_string(f.target.repo.join("wip.txt")).unwrap(),
        "keep\n"
    );
}

#[test]
fn first_checkout_reconstructs_private_ref_and_grouped_mailboxes() {
    let f = Fixture::new();
    fs::write(f.target.repo.join("another.txt"), "second\n").unwrap();
    git(&f.target.repo, &["add", "another.txt"]).unwrap();
    git(&f.target.repo, &["commit", "--quiet", "-m", "second"]).unwrap();
    let commits = f.target.commits(&f.target.base).unwrap();
    f.target.group("group.patch", &commits).unwrap();
    f.target.export(true, false).unwrap();
    git(&f.target.repo, &["reset", "--hard", &f.target.base]).unwrap();
    git(&f.target.repo, &["update-ref", "-d", spo::UPSTREAM_REF]).unwrap();
    fs::remove_file(f.target.state_dir().unwrap().join("stack.json")).unwrap();
    for _ in 0..2 {
        f.success(&["bootstrap"]);
    }
    assert_eq!(
        git(&f.target.repo, &["rev-parse", spo::UPSTREAM_REF]).unwrap(),
        f.target.base
    );
    assert_eq!(f.target.commits(&f.target.base).unwrap().len(), 2);
    f.target.check(true).unwrap();
    assert!(!f.root.join("calls.jsonl").exists());
}

#[test]
fn first_checkout_conflict_recovers_without_a_base_ref() {
    let f = Fixture::new();
    git(&f.target.repo, &["reset", "--hard", &f.target.base]).unwrap();
    fs::write(f.target.repo.join("file.txt"), "upstream conflict\n").unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    git(
        &f.target.repo,
        &["commit", "--quiet", "-m", "conflicting upstream"],
    )
    .unwrap();
    let base = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    let deps = fs::read_to_string(f.source.join("DEPS"))
        .unwrap()
        .replace(&f.target.base, &base);
    fs::write(f.source.join("DEPS"), deps).unwrap();
    git(&f.target.repo, &["update-ref", "-d", spo::UPSTREAM_REF]).unwrap();
    assert!(!f.cli(&["bootstrap"]).status.success());
    let targets = spo::load_targets(&f.root, &f.source.join("patches/config.json")).unwrap();
    assert_eq!(targets[0].base, base);
    fs::write(f.target.repo.join("file.txt"), "resolved\n").unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    targets[0].resume(false).unwrap();
    assert_eq!(
        git(&f.target.repo, &["rev-parse", spo::UPSTREAM_REF]).unwrap(),
        base
    );
    assert!(targets[0].pending().unwrap().is_none());
}

#[test]
fn roll_can_defer_the_python_hook() {
    let f = Fixture::new();
    let before = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    let output = framework_command()
        .args([
            "--source",
            f.source.to_str().unwrap(),
            "bootstrap",
            "--non-interactive",
        ])
        .env("TUFFITE_SKIP_PATCHES", "1")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap(), before);
    assert!(String::from_utf8_lossy(&output.stdout).contains("deferred"));
}

#[test]
fn roll_patch_conflict_can_continue_without_repeating_sync() {
    let mut f = Fixture::new();
    let original = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    git(&f.target.repo, &["checkout", "--detach", &f.new_base]).unwrap();
    fs::write(f.target.repo.join("file.txt"), "upstream conflict\n").unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    git(
        &f.target.repo,
        &["commit", "--quiet", "-m", "conflicting upstream"],
    )
    .unwrap();
    f.new_base = git(&f.target.repo, &["rev-parse", "HEAD"]).unwrap();
    git(&f.target.repo, &["checkout", "main"]).unwrap();
    assert!(
        !f.cli(&["roll", "--chromium-revision", &f.new_base])
            .status
            .success()
    );
    assert!(f.target.native_operation().unwrap().is_some());
    let calls = fs::read_to_string(f.root.join("calls.jsonl")).unwrap();
    fs::write(f.target.repo.join("file.txt"), "resolved feature\n").unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    f.success(&["roll", "--continue"]);
    assert_eq!(
        fs::read_to_string(f.root.join("calls.jsonl")).unwrap(),
        calls
    );
    assert_eq!(
        git(&f.target.repo, &["rev-parse", "main"]).unwrap(),
        original
    );
    assert_eq!(
        fs::read_to_string(f.target.repo.join("file.txt")).unwrap(),
        "resolved feature\n"
    );
    assert!(f.target.pending().unwrap().is_none());
}

#[test]
fn roll_wip_conflict_retains_stash_until_explicit_resolution() {
    let f = Fixture::new();
    let filename = f.target.filenames().unwrap().remove(0);
    let mailbox = f.target.patch_dir.join(filename);
    let patch = fs::read_to_string(&mailbox).unwrap();
    fs::write(&mailbox, patch.replace("+patch\n", "+changed\n")).unwrap();
    fs::write(f.target.repo.join("file.txt"), "local WIP\n").unwrap();
    assert!(!f.cli(&["roll", "--dirty", "stash"]).status.success());
    assert!(
        !git(&f.target.repo, &["ls-files", "--unmerged"])
            .unwrap()
            .is_empty()
    );
    assert!(!git(&f.target.repo, &["stash", "list"]).unwrap().is_empty());
    assert!(!f.cli(&["roll", "--continue"]).status.success());
    assert!(!f.cli(&["roll", "--abort"]).status.success());
    fs::write(f.target.repo.join("file.txt"), "changed with local WIP\n").unwrap();
    git(&f.target.repo, &["add", "file.txt"]).unwrap();
    f.success(&["roll", "--continue", "--wip-restored"]);
    assert!(git(&f.target.repo, &["stash", "list"]).unwrap().is_empty());
    assert_eq!(
        fs::read_to_string(f.target.repo.join("file.txt")).unwrap(),
        "changed with local WIP\n"
    );
}

#[test]
fn roll_initializes_missing_baseline_without_stashing_nested_framework() {
    let f = Fixture::new();
    git(&f.target.repo, &["update-ref", "-d", spo::UPSTREAM_REF]).unwrap();
    fs::remove_file(f.target.state_dir().unwrap().join("stack.json")).unwrap();
    let framework_head = git(&f.source, &["rev-parse", "HEAD"]).unwrap();
    fs::write(f.source.join("local-framework.txt"), "Framework WIP\n").unwrap();
    assert!(
        git(&f.target.repo, &["status", "--porcelain"])
            .unwrap()
            .contains("tuffite/")
    );
    f.success(&["roll", "--chromium-revision", &f.new_base]);
    assert_eq!(
        git(&f.target.repo, &["rev-parse", spo::UPSTREAM_REF]).unwrap(),
        f.new_base
    );
    assert_eq!(
        git(&f.source, &["rev-parse", "HEAD"]).unwrap(),
        framework_head
    );
    assert!(f.source.join("local-framework.txt").exists());
    assert!(git(&f.target.repo, &["stash", "list"]).unwrap().is_empty());
}
