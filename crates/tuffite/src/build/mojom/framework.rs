//! Framework Rust clients over the standalone data codec.
use super::{descriptor, naming::lower_first, rust};
use quote::quote;
use std::collections::{BTreeMap, BTreeSet};
use tuffite_mojom::{Schema, Type};

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
        let res: syn::Ident = syn::parse_str(&method.response).map_err(|e| e.to_string())?;
        let result = schema.structures[&method.response]
            .fields
            .first()
            .map_or_else(|| Ok(quote!(())), |field| rust::rust_type(&field.ty))?;
        let channel = format!("tuffite.{}.{}", method.interface, method.name);
        let decode = if schema.structures[&method.response].fields.is_empty() {
            quote! { let _ = response; Ok(()) }
        } else {
            quote! { Ok(response.result) }
        };
        modules.entry(ns).or_default().push(quote! {
            pub type #name = super::#req;
            impl ::tuffite::ShellApiRequest for super::#req {
                type Response = #result;
                const CHANNEL: &'static str = #channel;
                fn encode(&self) -> Result<Vec<u8>, ::tuffite::__private::codec::CodecError> { ::tuffite::__private::mojom::Struct::encode_mojom(self) }
                fn decode(bytes: &[u8]) -> Result<Self::Response, ::tuffite::__private::codec::CodecError> {
                    let response = <super::#res as ::tuffite::__private::mojom::Struct>::decode_mojom(bytes)?;
                    #decode
                }
            }
        });
    }
    let modules = modules.into_iter().map(|(name, methods)| {
        fn collect(ty: &Type, names: &mut BTreeSet<String>) {
            match ty {
                Type::Named(name) | Type::Enum(name) => {
                    names.insert(name.clone());
                }
                Type::Array(inner) | Type::Nullable(inner) => collect(inner, names),
                _ => {}
            }
        }
        let mut names = BTreeSet::new();
        for method in &schema.methods {
            if lower_first(&method.interface) == name {
                for field in &schema.structures[&method.response].fields {
                    collect(&field.ty, &mut names);
                }
            }
        }
        let names = names
            .iter()
            .map(|name| syn::parse_str::<syn::Ident>(name).unwrap())
            .collect::<Vec<_>>();
        let imports = if names.is_empty() {
            quote!()
        } else {
            quote!(use super::{#(#names),*};)
        };
        let name: syn::Ident = syn::parse_str(&name).unwrap();
        quote! { pub mod #name { #imports #(#methods)* } }
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
