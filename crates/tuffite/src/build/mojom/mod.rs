//! Tuffite-specific bindings over the standalone Mojom frontend.
use super::{command_id, is_identifier};
use naming::{lower_first, snake};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use tuffite_mojom::{Method, Schema, Type};
mod framework;
mod javascript;
mod naming;
mod rust;

use super::shell_api::GeneratedBindings;

/// Compiles the application data profile without invoking external tools.
pub(super) fn generate(
    sources: &[(PathBuf, String)],
    application: &str,
) -> Result<GeneratedBindings, String> {
    let schema =
        tuffite_mojom::compile(sources).map_err(|error| super::render_mojom_error(&error))?;
    let root = &schema.module;
    if !root.split('.').all(is_identifier)
        || root == "tuffite"
        || root.starts_with("tuffite.")
        || application.is_empty()
    {
        return Err("invalid or reserved Mojom module or application identifier".into());
    }
    if schema.methods.is_empty() {
        return Err("Mojom has no unary methods".into());
    }
    let mut paths = BTreeSet::new();
    let mut ordinals = BTreeSet::new();
    let mut handlers = BTreeSet::new();
    for method in &schema.methods {
        let handler = snake(&method.name);
        if schema.structures.contains_key(&method.interface)
            || handler.starts_with("__tuffite_")
            || !handlers.insert((&method.interface, handler))
        {
            return Err("Mojom service trait name collision".into());
        }
        if !paths.insert((lower_first(&method.interface), lower_first(&method.name)))
            || !ordinals.insert((method.interface.clone(), method.ordinal))
        {
            return Err("duplicate Mojom endpoint".into());
        }
        for name in [lower_first(&method.interface), lower_first(&method.name)] {
            if ["then", "__proto__", "constructor", "prototype"].contains(&name.as_str()) {
                return Err(format!("reserved Mojom facade name {name}"));
            }
        }
    }
    for structure in schema.structures.values() {
        let mut names = BTreeSet::new();
        for field in &structure.fields {
            let normalized = snake(&field.name);
            if ["self", "super", "crate", "_"].contains(&normalized.as_str())
                || !names.insert(normalized)
            {
                return Err(format!(
                    "{}: field name collision or reserved Rust field",
                    structure.name
                ));
            }
        }
    }
    emit(&schema, root, application)
}
fn type_descriptor(schema: &Schema, ty: &Type) -> Value {
    match ty {
        Type::Named(target) => descriptor(schema, target),
        Type::Enum(name) => {
            json!({"enum": schema.enums[name].iter().map(|(_,value)| *value).collect::<Vec<_>>()})
        }
        Type::Array(inner) => {
            json!({"array": type_descriptor(schema, inner), "width": inner.size(), "boolean": **inner == Type::Bool})
        }
        Type::Nullable(inner) => json!({"nullable": type_descriptor(schema, inner)}),
        Type::Scalar(value) => json!(value),
        Type::Bool => json!("bool"),
        Type::String => json!("string"),
        Type::Bytes => json!("bytes"),
    }
}
fn descriptor(schema: &Schema, name: &str) -> Value {
    let structure = &schema.structures[name];
    json!({"size":structure.size,"fields":structure.fields.iter().map(|field| {
        let ty = type_descriptor(schema, &field.ty);
        if let Some((offset, bit)) = field.presence { json!([field.name,field.offset,field.bit,ty,[offset,bit]]) }
        else { json!([field.name,field.offset,field.bit,ty]) }
    }).collect::<Vec<_>>()})
}

struct Endpoint<'a> {
    method: &'a Method,
    channel: String,
    id: u64,
    fingerprint: u64,
}
fn emit(schema: &Schema, root: &str, application: &str) -> Result<GeneratedBindings, String> {
    let mut command_ids = BTreeMap::new();
    let mut endpoints = Vec::new();
    let mut entries = Vec::new();
    let interfaces = schema
        .methods
        .iter()
        .map(|method| method.interface.as_str())
        .collect::<BTreeSet<_>>();
    for interface in interfaces {
        for method in schema
            .methods
            .iter()
            .filter(|method| method.interface == interface)
        {
            let path = format!("{}.{}", lower_first(interface), lower_first(&method.name));
            let canonical = format!("{}.{}", method.interface, method.name);
            let channel = format!("{root}.{canonical}");
            let id = command_id(&format!("{application}:{channel}"));
            let request = descriptor(schema, &method.request);
            let response = descriptor(schema, &method.response);
            let fingerprint =
                command_id(&format!("mojom.v0:{}:{request}:{response}", method.ordinal));
            command_ids.insert(channel.clone(), format!("{id:016x}"));
            entries.push(json!([
                path,
                format!("0x{id:016x}"),
                format!("0x{fingerprint:016x}"),
                request,
                response,
                channel
            ]));
            endpoints.push(Endpoint {
                method,
                channel,
                id,
                fingerprint,
            });
        }
    }
    let types = schema
        .structures
        .keys()
        .map(|name| (name.clone(), descriptor(schema, name)))
        .collect::<BTreeMap<_, _>>();
    let rust = rust::generate(schema, &endpoints)?;
    let js = javascript::generate(schema, &entries, &types, root)?;
    Ok(GeneratedBindings {
        command_ids,
        rust_files: BTreeMap::from([("mojom.rs".into(), rust.clone())]),
        rust,
        javascript: js.javascript,
        declarations: js.declarations,
    })
}

#[cfg(test)]
mod tests;

pub(super) fn generate_framework(
    sources: &[(PathBuf, String)],
) -> Result<GeneratedBindings, String> {
    let schema =
        tuffite_mojom::compile(sources).map_err(|error| super::render_mojom_error(&error))?;
    if schema.module != "tuffite" {
        return Err("Framework contracts must use module tuffite".into());
    }
    let types = schema
        .structures
        .keys()
        .map(|name| (name.clone(), descriptor(&schema, name)))
        .collect();
    let js = javascript::generate(&schema, &framework::entries(&schema), &types, "tuffite")?;
    let rust = framework::rust(&schema)?;
    Ok(GeneratedBindings {
        command_ids: schema
            .methods
            .iter()
            .map(|method| {
                (
                    format!("tuffite.{}.{}", method.interface, method.name),
                    "0000000000000000".into(),
                )
            })
            .collect(),
        rust_files: BTreeMap::from([("mojom.rs".into(), rust.clone())]),
        rust,
        javascript: js.javascript,
        declarations: js.declarations,
    })
}
