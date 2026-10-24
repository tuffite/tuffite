use super::*;
use oxc_ast::ast::{ObjectProperty, PropertyKey, StringLiteral};
use oxc_ast_visit::{Visit, walk};

#[derive(Default)]
struct Literals {
    strings: Vec<String>,
    prototype_computed: bool,
}
impl<'a> Visit<'a> for Literals {
    fn visit_string_literal(&mut self, node: &StringLiteral<'a>) {
        self.strings.push(node.value.to_string());
    }
    fn visit_object_property(&mut self, node: &ObjectProperty<'a>) {
        if let PropertyKey::StringLiteral(key) = &node.key
            && key.value == "__proto__"
        {
            self.prototype_computed = node.computed;
        }
        walk::walk_object_property(self, node);
    }
}

#[test]
fn typed_holes_reject_missing_unused_duplicate_and_wrong_kind_bindings() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    assert!(
        Quote::new(ast)
            .parse("const value = $Missing;", "index.js")
            .is_err()
    );
    assert!(
        Quote::new(ast)
            .bind("$Unused", Hole::Ident(ast.ident("valid")))
            .parse("const value = 1;", "index.js")
            .is_err()
    );
    assert!(
        Quote::new(ast)
            .bind("$Value", Hole::Type(ast.ts_type_string_keyword(SPAN)))
            .parse("const value = $Value;", "index.js")
            .is_err()
    );
    assert!(
        Quote::new(ast)
            .bind("$Value", Hole::Expr(ast.expression_null_literal(SPAN)))
            .parse("const $Value = 1;", "index.js")
            .is_err()
    );
    assert!(
        Quote::new(ast)
            .bind("$Name", Hole::Ident(ast.ident("a; injected()")))
            .parse("const $Name = 1;", "index.js")
            .is_err()
    );
    assert!(
        Quote::new(ast)
            .bind("$Name", Hole::Ident(ast.ident("a")))
            .bind("$Name", Hole::Ident(ast.ident("b")))
            .parse("const $Name = 1;", "index.js")
            .is_err()
    );
    let program = Quote::new(ast)
        .bind("$Name", Hole::Ident(ast.ident("class")))
        .parse("const $Name = 1;", "index.js")
        .unwrap();
    assert!(quote::emit(&program, "index.js").is_err());
}

#[test]
fn malformed_type_and_rest_parameter_holes_are_rejected() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    assert!(
        Quote::new(ast)
            .bind("$Result", Hole::Type(ast.ts_type_string_keyword(SPAN)))
            .parse("type X = $Result<string>;", "index.d.ts")
            .is_err()
    );
    assert!(
        Quote::new(ast)
            .bind("$Args", Hole::Parameters(ast.vec()))
            .parse(
                "declare function f($Args: never, ...$Missing: unknown[]): void;",
                "index.d.ts"
            )
            .is_err()
    );
}

#[test]
fn metadata_is_inserted_as_data_and_survives_codegen_round_trip() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let text = "'\"`\\\n\0你好\u{2028}</script>; globalThis.injected = true; $Value";
    let value = serde_json::json!({"__proto__": {"text":text}, "values":[text, true, null]});
    let program = Quote::new(ast)
        .bind("$Value", Hole::Expr(json::literal(ast, &value).unwrap()))
        .parse("export const data = $Value;", "index.js")
        .unwrap();
    let code = quote::emit(&program, "index.js").unwrap();
    let parsed = quote::parse(ast, ast.allocator.alloc_str(&code), "index.js").unwrap();
    assert_eq!(parsed.body.len(), 1);
    let mut literals = Literals::default();
    literals.visit_program(&parsed);
    assert!(literals.prototype_computed);
    assert_eq!(
        literals
            .strings
            .iter()
            .filter(|value| value.as_str() == text)
            .count(),
        2
    );
    assert!(!code.contains("$Value;"));
}

#[test]
fn list_holes_can_be_empty_and_inserted_between_static_nodes() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let program = Quote::new(ast).bind("$Members", Hole::Members(ast.vec())).bind("$Args", Hole::Parameters(ast.vec()))
        .parse("interface X { before: string; $Members: unknown; after: boolean; } type F = (first: string, $Args: never, last: boolean) => void;", "index.d.ts").unwrap();
    let code = quote::emit(&program, "index.d.ts").unwrap();
    assert!(code.contains("before: string"));
    assert!(code.contains("after: boolean"));
    assert!(code.contains("first: string, last: boolean"));
    assert!(!code.contains('$'));
}

#[test]
fn identifiers_and_inserted_nodes_are_not_recursively_substituted() {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let program = Quote::new(ast)
        .bind("$Name", Hole::Ident(ast.ident("$Literal")))
        .bind(
            "$Value",
            Hole::Expr(ast.expression_string_literal(SPAN, "$Name", None)),
        )
        .parse("const $Name = $Value;", "index.js")
        .unwrap();
    let code = quote::emit(&program, "index.js").unwrap();
    assert!(code.contains("const $Literal = '$Name'"));
}
