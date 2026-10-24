//! Rust bindings are built as tokens, validated by syn and formatted by prettyplease.
use std::collections::BTreeSet;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, LitInt};
use tuffite_mojom::{Schema, Structure, Type};

use super::Endpoint;
use super::naming::{lower_first, snake};

fn ident(name: &str) -> Result<Ident, String> {
    syn::parse_str(name)
        .map_err(|error| format!("invalid generated Rust identifier {name}: {error}"))
}

fn field_ident(name: &str) -> Result<Ident, String> {
    ident(&format!("r#{}", snake(name)))
}

fn integer(value: usize) -> LitInt {
    LitInt::new(&value.to_string(), Span::call_site())
}

fn hex(value: u64) -> LitInt {
    LitInt::new(
        &format!(
            "0x{:04x}_{:04x}_{:04x}_{:04x}",
            value >> 48,
            (value >> 32) & 0xffff,
            (value >> 16) & 0xffff,
            value & 0xffff
        ),
        Span::call_site(),
    )
}

pub(super) fn rust_type(ty: &Type) -> Result<TokenStream, String> {
    Ok(match ty {
        Type::Scalar(name) | Type::Named(name) | Type::Enum(name) => {
            let name = ident(name)?;
            quote!(#name)
        }
        Type::Nullable(inner) => {
            let inner = rust_type(inner)?;
            quote!(Option<#inner>)
        }
        Type::Array(inner) => {
            let inner = rust_type(inner)?;
            quote!(Vec<#inner>)
        }
        Type::Bool => quote!(bool),
        Type::String => quote!(String),
        Type::Bytes => quote!(Vec<u8>),
    })
}

pub(super) fn structure(structure: &Structure) -> Result<TokenStream, String> {
    let name = ident(&structure.name)?;
    let size = integer(structure.size);
    let mut fields = Vec::new();
    let mut writes = Vec::new();
    let mut reads = Vec::new();
    for field in &structure.fields {
        let name = field_ident(&field.name)?;
        let ty = rust_type(&field.ty)?;
        let offset = integer(field.offset);
        let bit = integer(usize::from(field.bit));
        let at = quote!(base + #offset);
        fields.push(quote!(pub #name: #ty,));
        if let Some((flag, flag_bit)) = field.presence {
            let flag = integer(flag);
            let flag_bit = integer(flag_bit as usize);
            writes.push(quote! {
                w.boolean(base + #flag, #flag_bit, self.#name.is_some())?;
                if let Some(value) = &self.#name { ::tuffite::__private::mojom::Value::put(value, w, #at, #bit)?; }
            });
            reads.push(quote! { #name: if r.boolean(base + #flag, #flag_bit)? { Some(::tuffite::__private::mojom::Value::get(r, #at, #bit)?) } else { None }, });
            continue;
        }
        writes
            .push(quote! { ::tuffite::__private::mojom::Value::put(&self.#name, w, #at, #bit)?; });
        let read = quote! { <#ty as ::tuffite::__private::mojom::Value>::get(r, #at, #bit)? };
        reads.push(quote!(#name: #read,));
    }
    Ok(quote! {
        #[derive(Clone, Debug, Default, PartialEq, ::tuffite::__private::serde::Serialize, ::tuffite::__private::serde::Deserialize)]
        #[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
        pub struct #name { #(#fields)* }
        impl ::tuffite::__private::mojom::Value for #name {
            const WIDTH: usize = 8;
            const REFERENCE: bool = true;
            fn put(&self, w: &mut ::tuffite::__private::mojom::Encoder, at: usize, _: u8) -> Result<(), ::tuffite::__private::codec::CodecError> { w.structure(at, self) }
            fn get(r: &mut ::tuffite::__private::mojom::Decoder<'_>, at: usize, _: u8) -> Result<Self, ::tuffite::__private::codec::CodecError> { r.structure(at) }
        }
        impl ::tuffite::__private::mojom::Struct for #name {
            const SIZE: usize = #size;
            fn write(
                &self,
                w: &mut ::tuffite::__private::mojom::Encoder,
                base: usize,
            ) -> Result<(), ::tuffite::__private::codec::CodecError> {
                #(#writes)*
                let _ = (w, base);
                Ok(())
            }
            fn read(
                r: &mut ::tuffite::__private::mojom::Decoder<'_>,
                base: usize,
            ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
                let value = Self { #(#reads)* };
                let _ = (r, base);
                Ok(value)
            }
        }
    })
}

fn module_type(ty: &Type) -> Result<TokenStream, String> {
    match ty {
        Type::Named(name) | Type::Enum(name) => {
            let name = ident(name)?;
            Ok(quote!(super::#name))
        }
        Type::Array(inner) => {
            let inner = module_type(inner)?;
            Ok(quote!(::std::vec::Vec<#inner>))
        }
        Type::Nullable(inner) => {
            let inner = module_type(inner)?;
            Ok(quote!(::std::option::Option<#inner>))
        }
        _ => rust_type(ty),
    }
}

fn endpoint(schema: &Schema, endpoint: &Endpoint<'_>) -> Result<TokenStream, String> {
    let method = endpoint.method;
    let name = ident(&method.name)?;
    let request_name = ident(&method.request)?;
    let request = quote!(super::#request_name);
    let response_name = ident(&method.response)?;
    let response = quote!(super::#response_name);
    let channel = &endpoint.channel;
    let id = hex(endpoint.id);
    let fingerprint = hex(endpoint.fingerprint);
    let response_field = schema.structures[&method.response].fields.first();
    let result = response_field.map_or_else(|| Ok(quote!(())), |field| module_type(&field.ty))?;
    let response_fields = response_field
        .map(|field| {
            let name = field_ident(&field.name)?;
            Ok::<_, String>(if snake(&field.name) == "result" {
                quote!(#name)
            } else {
                quote!(#name: result)
            })
        })
        .transpose()?;
    let types = schema.structures[&method.request]
        .fields
        .iter()
        .map(|field| module_type(&field.ty))
        .collect::<Result<Vec<_>, _>>()?;
    let arguments = schema.structures[&method.request]
        .fields
        .iter()
        .map(|field| field_ident(&field.name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(quote! {
        pub type #name = #request;
        impl #request {
            pub const COMMAND: ::tuffite::CommandDescriptor =
                ::tuffite::CommandDescriptor::new(#channel, #id, #fingerprint);
            pub fn __tuffite_decode(command: ::tuffite::Command<'_>) -> Result<Self, ::tuffite::CommandError> {
                if command.id() != Self::COMMAND.id() {
                    return Err(::tuffite::CommandError::UnknownCommand);
                }
                if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                    return Err(::tuffite::CommandError::SchemaMismatch("Mojom schema mismatch".into()));
                }
                <Self as ::tuffite::__private::mojom::Struct>::decode_mojom(command.payload())
                    .map_err(|_| ::tuffite::CommandError::InvalidArgument("invalid Mojom payload".into()))
            }
            pub fn respond(result: Result<#result, ::tuffite::CommandError>) -> ::tuffite::CommandResult {
                let result = result?;
                let _ = &result;
                <#response as ::tuffite::__private::mojom::Struct>::encode_mojom(&#response { #response_fields })
                    .map_err(|_| ::tuffite::CommandError::Failed("Mojom encode failed".into()))
            }
            pub fn __tuffite_dispatch<A>(
                command: ::tuffite::Command<'_>, app: &mut A,
                handler: fn(&mut A #(, #types)*) -> Result<#result, ::tuffite::CommandError>,
            ) -> ::tuffite::CommandResult {
                let request = Self::__tuffite_decode(command)?;
                let _ = &request;
                Self::respond(handler(app #(, request.#arguments)*))
            }
        }
    })
}

fn interface_bindings(
    schema: &Schema,
    endpoints: &[Endpoint<'_>],
    interface: &str,
) -> Result<(TokenStream, TokenStream), String> {
    let namespace = ident(&lower_first(interface))?;
    let service = ident(interface)?;
    let interface_name = format!("{}.{}", schema.module, interface);
    let mut methods = Vec::new();
    let mut signatures = Vec::new();
    let mut branches = Vec::new();
    let mut descriptors = Vec::new();
    for binding in endpoints
        .iter()
        .filter(|binding| binding.method.interface == interface)
    {
        methods.push(endpoint(schema, binding)?);
        let method = binding.method;
        let handler = field_ident(&method.name)?;
        let request = ident(&method.request)?;
        let endpoint_name = ident(&method.name)?;
        let parameters = schema.structures[&method.request]
            .fields
            .iter()
            .map(|field| {
                let name = field_ident(&field.name)?;
                let ty = rust_type(&field.ty)?;
                Ok(quote!(#name: #ty))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let arguments = schema.structures[&method.request]
            .fields
            .iter()
            .map(|field| field_ident(&field.name))
            .collect::<Result<Vec<_>, _>>()?;
        let result = schema.structures[&method.response]
            .fields
            .first()
            .map_or_else(|| Ok(quote!(())), |field| rust_type(&field.ty))?;
        signatures.push(quote!(fn #handler(&self #(, #parameters)*) -> Result<#result, ::tuffite::CommandError>;));
        descriptors.push(quote!(#namespace::#endpoint_name::COMMAND));
        branches.push(quote! {
            if command.id() == #request::COMMAND.id() {
                let request = #request::__tuffite_decode(command)?;
                let _ = &request;
                return #request::respond(self.#handler(#(request.#arguments),*));
            }
        });
    }
    let module = quote! {
        #[allow(dead_code)]
        pub mod #namespace {
            #(#methods)*
            /// Registers this interface on the Browser backend.
            pub fn registry<B: super::#service + ::tuffite::Browser>() -> ::tuffite::__private::CommandRegistry<B> {
                ::tuffite::__private::CommandRegistry::new(
                    |browser, command| <B as super::#service>::__tuffite_dispatch(browser, command),
                    <B as super::#service>::__TUFFITE_COMMANDS,
                )
            }
        }
    };
    let service_trait = quote! {
        pub trait #service {
            #(#signatures)*
            #[doc(hidden)]
            const __TUFFITE_INTERFACE: &'static str = #interface_name;
            #[doc(hidden)]
            const __TUFFITE_COMMANDS: &'static [::tuffite::CommandDescriptor] = &[#(#descriptors),*];
            #[doc(hidden)]
            #[allow(clippy::too_many_lines)]
            fn __tuffite_dispatch(&self, command: ::tuffite::Command<'_>) -> ::tuffite::CommandResult {
                #(#branches)*
                Err(::tuffite::CommandError::UnknownCommand)
            }
        }
    };
    Ok((module, service_trait))
}

pub(super) fn generate(schema: &Schema, endpoints: &[Endpoint<'_>]) -> Result<String, String> {
    let structures = schema
        .structures
        .values()
        .map(structure)
        .collect::<Result<Vec<_>, _>>()?;
    let interfaces = schema
        .methods
        .iter()
        .map(|method| method.interface.as_str())
        .collect::<BTreeSet<_>>();
    let (modules, traits): (Vec<_>, Vec<_>) = interfaces
        .into_iter()
        .map(|interface| interface_bindings(schema, endpoints, interface))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .unzip();
    let enums = enum_bindings(schema)?;
    let syntax = syn::parse2(quote! { #enums #(#structures)* #(#modules)* #(#traits)* })
        .map_err(|error| format!("generated Mojom Rust: {error}"))?;
    Ok(format!(
        "// Generated from Mojom v0. Do not edit.\n{}",
        prettyplease::unparse(&syntax)
    ))
}

pub(super) fn enum_bindings(schema: &Schema) -> Result<TokenStream, String> {
    let mut output = TokenStream::new();
    for (name, values) in &schema.enums {
        let name = ident(name)?;
        let variants = values
            .iter()
            .map(|(name, value)| {
                let name = ident(name)?;
                let value = syn::LitInt::new(&value.to_string(), Span::call_site());
                Ok::<_, String>((name, value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let names = variants.iter().map(|(n, _)| n).collect::<Vec<_>>();
        let numbers = variants.iter().map(|(_, v)| v).collect::<Vec<_>>();
        let declarations = names
            .iter()
            .zip(numbers.iter())
            .enumerate()
            .map(|(index, (name, value))| {
                let default = if index == 0 {
                    quote!(#[default])
                } else {
                    TokenStream::new()
                };
                quote!(#default #name = #value,)
            })
            .collect::<Vec<_>>();
        output.extend(quote! {
            #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ::tuffite::__private::serde::Serialize, ::tuffite::__private::serde::Deserialize)]
            #[serde(crate = "::tuffite::__private::serde")]
            #[repr(i32)]
            pub enum #name { #(#declarations)* }
            impl ::tuffite::__private::mojom::Value for #name {
                const WIDTH: usize = 4;
                fn put(&self, w: &mut ::tuffite::__private::mojom::Encoder, at: usize, _: u8) -> Result<(), ::tuffite::__private::codec::CodecError> { w.scalar(at, &(*self as i32)) }
                fn get(r: &mut ::tuffite::__private::mojom::Decoder<'_>, at: usize, _: u8) -> Result<Self, ::tuffite::__private::codec::CodecError> {
                    match r.scalar::<i32>(at)? { #(#numbers => Ok(Self::#names),)* _ => Err(::tuffite::__private::codec::CodecError) }
                }
            }
        });
    }
    Ok(output)
}
