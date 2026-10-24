use crate::Error;
use crate::parser::parse;
use crate::schema::{Schema, check_types, layout};
use std::collections::BTreeSet;
use std::path::PathBuf;

fn resolve(
    ty: &mut crate::Type,
    enums: &std::collections::BTreeMap<String, Vec<(String, i32)>>,
) -> Result<(), String> {
    match ty {
        crate::Type::Named(name) if enums.contains_key(name) => {
            *ty = crate::Type::Enum(name.clone());
        }
        crate::Type::Array(inner) => {
            resolve(inner, enums)?;
            if matches!(inner.as_ref(), crate::Type::Nullable(value) if value.is_value()) {
                return Err("arrays of nullable primitives are not supported".into());
            }
        }
        crate::Type::Nullable(inner) => resolve(inner, enums)?,
        _ => {}
    }
    Ok(())
}

/// Resolves the handle-free v0 data profile and computes wire field layouts.
/// Sources must belong to the same module, with imports loaded by `read_sources`.
pub fn compile(sources: &[(PathBuf, String)]) -> Result<Schema, Error> {
    let mut schema = Schema::default();
    let mut first = true;
    let mut origins = std::collections::BTreeMap::new();
    for (path, source) in sources {
        let parsed = parse(source).map_err(|error| error.with_source(path, source))?;
        if first {
            schema.module.clone_from(&parsed.module);
            first = false;
        }
        if parsed.module != schema.module {
            return Err(format!("{}: expected module {}", path.display(), schema.module).into());
        }
        for name in parsed.structures.keys().chain(parsed.enums.keys()) {
            origins.insert(name.clone(), (path, source, parsed.declaration_spans[name]));
        }
        for (name, structure) in parsed.structures {
            if schema.structures.insert(name.clone(), structure).is_some() {
                return Err(format!("duplicate Mojom type {name}").into());
            }
        }
        for (name, values) in parsed.enums {
            if schema.enums.insert(name.clone(), values).is_some() {
                return Err(format!("duplicate Mojom enum {name}").into());
            }
        }
        schema.methods.extend(parsed.methods);
    }
    if first {
        return Err("no Mojom sources".into());
    }
    let mut names = BTreeSet::new();
    let mut ordinals = BTreeSet::new();
    for method in &schema.methods {
        if !names.insert((&method.interface, &method.name))
            || !ordinals.insert((&method.interface, method.ordinal))
        {
            return Err("duplicate Mojom endpoint".into());
        }
    }

    for (name, structure) in &mut schema.structures {
        if schema.enums.contains_key(name) {
            return Err(format!("duplicate Mojom type {name}").into());
        }
        for field in &mut structure.fields {
            resolve(&mut field.ty, &schema.enums).map_err(|message| {
                let (path, source, _) = origins[name];
                Error::at(message, field.source_span.0, field.source_span.1)
                    .with_source(path, source)
            })?;
        }
    }
    for name in schema.structures.keys() {
        check_types(&schema, name, &mut Vec::new(), &mut 4096).map_err(|error| {
            let (path, source, _) = origins[error.declaration().unwrap_or(name)];
            error.with_source(path, source)
        })?;
    }
    for structure in schema.structures.values_mut() {
        layout(structure);
    }
    Ok(schema)
}
