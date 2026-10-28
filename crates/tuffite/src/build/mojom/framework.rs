//! Framework protocol data models over the standalone data codec.
use super::{descriptor, naming::lower_first, rust};
use quote::quote;
use std::collections::BTreeMap;
use tuffite_mojom::Schema;

pub(super) fn rust(schema: &Schema) -> Result<String, String> {
    let structures = schema
        .structures
        .values()
        .map(rust::structure)
        .collect::<Result<Vec<_>, _>>()?;
    let enums = rust::enum_bindings(schema)?;
    let mut modules: BTreeMap<String, Vec<proc_macro2::TokenStream>> = BTreeMap::new();
    for method in &schema.methods {
        let ns = lower_first(&method.interface);
        let name: syn::Ident = syn::parse_str(&method.name).map_err(|e| e.to_string())?;
        let req: syn::Ident = syn::parse_str(&method.request).map_err(|e| e.to_string())?;
        modules
            .entry(ns)
            .or_default()
            .push(quote! { pub type #name = super::#req; });
    }
    let modules = modules.into_iter().map(|(name, methods)| {
        let name: syn::Ident = syn::parse_str(&name).unwrap();
        quote! { pub mod #name { #(#methods)* } }
    });
    let syntax = syn::parse2(quote! { #enums #(#structures)* #(#modules)* })
        .map_err(|e| format!("Framework Rust generation: {e}"))?;
    Ok(format!(
        "// Generated from Framework Mojom contracts.\n{}",
        prettyplease::unparse(&syntax)
    ))
}

pub(super) fn entries(schema: &Schema) -> Vec<serde_json::Value> {
    schema
        .methods
        .iter()
        .map(|method| {
            serde_json::json!([
                format!(
                    "{}.{}",
                    lower_first(&method.interface),
                    lower_first(&method.name)
                ),
                "0",
                "0",
                descriptor(schema, &method.request),
                descriptor(schema, &method.response),
                format!("tuffite.{}.{}", method.interface, method.name)
            ])
        })
        .collect()
}
