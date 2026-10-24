use super::*;
use std::path::Path;

#[test]
fn module_is_the_only_source_of_method_identity() {
    let sources = vec![(
        "api.mojom".into(),
        "module example.files; interface Fs { Read@0(string path) => (string result); };".into(),
    )];
    let bindings = generate(&sources, "org.example.app").unwrap();
    assert!(bindings.command_ids.contains_key("example.files.Fs.Read"));
    assert!(bindings.javascript.contains("createExampleFilesApi"));
    assert!(bindings.javascript.contains("example.files.Fs.Read"));
    let other = vec![(
        "api.mojom".into(),
        sources[0].1.replace("example.files", "other.files"),
    )];
    let other = generate(&other, "org.example.app").unwrap();
    assert_ne!(
        bindings.command_ids["example.files.Fs.Read"],
        other.command_ids["other.files.Fs.Read"]
    );
    for module in ["tuffite", "tuffite.internal"] {
        let sources = vec![(
            "api.mojom".into(),
            format!("module {module}; interface X {{ Run@0() => (); }};"),
        )];
        assert!(
            generate(&sources, "org.example.app")
                .unwrap_err()
                .contains("reserved Mojom module")
        );
    }
}

#[test]
fn playground_bindings_preserve_checked_in_sources() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../playground");
    let sources = tuffite_mojom::read_sources(&root, &["shell_api.mojom".into()]).unwrap();
    let bindings = generate(&sources, "org.tuffite.playground").unwrap();
    assert_eq!(
        bindings.rust,
        include_str!("../../../tests/fixtures/mojom_generated.rs").replace("\r\n", "\n")
    );
    assert_eq!(
        bindings.javascript,
        std::fs::read_to_string(root.join("generated/index.js"))
            .unwrap()
            .replace("\r\n", "\n")
    );
    assert_eq!(
        bindings.declarations,
        std::fs::read_to_string(root.join("generated/index.d.ts"))
            .unwrap()
            .replace("\r\n", "\n")
    );
}

#[test]
fn quotes_keywords_and_rejects_invalid_rust_identifiers_without_panicking() {
    let sources = vec![(
        "api.mojom".into(),
        "module demo; interface Commands { Run(string type) => (string match); Empty() => (); };"
            .into(),
    )];
    let bindings = generate(&sources, "demo").unwrap();
    assert!(bindings.rust.contains("pub r#type: String"));
    assert!(bindings.rust.contains("r#match: result"));
    syn::parse_file(&bindings.rust).unwrap();
    let sources = vec![(
        "api.mojom".into(),
        "module demo; interface Fn { Run() => (); };".into(),
    )];
    assert!(
        generate(&sources, "demo")
            .unwrap_err()
            .contains("Rust identifier")
    );
}

#[test]
fn rejects_unsupported_and_invalid_contracts() {
    for source in [
        "module demo; struct X { handle value; };",
        "module demo; struct X { array<int32?> value; };",
        "module demo; struct X { uint8 x@0; uint8 y@0; };",
        "module demo; [Sync] interface X {};",
        "module demo; /* never closed",
    ] {
        assert!(
            tuffite_mojom::compile(&[(PathBuf::from("test.mojom"), source.into())]).is_err(),
            "{source}"
        );
    }
    let sources = vec![(
        PathBuf::from("x.mojom"),
        "module demo; struct X { X child; }; interface Commands { Run(X value) => (); };".into(),
    )];
    assert!(generate(&sources, "demo").is_err());
}

#[test]
fn playground_imports_and_generated_sources_are_reproducible() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tuffite-mojom/tests/fixtures");
    let sources = tuffite_mojom::read_sources(&root, &[PathBuf::from("shell_api.mojom")]).unwrap();
    assert_eq!(sources.len(), 2);
    assert_eq!(sources[0].0, Path::new("mojom_data.mojom"));
    let generated = generate(&sources, "org.tuffite.playground").unwrap();
    assert_eq!(
        generated.rust,
        generate(&sources, "org.tuffite.playground").unwrap().rust
    );
    assert!(
        generated
            .command_ids
            .contains_key("playground.Commands.EchoMojom")
    );
    for path in ["../outside.mojom", "/outside.mojom", "shell_api.d.ts"] {
        assert!(tuffite_mojom::read_sources(&root, &[PathBuf::from(path)]).is_err());
    }
}

#[test]
fn rejects_unknown_types_and_names_that_normalize_to_collisions() {
    for source in [
        "module demo; interface Commands { Run(Unknown value) => (); };",
        "module demo; struct X { int8 someValue; int8 some_value; }; interface Commands { Run(X value) => (); };",
        "module demo; interface Commands { Run@0() => (); run@1() => (); };",
        "module demo; interface Commands { Then() => (); };",
        "module demo; interface Commands { FooBar() => (); Foo_Bar() => (); };",
        "module demo; interface Commands { __tuffite_dispatch() => (); };",
        "module demo; struct Commands {}; interface Commands { Run() => (); };",
        "module demo; /*",
    ] {
        let sources = vec![(PathBuf::from("api.mojom"), source.to_owned())];
        assert!(generate(&sources, "demo").is_err(), "{source}");
    }
}

#[test]
fn method_named_like_its_payload_keeps_the_types_distinct() {
    let sources = vec![(
        PathBuf::from("api.mojom"),
        include_str!("../../../tests/fixtures/mojom_collision.mojom").to_owned(),
    )];
    let bindings = generate(&sources, "example").unwrap();
    assert_eq!(
        bindings.rust,
        include_str!("../../../tests/fixtures/mojom_collision.rs").replace("\r\n", "\n")
    );
}
