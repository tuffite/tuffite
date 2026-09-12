//! Procedural macros for Tuffite applications.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{FnArg, ItemFn, ReturnType, Type, parse_macro_input};

struct MainArgs {
    browser: syn::TypePath,
    utilities: Vec<syn::TypePath>,
}

impl syn::parse::Parse for MainArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let key: syn::Ident = input.parse()?;
        if key != "browser" {
            return Err(syn::Error::new_spanned(
                key,
                "expected browser = BrowserType",
            ));
        }
        input.parse::<syn::Token![=]>()?;
        let browser = input.parse()?;
        let mut utilities = Vec::new();
        if !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            if !input.is_empty() {
                let key: syn::Ident = input.parse()?;
                if key != "utilities" {
                    return Err(syn::Error::new_spanned(
                        key,
                        "expected utilities(Type, ...)",
                    ));
                }
                let content;
                syn::parenthesized!(content in input);
                utilities = content
                    .parse_terminated(syn::TypePath::parse, syn::Token![,])?
                    .into_iter()
                    .collect();
                if !input.is_empty() {
                    input.parse::<syn::Token![,]>()?;
                }
            }
        }
        Ok(Self { browser, utilities })
    }
}

/// Declares the logical application main and its explicit process implementations.
///
/// The zero-argument factory returns `app::Builder<Browser>`. It assembles static
/// definitions in each process; process startup hooks run before `ContentMain`.
#[proc_macro_attribute]
pub fn main(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(arguments as MainArgs);
    let function = parse_macro_input!(item as ItemFn);
    match expand_main(&args, &function) {
        Ok(output) => output.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_main(args: &MainArgs, function: &ItemFn) -> syn::Result<TokenStream2> {
    let signature = &function.sig;
    if signature.asyncness.is_some()
        || signature.constness.is_some()
        || matches!(signature.safety, syn::Safety::Unsafe(_))
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || !signature.inputs.is_empty()
        || matches!(signature.output, ReturnType::Default)
    {
        return Err(syn::Error::new_spanned(
            signature,
            "a Tuffite main must be a synchronous, safe, non-generic, zero-argument factory returning tuffite::app::Builder<Browser>",
        ));
    }
    let name = &signature.ident;
    let browser = &args.browser;
    let utilities = &args.utilities;
    Ok(quote! {
        #function

        const _: &str = <#browser as ::tuffite::__private::BrowserDefinition>::TYPE_NAME;

        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn tuffite_get_application(
        ) -> *const ::tuffite::__private::Application {
            const UTILITIES: &[::tuffite::UtilityServiceRegistration] = &[
                #(<#utilities as ::tuffite::__private::UtilityDefinition>::REGISTRATION,)*
            ];
            const _: () = ::tuffite::__private::validate_utility_names(UTILITIES);
            static APPLICATION: ::std::sync::OnceLock<
                ::std::option::Option<::tuffite::__private::Application>,
            > = ::std::sync::OnceLock::new();
            APPLICATION.get_or_init(|| {
                match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    let builder: ::tuffite::app::Builder<#browser> = #name();
                    builder.into_application(UTILITIES)
                })) {
                    Ok(Ok(table)) => Some(table),
                    Ok(Err(error)) => {
                        ::std::eprintln!("Tuffite main registration failed: {error}");
                        None
                    }
                    Err(_) => {
                        ::std::eprintln!("Tuffite main factory panicked");
                        None
                    }
                }
            }).as_ref().map_or(::std::ptr::null(), ::std::ptr::from_ref)
        }
    })
}

/// Declares the browser backend selected by the main attribute.
#[proc_macro_attribute]
pub fn browser(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let implementation = parse_macro_input!(item as syn::ItemImpl);
    if !arguments.is_empty() {
        return syn::Error::new_spanned(&implementation, "tuffite::browser takes no arguments")
            .into_compile_error()
            .into();
    }
    match expand_browser(&implementation) {
        Ok(output) => output.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_browser(implementation: &syn::ItemImpl) -> syn::Result<proc_macro2::TokenStream> {
    if !implementation.generics.params.is_empty()
        || implementation.generics.where_clause.is_some()
        || !implementation.trait_.as_ref().is_some_and(|(path, _)| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "Browser")
        })
    {
        return Err(syn::Error::new_spanned(
            implementation,
            "tuffite::browser requires a concrete impl tuffite::Browser for Type",
        ));
    }
    let ty = &implementation.self_ty;
    Ok(quote! {
        #implementation
        impl ::tuffite::__private::BrowserDefinition for #ty {
            const TYPE_NAME: &'static str = stringify!(#ty);
        }
    })
}

/// Declares a named Rust or V8 utility by its implementation trait.
#[proc_macro_attribute]
pub fn utility(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let name = parse_macro_input!(arguments as syn::MetaNameValue);
    let implementation = parse_macro_input!(item as syn::ItemImpl);
    match expand_utility(&name, &implementation) {
        Ok(output) => output.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_utility(
    name: &syn::MetaNameValue,
    implementation: &syn::ItemImpl,
) -> syn::Result<TokenStream2> {
    if !name.path.is_ident("name") {
        return Err(syn::Error::new_spanned(
            name,
            "expected name = \"service_name\"",
        ));
    }
    let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Str(value),
        ..
    }) = &name.value
    else {
        return Err(syn::Error::new_spanned(
            name,
            "utility name must be a string literal",
        ));
    };
    let bytes = value.value().into_bytes();
    if bytes.is_empty()
        || bytes.len() > 128
        || bytes.iter().enumerate().any(|(i, b)| {
            !(b.is_ascii_alphabetic() || matches!(b, b'_' | b'$') || (i > 0 && b.is_ascii_digit()))
        })
    {
        return Err(syn::Error::new_spanned(
            value,
            "invalid utility service name",
        ));
    }
    if !implementation.generics.params.is_empty()
        || implementation.generics.where_clause.is_some()
        || !implementation.trait_.as_ref().is_some_and(|(path, _)| {
            path.segments
                .last()
                .is_some_and(|segment| segment.ident == "Utility" || segment.ident == "V8")
        })
    {
        return Err(syn::Error::new_spanned(
            implementation,
            "tuffite::utility requires a concrete impl tuffite::utility::Utility or V8 for Type",
        ));
    }
    let ty = &implementation.self_ty;
    let native_v8 = implementation.trait_.as_ref().is_some_and(|(path, _)| {
        path.segments
            .last()
            .is_some_and(|segment| segment.ident == "V8")
    });
    let registration = if native_v8 {
        quote!(::tuffite::UtilityServiceRegistration::v8::<Self>(#value))
    } else {
        quote!(::tuffite::UtilityServiceRegistration::new::<Self>(#value))
    };
    Ok(quote! {
        #implementation
        impl ::tuffite::__private::UtilityDefinition for #ty {
            const REGISTRATION: ::tuffite::UtilityServiceRegistration =
                #registration;
        }
    })
}

/// Marks one synchronous Rust function as a generated `ShellAPI` handler.
///
/// Pass the generated endpoint type; the macro resolves its `COMMAND`
/// descriptor automatically. The TypeScript contract remains authoritative,
/// so Rust checks browser, parameter, result, and error types at compile
/// time.
#[proc_macro_attribute]
pub fn command(arguments: TokenStream, item: TokenStream) -> TokenStream {
    let endpoint = parse_macro_input!(arguments as syn::TypePath);
    let function = parse_macro_input!(item as ItemFn);
    match expand_command(&endpoint, &function) {
        Ok(output) => output.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_command(endpoint: &syn::TypePath, function: &ItemFn) -> syn::Result<TokenStream2> {
    let signature = &function.sig;
    if signature.asyncness.is_some()
        || signature.constness.is_some()
        || matches!(signature.safety, syn::Safety::Unsafe(_))
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
    {
        return Err(syn::Error::new_spanned(
            signature,
            "a Tuffite command must be a synchronous, safe, non-generic Rust function",
        ));
    }
    let Some(FnArg::Typed(browser)) = signature.inputs.first() else {
        return Err(syn::Error::new_spanned(
            &signature.inputs,
            "a Tuffite command's first parameter must be `&mut BrowserType`",
        ));
    };
    let Type::Reference(browser_reference) = browser.ty.as_ref() else {
        return Err(syn::Error::new_spanned(
            &browser.ty,
            "a Tuffite command's first parameter must be `&mut BrowserType`",
        ));
    };
    if browser_reference.mutability.is_none() {
        return Err(syn::Error::new_spanned(
            &browser.ty,
            "a Tuffite command's first parameter must be `&mut BrowserType`",
        ));
    }
    let browser_type = &browser_reference.elem;
    let attributes = &function.attrs;
    let visibility = &function.vis;
    let name = &signature.ident;
    let inputs = &signature.inputs;
    let output = &signature.output;
    let block = &function.block;
    Ok(quote! {
        #(#attributes)*
        #[allow(non_camel_case_types)]
        #visibility struct #name;

        impl #name {
            #[doc(hidden)]
            #[allow(clippy::unnecessary_wraps)]
            pub(crate) fn __tuffite_invoke(#inputs) #output #block
        }

        impl ::tuffite::__private::CommandHandler for #name {
            type Browser = #browser_type;

            const COMMAND: ::tuffite::CommandDescriptor = <#endpoint>::COMMAND;

            fn dispatch(
                browser: &mut Self::Browser,
                command: ::tuffite::Command<'_>,
            ) -> ::tuffite::CommandResult {
                <#endpoint>::__tuffite_dispatch(
                    command,
                    browser,
                    Self::__tuffite_invoke,
                )
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use syn::{ItemFn, TypePath, parse_quote};

    use super::{MainArgs, expand_command, expand_main, expand_utility};

    #[test]
    fn main_exports_explicit_browser_and_utility_definitions() {
        let application: ItemFn = parse_quote!(
            fn builder() -> tuffite::app::Builder<Demo> {
                tuffite::app::Builder::new()
            }
        );
        let args =
            syn::parse_str::<MainArgs>("browser = Demo, utilities(Indexer, Parser),").unwrap();
        let output = expand_main(&args, &application).unwrap().to_string();
        assert!(output.contains("fn builder"));
        assert!(output.contains("tuffite_get_application"));
        assert!(output.contains("builder ()"));
        assert!(output.contains("into_application (UTILITIES)"));
        assert!(output.contains("Indexer"));
        assert!(output.contains("validate_utility_names"));
        assert!(output.contains("catch_unwind"));
    }

    #[test]
    fn main_rejects_invalid_arguments_and_async_factories() {
        assert!(syn::parse_str::<MainArgs>("utilities(Worker)").is_err());
        assert!(syn::parse_str::<MainArgs>("browser = Demo, unknown(Worker)").is_err());
        let args = syn::parse_str::<MainArgs>("browser = Demo").unwrap();
        let function = parse_quote!(
            async fn main() -> Builder<Demo> {
                todo!()
            }
        );
        assert!(expand_main(&args, &function).is_err());
    }

    #[test]
    fn browser_requires_a_concrete_browser_implementation() {
        let implementation = syn::parse_quote!(impl tuffite::Browser for Backend {});
        assert!(
            super::expand_browser(&implementation)
                .unwrap()
                .to_string()
                .contains("BrowserDefinition")
        );
        for source in [
            "impl Backend {}",
            "impl Utility for Backend {}",
            "impl<T> Browser for Backend<T> {}",
            "impl Browser for Backend where Backend: Send {}",
        ] {
            let implementation = syn::parse_str(source).unwrap();
            assert!(super::expand_browser(&implementation).is_err(), "{source}");
        }
    }

    #[test]
    fn utility_validates_name_and_implementation() {
        let implementation = parse_quote!(
            impl tuffite::Utility for Worker {}
        );
        let name = parse_quote!(name = "indexer");
        let output = expand_utility(&name, &implementation).unwrap().to_string();
        assert!(output.contains("UtilityDefinition for Worker"));
        assert!(output.contains("indexer"));
        let invalid = parse_quote!(name = "bad-name");
        assert!(expand_utility(&invalid, &implementation).is_err());
        let invalid = parse_quote!(runtime = "v8");
        assert!(expand_utility(&invalid, &implementation).is_err());
    }

    #[test]
    fn v8_utility_selects_backend_from_its_trait() {
        let name = parse_quote!(name = "parser");
        let implementation = parse_quote!(impl tuffite::utility::V8 for Parser {});
        let output = expand_utility(&name, &implementation).unwrap().to_string();
        assert!(output.contains("UtilityServiceRegistration :: v8 :: < Self >"));
        for source in [
            "impl Parser {}",
            "impl Other for Parser {}",
            "impl<T> V8 for Parser<T> {}",
        ] {
            assert!(expand_utility(&name, &syn::parse_str(source).unwrap()).is_err());
        }
        assert!(
            syn::parse_str::<syn::MetaNameValue>("name = \"parser\", v8(configuration)").is_err()
        );
    }

    #[test]
    fn command_becomes_a_statically_invokable_handler() {
        let command: ItemFn = parse_quote!(
            fn ping(app: &mut Demo, message: String) -> Result<String, Error> {
                app.ping(message)
            }
        );
        let endpoint: TypePath = parse_quote!(shell_api::commands::Ping);
        let output = expand_command(&endpoint, &command).unwrap().to_string();
        assert!(output.contains("struct ping"));
        assert!(output.contains("fn __tuffite_invoke"));
        assert!(output.contains("app : & mut Demo"));
        assert!(output.contains("type Browser = Demo"));
        assert!(output.contains("__tuffite_dispatch"));
        assert!(output.contains("Self :: __tuffite_invoke"));
    }
}
