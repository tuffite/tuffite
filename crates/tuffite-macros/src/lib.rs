//! Procedural macros for Tuffite applications.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{FnArg, ItemFn, ReturnType, Type, parse_macro_input};

#[derive(Default)]
struct MainArgs {
    target: Option<(bool, syn::TypePath)>,
    utilities: Vec<syn::TypePath>,
}

impl syn::parse::Parse for MainArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let mut result = Self::default();
        let mut seen_utilities = false;
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            if (key == "application" || key == "browser") && result.target.is_none() {
                input.parse::<syn::Token![=]>()?;
                result.target = Some((key == "application", input.parse()?));
            } else if key == "utilities" && !seen_utilities {
                seen_utilities = true;
                let content;
                syn::parenthesized!(content in input);
                result.utilities = content
                    .parse_terminated(syn::TypePath::parse, syn::Token![,])?
                    .into_iter()
                    .collect();
            } else {
                return Err(syn::Error::new_spanned(
                    key,
                    "expected optional application/browser type and utilities(Type, ...)",
                ));
            }
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
        }
        Ok(result)
    }
}

/// Declares the logical application main and its explicit process implementations.
///
/// The zero-argument factory returns `Embedding<Application>`. The macro emits
/// the cached native export directly; process setup runs later, before `ContentMain`.
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
            "a Tuffite main must be a synchronous, safe, non-generic, zero-argument factory returning Embedding<Application>",
        ));
    }
    if args.target.is_some() || !args.utilities.is_empty() {
        return Err(syn::Error::new_spanned(
            signature,
            "select implementations in Embedding, not main attributes",
        ));
    }
    let output = &signature.output;
    let body = &function.block;
    let attrs = &function.attrs;
    Ok(quote! {
        #(#attrs)*
        #[doc(hidden)]
        #[unsafe(no_mangle)]
        pub extern "C" fn tuffite_get_application() -> *const ::tuffite::__private::Application {
            struct SharedApplication(::tuffite::__private::Application);
            // SAFETY: The frozen table is immutable and has library lifetime.
            unsafe impl Send for SharedApplication {}
            // SAFETY: The frozen table is read concurrently and owns no mutable references.
            unsafe impl Sync for SharedApplication {}
            static APPLICATION: ::std::sync::OnceLock<Option<SharedApplication>> = ::std::sync::OnceLock::new();
            APPLICATION.get_or_init(|| {
                match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
                    let factory = || #output #body;
                    factory().into_application()
                })) {
                    Ok(Ok(table)) => Some(SharedApplication(table)),
                    Ok(Err(error)) => { eprintln!("Tuffite embedding failed: {error}"); None }
                    Err(_) => { eprintln!("Tuffite embedding factory panicked"); None }
                }
            }).as_ref().map_or(::std::ptr::null(), |table| ::std::ptr::from_ref(&table.0))
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
            type Target = #browser_type;

            const COMMAND: ::tuffite::CommandDescriptor = <#endpoint>::COMMAND;

            fn dispatch(
                browser: &mut Self::Target,
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

    use super::{MainArgs, expand_command, expand_main};

    #[test]
    fn main_exports_embedding_factory_without_running_setup() {
        let function = parse_quote!(
            fn main() -> tuffite::Embedding<App> {
                tuffite::Embedding::new()
                    .browser::<Profile>()
                    .utility::<Worker>()
            }
        );
        let output = expand_main(&MainArgs::default(), &function).unwrap();
        syn::parse2::<syn::File>(output.clone()).unwrap();
        let output = output.to_string();
        assert!(output.contains("tuffite_get_application"));
        assert!(!output.contains("fn main"));
        assert!(output.contains("into_application ()"));
        assert!(!output.contains("setup"));
        assert!(output.contains("catch_unwind"));
    }
    #[test]
    fn main_rejects_async_and_old_implementation_attributes() {
        let function = parse_quote!(
            async fn main() -> tuffite::Embedding<App> {
                todo!()
            }
        );
        assert!(expand_main(&MainArgs::default(), &function).is_err());
        let function = parse_quote!(
            fn main() -> tuffite::Embedding<App> {
                todo!()
            }
        );
        let args = syn::parse_str::<MainArgs>("utilities(Worker)").unwrap();
        assert!(expand_main(&args, &function).is_err());
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
        assert!(output.contains("type Target = Demo"));
        assert!(output.contains("__tuffite_dispatch"));
        assert!(output.contains("Self :: __tuffite_invoke"));
    }
}
