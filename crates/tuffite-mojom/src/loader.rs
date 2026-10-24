use crate::Error;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::parser::parse;

fn load(
    root: &Path,
    path: &Path,
    stack: &mut Vec<PathBuf>,
    seen: &mut BTreeSet<PathBuf>,
    sources: &mut Vec<(PathBuf, String)>,
) -> Result<(), Error> {
    if path.extension().and_then(|value| value.to_str()) != Some("mojom")
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "Mojom import must be a root-relative .mojom path: {}",
            path.display()
        )
        .into());
    }
    if stack.iter().any(|value| value == path) {
        return Err(format!("cyclic Mojom import {}", path.display()).into());
    }
    if seen.contains(path) {
        return Ok(());
    }
    if stack.len() >= 64 || seen.len() >= 128 {
        return Err("Mojom import limit exceeded".into());
    }
    let absolute = root
        .join(path)
        .canonicalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    if !absolute.starts_with(canonical_root) {
        return Err("Mojom import escapes source root".into());
    }
    let source = std::fs::read_to_string(absolute).map_err(|error| error.to_string())?;
    let parsed = parse(&source).map_err(|error| error.with_source(path, &source))?;
    stack.push(path.into());
    for (import, &(offset, length)) in parsed.imports.iter().zip(&parsed.import_spans) {
        load(root, import, stack, seen, sources).map_err(|error| {
            if error.has_source() { return error; }
            Error::at(error.to_string(), offset, length)
                .with_source(path, &source)
                .with_help("Imports must name existing .mojom files within the source root; import cycles are forbidden.")
        })?;
    }
    stack.pop();
    seen.insert(path.into());
    sources.push((path.into(), source));
    Ok(())
}

/// Loads confined, root-relative Mojom sources and their imports.
pub fn read_sources(root: &Path, definitions: &[PathBuf]) -> Result<Vec<(PathBuf, String)>, Error> {
    let mut sources = Vec::new();
    let mut seen = BTreeSet::new();
    for path in definitions {
        load(root, path, &mut Vec::new(), &mut seen, &mut sources)?;
    }
    Ok(sources)
}
