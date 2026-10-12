use std::{env, fs, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-env-changed=TUFFITE_BUILD_COMMIT");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var_os("CARGO_FEATURE_CLI").is_none() {
        return;
    }
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&crate_dir).parent().unwrap().parent().unwrap();
    let commit = env::var("TUFFITE_BUILD_COMMIT")
        .ok()
        .or_else(|| git_commit(root))
        .or_else(|| archive_commit(Path::new(&crate_dir)))
        .unwrap_or_else(|| "unknown".into());
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    println!("cargo:rustc-env=TUFFITE_CLI_VERSION={version} ({commit})");
}

fn git_commit(root: &Path) -> Option<String> {
    // Do not accidentally report the containing Chromium repository's commit.
    if !root.join(".git").exists() {
        return None;
    }
    let git_dir = git(root, &["rev-parse", "--absolute-git-dir"])?;
    let git_dir = Path::new(&git_dir);
    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());
    println!(
        "cargo:rerun-if-changed={}",
        git_dir.join("packed-refs").display()
    );
    if let Some(reference) = git(root, &["symbolic-ref", "-q", "HEAD"]) {
        println!(
            "cargo:rerun-if-changed={}",
            git_dir.join(reference).display()
        );
    }
    git(root, &["rev-parse", "HEAD"])
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn archive_commit(crate_dir: &Path) -> Option<String> {
    let path = crate_dir.join(".cargo_vcs_info.json");
    println!("cargo:rerun-if-changed={}", path.display());
    // Cargo generates this small file when packaging a crate.
    let info = fs::read_to_string(path).ok()?;
    let value = info
        .split('"')
        .skip_while(|value| *value != "sha1")
        .nth(2)?;
    (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())).then(|| value.into())
}
