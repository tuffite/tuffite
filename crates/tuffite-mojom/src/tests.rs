use super::*;
use crate::parser::parse;
use crate::schema::layout;
use std::path::{Path, PathBuf};
#[test]
fn packs_holes_and_boolean_bits_like_mojom() {
    let mut schema = parse("module demo; struct Data { uint8 small@0; uint64 large@1; bool a@2; bool b@3; uint16 medium@4; }; ").unwrap();
    let data = schema.structures.get_mut("Data").unwrap();
    layout(data);
    assert_eq!(data.size, 24);
    assert_eq!(
        data.fields
            .iter()
            .map(|field| (field.offset, field.bit))
            .collect::<Vec<_>>(),
        vec![(8, 0), (16, 0), (9, 0), (9, 1), (10, 0)]
    );
}

#[test]
fn compiles_data_only_contracts_without_sdk_naming_rules() {
    let schema = compile(&[(
        "data.mojom".into(),
        "module sample.data; struct Data { int8 self; int8 someValue; int8 some_value; };".into(),
    )])
    .unwrap();
    assert_eq!(schema.module, "sample.data");
    assert_eq!(schema.structures["Data"].size, 16);
    assert!(schema.methods.is_empty());
}

#[test]
fn imports_are_confined_and_layout_is_resolved() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let sources = read_sources(&root, &[PathBuf::from("shell_api.mojom")]).unwrap();
    assert_eq!(sources.len(), 2);
    assert_eq!(sources[0].0, Path::new("mojom_data.mojom"));
    let schema = compile(&sources).unwrap();
    assert_eq!(schema.module, "playground");
    assert_eq!(schema.methods.len(), 4);
    assert!(schema.structures["MojomSample"].size > 8);
    for path in ["../outside.mojom", "/outside.mojom", "shell_api.d.ts"] {
        assert!(read_sources(&root, &[path.into()]).is_err());
    }
}

#[test]
fn rejects_invalid_types_modules_and_duplicate_endpoints() {
    for source in [
        "module demo; struct X { handle value; };",
        "module demo; struct X { array<int32?> value; };",
        "module demo; struct X { uint8 x@0; uint8 y@0; };",
        "module demo; [Sync] interface X {};",
        "module demo; /* never closed",
        "module demo; struct X { X child; };",
        "module demo; interface X { Run(Unknown value) => (); };",
    ] {
        assert!(
            compile(&[("api.mojom".into(), source.into())]).is_err(),
            "{source}"
        );
    }
    for sources in [
        vec![
            ("a.mojom".into(), "module a;".into()),
            ("b.mojom".into(), "module b;".into()),
        ],
        vec![
            (
                "a.mojom".into(),
                "module a; interface X { Run@0() => (); };".into(),
            ),
            (
                "b.mojom".into(),
                "module a; interface X { Other@0() => (); };".into(),
            ),
        ],
        vec![],
    ] {
        assert!(compile(&sources).is_err());
    }
}

#[test]
fn diagnostics_preserve_utf8_offsets_and_eof_spans() {
    use miette::Diagnostic;
    for (source, token) in [
        ("module demo; // 中文\nstruct Data { string value }", "}"),
        ("module demo; /* 中文", "/* 中文"),
        ("module demo; import \"missing.mojom", "\"missing.mojom"),
        ("module demo; 界", "界"),
        ("module demo", ""),
    ] {
        let error = compile(&[("bad.mojom".into(), source.into())]).unwrap_err();
        let label = error.labels().unwrap().next().unwrap();
        let offset = if token.is_empty() {
            source.len()
        } else {
            source.find(token).unwrap()
        };
        assert_eq!(label.offset(), offset, "{error}");
        assert_eq!(label.len(), token.len(), "{error}");
        let snippet = error
            .source_code()
            .unwrap()
            .read_span(label.inner(), 0, 0)
            .unwrap();
        assert_eq!(snippet.name(), Some("bad.mojom"));
    }
}

#[test]
fn missing_import_diagnostic_points_to_the_importing_source() {
    use miette::Diagnostic;
    let root =
        std::env::temp_dir().join(format!("tuffite-import-diagnostic-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let source = "module demo;\nimport \"missing.mojom\";";
    std::fs::write(root.join("main.mojom"), source).unwrap();
    let error = read_sources(&root, &["main.mojom".into()]).unwrap_err();
    let label = error.labels().unwrap().next().unwrap();
    assert_eq!(
        &source[label.offset()..label.offset() + label.len()],
        "\"missing.mojom\""
    );
    let snippet = error
        .source_code()
        .unwrap()
        .read_span(label.inner(), 0, 0)
        .unwrap();
    assert_eq!(snippet.name(), Some("main.mojom"));
    assert!(error.help().unwrap().to_string().contains("source root"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn unknown_type_diagnostic_keeps_the_referenced_files_location() {
    use miette::Diagnostic;
    let dependency = "module demo; struct Z { Missing field; };";
    let error = compile(&[
        (
            "main.mojom".into(),
            "module demo; struct A { Z nested; };".into(),
        ),
        ("dependency.mojom".into(), dependency.into()),
    ])
    .unwrap_err();
    assert!(error.message().contains("unknown Mojom type Missing"));
    let label = error.labels().unwrap().next().unwrap();
    assert_eq!(
        &dependency[label.offset()..label.offset() + label.len()],
        "Missing"
    );
    let snippet = error
        .source_code()
        .unwrap()
        .read_span(label.inner(), 0, 0)
        .unwrap();
    assert_eq!(snippet.name(), Some("dependency.mojom"));
}
