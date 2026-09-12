//! File-based JSON interface for external orchestration. No Framework-specific policy.
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::{Value, json};
use spo::{Group, Target};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    operation: String,
    target: Option<Target>,
    workspace: Option<PathBuf>,
    config: Option<PathBuf>,
    #[serde(default)]
    bases: BTreeMap<String, String>,
    base: Option<String>,
    oid: Option<String>,
    groups: Option<Vec<Group>>,
}

pub fn run(request: &Path, response: &Path) -> Result<()> {
    let request: Request = serde_json::from_slice(
        &fs::read(request).with_context(|| format!("read bridge request {}", request.display()))?,
    )?;
    let result = dispatch(request)?;
    spo::write_json(response, &result)
}

fn dispatch(request: Request) -> Result<Value> {
    if request.operation == "load" {
        return Ok(serde_json::to_value(spo::load_targets_with_bases(
            request.workspace.as_deref().context("workspace")?,
            request.config.as_deref().context("config")?,
            &request.bases,
        )?)?);
    }
    let target = request.target.context("target")?;
    let result = match request.operation.as_str() {
        "baseline" => {
            let base = target
                .upstream_base()?
                .unwrap_or_else(|| target.base.clone());
            target.initialize_upstream(&base)?;
            json!(base)
        }
        "inspect" => json!({
            "dirty": target.dirty()?,
            "pending": target.pending()?,
            "native": target.native_operation()?,
        }),
        "status" => {
            let status = target.status()?;
            json!({"current": status.current, "pending": status.pending})
        }
        "groups" => serde_json::to_value(target.groups(request.base.as_deref().context("base")?)?)?,
        "filenames" => serde_json::to_value(target.filenames()?)?,
        "stash" => serde_json::to_value(target.stash()?)?,
        "restore-stash" => {
            target.restore_stash(request.oid.as_deref().context("oid")?)?;
            Value::Null
        }
        "drop-stash" => {
            target.drop_stash(request.oid.as_deref().context("oid")?)?;
            Value::Null
        }
        "record" => {
            target.record_stack(
                request.base.as_deref().context("base")?,
                request.groups.as_deref().context("groups")?,
            )?;
            Value::Null
        }
        "export" => {
            target.export(true, false)?;
            Value::Null
        }
        "apply" => {
            target.apply(false)?;
            Value::Null
        }
        "continue" => {
            target.resume(false)?;
            Value::Null
        }
        "abort" => {
            target.abort()?;
            Value::Null
        }
        operation => bail!("unknown bridge operation: {operation}"),
    };
    Ok(result)
}
