mod bridge;
use anyhow::{Context, Result, bail, ensure};
use clap::{Args, Parser, Subcommand, ValueEnum};
use spo::{Target, git, ui};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser)]
#[command(about = "Git-backed stacked patch orchestration", version)]
struct Cli {
    /// Target configuration; defaults to the current Framework's patches/config.json.
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    /// Root relative to which configured repository paths are resolved.
    #[arg(long, global = true)]
    workspace: Option<PathBuf>,
    #[arg(long, global = true)]
    non_interactive: bool,
    #[arg(long, global = true)]
    dry_run: bool,
    #[arg(long, global = true)]
    yes: bool,
    /// Preserve uncommitted work before history operations.
    #[arg(long, global = true, value_enum, default_value = "reject")]
    dirty: Dirty,
    /// Initialize a missing upstream reference from a known commit (repeat per target).
    #[arg(long, global = true, value_name = "TARGET=COMMIT")]
    base: Vec<String>,
    #[command(subcommand)]
    command: Option<Action>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Dirty {
    Reject,
    Stash,
}

#[derive(Args, Default)]
struct Selection {
    targets: Vec<String>,
    #[arg(long, conflicts_with = "targets")]
    all: bool,
}

#[derive(Subcommand)]
enum Action {
    /// Internal structured interface for external task runners.
    #[command(hide = true)]
    Bridge { request: PathBuf, response: PathBuf },
    /// Show local work, series size, and interrupted operations.
    Status(Selection),
    /// Export HEAD into ordered mailbox files. WIP is not committed implicitly.
    Export {
        #[command(flatten)]
        selection: Selection,
        #[arg(long)]
        verify_apply: bool,
    },
    /// Check semantic content, or exact canonical output with --strict.
    Check {
        #[command(flatten)]
        selection: Selection,
        #[arg(long)]
        strict: bool,
    },
    /// Apply checked-in mailboxes on their configured base.
    Apply(Selection),
    /// Edit a rebase-style plan; group filenames may contain multiple commits.
    Edit {
        target: Option<String>,
        #[arg(long)]
        plan: Option<PathBuf>,
    },
    /// Assign consecutive commits to one mailbox file.
    Group {
        target: Option<String>,
        #[arg(long)]
        patch: String,
        #[arg(long, num_args = 1.., required = true)]
        commits: Vec<String>,
    },
    /// Commit selected files as a new patch; other WIP remains untouched.
    New {
        target: Option<String>,
        #[arg(short, long)]
        message: Option<String>,
        #[arg(long, num_args = 1..)]
        paths: Vec<String>,
        #[arg(long)]
        patch: Option<String>,
        #[arg(long)]
        export: bool,
    },
    /// Add selected files to an existing patch using a fixup rebase.
    Refresh {
        target: Option<String>,
        #[arg(long)]
        patch: String,
        #[arg(long, num_args = 1..)]
        paths: Vec<String>,
        #[arg(long)]
        export: bool,
    },
    /// Resume a Git operation or interrupted publication.
    Continue {
        target: Option<String>,
        /// Confirm that interrupted WIP restoration has been resolved manually.
        #[arg(long)]
        wip_restored: bool,
    },
    /// Restore the operation's original stack, preserving saved WIP.
    Abort { target: Option<String> },
    #[command(hide = true)]
    SequenceEditor {
        source: PathBuf,
        destination: PathBuf,
    },
    #[command(hide = true)]
    Checkpoint {
        #[arg(long)]
        repo: PathBuf,
        #[arg(long)]
        name: String,
    },
}

fn configuration(cli: &Cli) -> Result<(PathBuf, PathBuf)> {
    let current = std::env::current_dir()?;
    if let Some(config) = &cli.config {
        let config = if config.is_absolute() {
            config.clone()
        } else {
            current.join(config)
        };
        return Ok((cli.workspace.clone().unwrap_or(current), config));
    }
    let framework = current
        .ancestors()
        .find(|path| path.join("patches/config.json").is_file())
        .map(Path::to_path_buf)
        .or_else(|| std::env::var_os("TUFFITE_FRAMEWORK_DIR").map(PathBuf::from))
        .context("no patch configuration; use --config and --workspace")?;
    let workspace = cli.workspace.clone().unwrap_or(
        framework
            .parent()
            .and_then(Path::parent)
            .context("Framework workspace layout")?
            .into(),
    );
    Ok((workspace, framework.join("patches/config.json")))
}

fn select_targets<'a>(
    targets: &'a [Target],
    selection: &Selection,
    disabled: bool,
) -> Result<Vec<&'a Target>> {
    if selection.all {
        return Ok(targets.iter().collect());
    }
    if !selection.targets.is_empty() {
        let mut result = Vec::new();
        for name in &selection.targets {
            let target = targets
                .iter()
                .find(|t| &t.name == name)
                .with_context(|| format!("unknown target {name}"))?;
            ensure!(
                !result.iter().any(|t: &&Target| t.name == *name),
                "duplicate target {name}"
            );
            result.push(target);
        }
        return Ok(result);
    }
    if targets.len() == 1 {
        return Ok(vec![&targets[0]]);
    }
    let index = ui::select(
        "Select patch target",
        &targets.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
        disabled,
    )?;
    Ok(vec![&targets[index]])
}

fn single<'a>(targets: &'a [Target], name: Option<&String>, disabled: bool) -> Result<&'a Target> {
    Ok(select_targets(
        targets,
        &Selection {
            targets: name.cloned().into_iter().collect(),
            all: false,
        },
        disabled,
    )?[0])
}

fn selected_paths(target: &Target, paths: &[String], disabled: bool) -> Result<Vec<String>> {
    let paths = if paths.is_empty() {
        let choices = target.changed_paths()?;
        ui::multi_select("Select files to commit", &choices, disabled)?
            .into_iter()
            .map(|index| choices[index].clone())
            .collect()
    } else {
        paths.to_vec()
    };
    for path in &paths {
        ensure!(
            !Path::new(path).is_absolute()
                && Path::new(path)
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "paths must be repository-relative files: {path}"
        );
        ensure!(
            !target
                .exclude
                .iter()
                .any(|prefix| path == prefix || path.starts_with(&format!("{prefix}/"))),
            "{path} belongs to a separate repository"
        );
    }
    Ok(paths)
}

fn commit_paths(target: &Target, paths: &[String], message_args: &[&str]) -> Result<()> {
    // commit --only excludes unrelated staged files; add is needed for selected untracked files.
    let mut add = vec!["add", "--all", "--"];
    add.extend(paths.iter().map(String::as_str));
    spo::run_git(&target.repo, &add)?;
    let mut commit = vec!["commit", "--only"];
    commit.extend_from_slice(message_args);
    commit.push("--");
    commit.extend(paths.iter().map(String::as_str));
    spo::run_git(&target.repo, &commit)
}

#[allow(clippy::too_many_lines)] // Keep the CLI command dispatch together.
fn run(mut cli: Cli) -> Result<()> {
    match cli.command.as_ref() {
        Some(Action::SequenceEditor {
            source,
            destination,
        }) => {
            fs::copy(source, destination)?;
            return Ok(());
        }
        Some(Action::Checkpoint { repo, name }) => return spo::checkpoint(repo, name),
        Some(Action::Bridge { request, response }) => return bridge::run(request, response),
        _ => {}
    }
    let (workspace, config) = configuration(&cli)?;
    let mut bases = std::collections::BTreeMap::new();
    for value in &cli.base {
        let (name, revision) = value
            .split_once('=')
            .context("--base requires TARGET=COMMIT")?;
        ensure!(
            !name.is_empty() && !revision.is_empty(),
            "--base requires TARGET=COMMIT"
        );
        ensure!(
            bases.insert(name.to_owned(), revision.to_owned()).is_none(),
            "duplicate --base target {name}"
        );
    }
    let targets = spo::load_targets_with_bases(&workspace, &config, &bases)?;
    for name in bases.keys() {
        ensure!(
            targets.iter().any(|target| &target.name == name),
            "unknown --base target {name}"
        );
    }
    for target in &targets {
        if bases.contains_key(&target.name) {
            if cli.dry_run {
                target.commits(&target.base)?;
            } else {
                target.initialize_upstream(&target.base)?;
            }
        } else if !cli.dry_run {
            target.repair_upstream()?;
        }
    }
    if cli.command.is_none() {
        for target in &targets {
            print_status(target)?;
        }
        let choices = [
            "export\tReview and export committed patches",
            "new\tCreate a patch from selected files",
            "refresh\tUpdate an existing patch",
            "edit\tEdit stack with a rebase plan",
            "check\tVerify patch content and order",
            "apply\tApply the committed patches",
            "continue\tResume an interrupted operation",
            "abort\tAbort an interrupted operation",
        ];
        let selected = ui::select(
            "Patch actions",
            &choices.map(str::to_owned),
            cli.non_interactive,
        )?;
        cli.command = Some(match selected {
            0 => Action::Export {
                selection: Selection::default(),
                verify_apply: true,
            },
            1 => Action::New {
                target: None,
                message: None,
                paths: vec![],
                patch: None,
                export: true,
            },
            2 => Action::Refresh {
                target: None,
                patch: ui::input("Patch filename", false)?,
                paths: vec![],
                export: true,
            },
            3 => Action::Edit {
                target: None,
                plan: None,
            },
            4 => Action::Check {
                selection: Selection::default(),
                strict: false,
            },
            5 => Action::Apply(Selection::default()),
            6 => Action::Continue {
                target: None,
                wip_restored: false,
            },
            _ => Action::Abort { target: None },
        });
    }
    let stash = matches!(cli.dirty, Dirty::Stash);
    match cli.command.context("command")? {
        Action::Status(selection) => {
            for target in select_targets(&targets, &selection, cli.non_interactive)? {
                print_status(target)?;
            }
        }
        Action::Export {
            selection,
            verify_apply,
        } => {
            for target in select_targets(&targets, &selection, cli.non_interactive)? {
                let groups = target.groups(&target.base)?;
                println!(
                    "{}: export {} commits into {} files",
                    target.name,
                    groups.iter().map(|g| g.commits.len()).sum::<usize>(),
                    groups.len()
                );
                if !cli.dry_run {
                    target.export(verify_apply, stash)?;
                }
            }
        }
        Action::Check { selection, strict } => {
            for target in select_targets(&targets, &selection, cli.non_interactive)? {
                if cli.dry_run {
                    println!("{}: check strict={strict}", target.name);
                } else {
                    target.check(strict)?;
                }
            }
        }
        Action::Apply(selection) => {
            for target in select_targets(&targets, &selection, cli.non_interactive)? {
                if cli.dry_run {
                    println!("{}: apply checked-in mailboxes", target.name);
                } else {
                    target.apply(stash)?;
                }
            }
        }
        Action::Group {
            target,
            patch,
            commits,
        } => {
            let target = single(&targets, target.as_ref(), cli.non_interactive)?;
            if cli.dry_run {
                println!(
                    "{}: assign {} commits to {patch}",
                    target.name,
                    commits.len()
                );
            } else {
                target.group(&patch, &commits)?;
            }
        }
        Action::Edit { target, plan } => {
            let target = single(&targets, target.as_ref(), cli.non_interactive)?;
            let text = if let Some(plan) = plan {
                fs::read_to_string(plan)?
            } else {
                ensure!(
                    ui::interactive(cli.non_interactive),
                    "edit requires --plan in non-interactive mode"
                );
                let path = target.state_dir()?.join("plan.todo");
                fs::create_dir_all(path.parent().context("plan parent")?)?;
                fs::write(&path, target.plan()?)?;
                let editor = git(&target.repo, &["var", "GIT_EDITOR"])?;
                ensure!(
                    Command::new("sh")
                        .args([
                            "-c",
                            &format!(
                                "{editor} {}",
                                spo::shell_quote(path.to_str().context("plan path")?)
                            )
                        ])
                        .status()?
                        .success(),
                    "editor failed"
                );
                fs::read_to_string(path)?
            };
            target.parse_plan(&text)?;
            if cli.dry_run {
                print!("{text}");
            } else {
                target.edit(&text, &std::env::current_exe()?, stash, cli.non_interactive)?;
            }
        }
        Action::New {
            target,
            message,
            paths,
            patch,
            export,
        } => {
            let target = single(&targets, target.as_ref(), cli.non_interactive)?;
            ensure!(
                target.pending()?.is_none(),
                "finish the current operation first"
            );
            let message =
                message.map_or_else(|| ui::input("Commit message", cli.non_interactive), Ok)?;
            ensure!(!message.is_empty(), "commit message is required");
            let paths = selected_paths(target, &paths, cli.non_interactive)?;
            if cli.dry_run {
                println!(
                    "{}: commit selected paths {paths:?} as {message:?}",
                    target.name
                );
            } else {
                commit_paths(target, &paths, &["-m", &message])?;
                if let Some(patch) = patch {
                    target.group(&patch, &[git(&target.repo, &["rev-parse", "HEAD"])?])?;
                }
                if export {
                    target.export(true, false)?;
                }
            }
        }
        Action::Refresh {
            target,
            patch,
            paths,
            export,
        } => {
            let target = single(&targets, target.as_ref(), cli.non_interactive)?;
            ensure!(
                target.pending()?.is_none(),
                "finish the current operation first"
            );
            let groups = target.groups(&target.base)?;
            let group = groups
                .iter()
                .find(|g| g.name == patch)
                .context("unknown patch group")?;
            let tip = group.commits.last().context("empty patch group")?;
            let paths = selected_paths(target, &paths, cli.non_interactive)?;
            if cli.dry_run {
                println!("{}: refresh {patch} with {paths:?}", target.name);
            } else {
                commit_paths(target, &paths, &["--fixup", tip])?;
                let fixup = git(&target.repo, &["rev-parse", "HEAD"])?;
                let mut plan = String::new();
                for group in groups {
                    for commit in group.commits {
                        writeln!(plan, "pick {commit} {}", group.name)?;
                    }
                    if group.name == patch {
                        writeln!(plan, "fixup {fixup} {patch}")?;
                    }
                }
                target.edit(&plan, &std::env::current_exe()?, stash, cli.non_interactive)?;
                if export {
                    target.export(true, false)?;
                }
            }
        }
        Action::Continue {
            target,
            wip_restored,
        } => {
            let target = single(&targets, target.as_ref(), cli.non_interactive)?;
            if cli.dry_run {
                println!("{}: continue {:?}", target.name, target.pending()?);
            } else {
                target.resume(wip_restored)?;
            }
        }
        Action::Abort { target } => {
            let target = single(&targets, target.as_ref(), cli.non_interactive)?;
            if cli.dry_run {
                println!("{}: abort {:?}", target.name, target.pending()?);
            } else {
                target.abort()?;
            }
        }
        Action::SequenceEditor { .. } | Action::Checkpoint { .. } | Action::Bridge { .. } => {
            bail!("internal command dispatch")
        }
    }
    Ok(())
}

fn print_status(target: &Target) -> Result<()> {
    let status = target.status()?;
    println!(
        "Target: {}\nBase: {}\nCommits: {}\nPatch files: {}\nWorking changes: {}\nSeries: {}\nPending: {}\n",
        target.name,
        target.base,
        status.commits,
        status.files,
        status.dirty,
        if status.current {
            "current"
        } else {
            "needs review"
        },
        status.pending.as_deref().unwrap_or("none")
    );
    Ok(())
}

fn main() {
    if let Err(error) = run(Cli::parse()) {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}
