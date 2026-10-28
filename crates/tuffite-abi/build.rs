//! Optionally generate Rust declarations from the public C ABI for the Cargo target.

fn main() {
    #[cfg(feature = "bindgen")]
    bindings::generate();
}

#[cfg(feature = "bindgen")]
mod bindings {
    use std::{collections::HashSet, path::PathBuf};
    use syn::visit_mut::{self, VisitMut};

    #[derive(Debug)]
    struct Names;
    impl bindgen::callbacks::ParseCallbacks for Names {
        fn item_name(&self, info: bindgen::callbacks::ItemInfo<'_>) -> Option<String> {
            let name = info.name.strip_prefix("tuffite_")?;
            Some(match name {
                "abi_version" => "Version".into(),
                "abi_header" => "Header".into(),
                "byte_slice_t" => "ByteSlice".into(),
                _ => name
                    .trim_end_matches("_t")
                    .split('_')
                    .map(|part| {
                        let mut chars = part.chars();
                        chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
                    })
                    .collect(),
            })
        }
    }

    // Keep callback aliases callable, with nullability expressed at each use site.
    struct NullableCallbacks(HashSet<String>);
    impl VisitMut for NullableCallbacks {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            if let syn::Type::Path(path) = ty
                && path.qself.is_none()
                && path.path.segments.len() == 1
                && self.0.contains(&path.path.segments[0].ident.to_string())
            {
                *ty = syn::parse_quote!(::std::option::Option<#ty>);
                return;
            }
            visit_mut::visit_type_mut(self, ty);
        }
    }

    pub fn generate() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        // Clang expects a regular path rather than Windows' extended prefix.
        let root = PathBuf::from(
            root.to_str()
                .unwrap()
                .trim_start_matches(r"\\?\")
                .replace('\\', "/"),
        );
        let output_dir =
            PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
        // Public headers use source-absolute tuffite/... includes. Cargo Git checkouts
        // are named after the revision, so build an include tree independent of the
        // checkout's directory name (also usable on Windows without symlinks).
        let include_dir = output_dir.join("include");
        let headers = root.join("framework/public/c_api");
        let include_headers = include_dir.join("tuffite/framework/public/c_api");
        std::fs::create_dir_all(&include_headers).expect("create ABI include directory");
        println!("cargo:rerun-if-changed={}", headers.display());
        for entry in std::fs::read_dir(&headers).expect("read ABI headers") {
            let entry = entry.expect("read ABI header entry");
            if entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "h")
            {
                println!("cargo:rerun-if-changed={}", entry.path().display());
                std::fs::copy(entry.path(), include_headers.join(entry.file_name()))
                    .expect("copy ABI header");
            }
        }
        let target = std::env::var("TARGET").expect("Cargo target triple");
        let bindings = bindgen::Builder::default()
            .header(root.join("crates/tuffite-abi/wrapper.h").to_str().unwrap())
            .clang_arg(format!("-I{}", include_dir.display()))
            .clang_arg(format!("--target={target}"))
            .clang_arg("-x")
            .clang_arg("c")
            .clang_arg("-ffreestanding")
            .allowlist_type("tuffite_.*")
            .allowlist_var("TUFFITE_.*")
            .allowlist_function("^$")
            .blocklist_item("TUFFITE_FFI_ABI_.*")
            .new_type_alias("Status|RuntimeEventKind|TaskPriority")
            .impl_debug(true)
            .derive_eq(true)
            .derive_partialeq(true)
            .layout_tests(false)
            .generate_comments(true)
            .parse_callbacks(Box::new(Names))
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .generate()
            .expect("generate public C ABI bindings")
            .to_string();
        let mut file = syn::parse_file(&bindings).unwrap();
        let mut callbacks = HashSet::new();
        for item in &mut file.items {
            if let syn::Item::Type(alias) = item
                && alias.ident.to_string().ends_with("Fn")
                && let syn::Type::Path(path) = &*alias.ty
                && let Some(segment) = path.path.segments.last()
                && segment.ident == "Option"
                && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                && let Some(syn::GenericArgument::Type(inner @ syn::Type::BareFn(_))) =
                    args.args.first()
            {
                *alias.ty = inner.clone();
                callbacks.insert(alias.ident.to_string());
            }
        }
        NullableCallbacks(callbacks).visit_file_mut(&mut file);
        let output = output_dir.join("bindings.rs");
        std::fs::write(output, prettyplease::unparse(&file)).expect("write ABI bindings");
    }
}
