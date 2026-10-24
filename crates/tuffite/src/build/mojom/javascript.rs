//! JS/TS syntax templates are parsed, populated with typed AST nodes and printed by OXC.
use super::naming::lower_first;
use oxc_allocator::Allocator;
use oxc_ast::{
    AstBuilder, NONE,
    ast::{BindingPattern, Program, Statement, TSSignature, TSType, TSTypeQueryExprName},
};
use oxc_span::SPAN;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use tuffite_mojom::{Schema, Type};

mod json;
mod quote;
use quote::{Hole, Quote};

pub(super) struct Bindings {
    pub javascript: String,
    pub declarations: String,
}

fn ts_type<'a>(ast: AstBuilder<'a>, ty: &Type) -> TSType<'a> {
    match ty {
        Type::Bool => ast.ts_type_boolean_keyword(SPAN),
        Type::String => ast.ts_type_string_keyword(SPAN),
        Type::Scalar(name) if name == "u64" || name == "i64" => ast.ts_type_big_int_keyword(SPAN),
        Type::Scalar(_) => ast.ts_type_number_keyword(SPAN),
        Type::Bytes => named_type(ast, "Uint8Array"),
        Type::Named(name) | Type::Enum(name) => named_type(ast, name),
        Type::Array(inner) => ast.ts_type_array_type(SPAN, ts_type(ast, inner)),
        Type::Nullable(inner) => ast.ts_type_union_type(
            SPAN,
            ast.vec_from_array([ts_type(ast, inner), ast.ts_type_null_keyword(SPAN)]),
        ),
    }
}

fn named_type<'a>(ast: AstBuilder<'a>, name: &str) -> TSType<'a> {
    let name = ast.ts_type_name_identifier_reference(SPAN, ast.ident(name));
    ast.ts_type_type_reference(SPAN, name, NONE)
}

fn property<'a>(
    ast: AstBuilder<'a>,
    name: &str,
    ty: TSType<'a>,
    readonly: bool,
) -> TSSignature<'a> {
    let optional = matches!(ty, TSType::TSUnionType(_));
    ast.ts_signature_property_signature(
        SPAN,
        false,
        optional,
        readonly,
        ast.expression_string_literal(SPAN, ast.atom(name), None)
            .into(),
        Some(ast.ts_type_annotation(SPAN, ty)),
    )
}

fn method_type<'a>(
    ast: AstBuilder<'a>,
    schema: &Schema,
    method: &tuffite_mojom::Method,
) -> Result<TSType<'a>, String> {
    let parameters = ast.vec_from_iter(
        schema.structures[&method.request]
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                ast.formal_parameter(
                    SPAN,
                    ast.vec(),
                    ast.binding_pattern_binding_identifier(SPAN, ast.ident(&format!("arg{index}"))),
                    Some(ast.ts_type_annotation(SPAN, ts_type(ast, &field.ty))),
                    NONE,
                    schema.structures[&method.request].fields[index..]
                        .iter()
                        .all(|field| matches!(field.ty, Type::Nullable(_))),
                    None,
                    false,
                    false,
                )
            }),
    );
    let result = schema.structures[&method.response]
        .fields
        .first()
        .map_or_else(
            || ast.ts_type_void_keyword(SPAN),
            |field| ts_type(ast, &field.ty),
        );
    let mut program = Quote::new(ast)
        .bind("$Args", Hole::Parameters(parameters))
        .bind("$Result", Hole::Type(result))
        .parse(include_str!("templates/method.d.ts.in"), "method.d.ts")?;
    let Some(Statement::TSTypeAliasDeclaration(alias)) = program.body.pop() else {
        return Err("method AST template must contain a type alias".into());
    };
    Ok(alias.unbox().type_annotation)
}

fn declarations<'a>(
    ast: AstBuilder<'a>,
    schema: &Schema,
    client: &str,
    factory: &str,
) -> Result<(Program<'a>, Program<'a>), String> {
    let mut types = Quote::new(ast).parse(include_str!("templates/types.d.ts.in"), "mojom.d.ts")?;
    for (name, values) in &schema.enums {
        let members = values
            .iter()
            .map(|(name, value)| {
                Ok(ast.ts_enum_member(
                    SPAN,
                    ast.ts_enum_member_name_identifier(SPAN, ast.ident(name)),
                    Some(json::literal(ast, &serde_json::json!(value))?),
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let declaration = Quote::new(ast)
            .bind("$Name", Hole::Ident(ast.ident(name)))
            .bind("$Members", Hole::EnumMembers(ast.vec_from_iter(members)))
            .parse(include_str!("templates/enum.d.ts.in"), "enum.d.ts")?;
        types.body.extend(declaration.body);
    }
    for structure in schema.structures.values() {
        let members = ast.vec_from_iter(
            structure
                .fields
                .iter()
                .map(|field| property(ast, &field.name, ts_type(ast, &field.ty), false)),
        );
        let declaration = Quote::new(ast)
            .bind("$Name", Hole::Ident(ast.ident(&structure.name)))
            .bind("$Members", Hole::Members(members))
            .parse(include_str!("templates/struct.d.ts.in"), "struct.d.ts")?;
        types.body.extend(declaration.body);
    }
    let mut properties = ast.vec();
    let mut interfaces = ast.vec();
    for interface in schema
        .methods
        .iter()
        .map(|m| m.interface.as_str())
        .collect::<BTreeSet<_>>()
    {
        let name = lower_first(interface);
        let members = schema
            .methods
            .iter()
            .filter(|m| m.interface == interface)
            .map(|method| {
                Ok(property(
                    ast,
                    &lower_first(&method.name),
                    method_type(ast, schema, method)?,
                    true,
                ))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let declaration = Quote::new(ast)
            .bind("$Name", Hole::Ident(ast.ident(&name)))
            .bind("$Members", Hole::Members(ast.vec_from_iter(members)))
            .parse(
                include_str!("templates/interface.d.ts.in"),
                "interface.d.ts",
            )?;
        interfaces.extend(declaration.body);
        let ty = ast.ts_type_type_query(
            SPAN,
            TSTypeQueryExprName::IdentifierReference(
                ast.alloc_identifier_reference(SPAN, ast.ident(&name)),
            ),
            NONE,
        );
        properties.push(property(ast, &name, ty, true));
    }
    let mut client = Quote::new(ast)
        .bind("$Client", Hole::Ident(ast.ident(client)))
        .bind("$Factory", Hole::Ident(ast.ident(factory)))
        .bind("$Properties", Hole::Members(properties))
        .parse(include_str!("templates/client.d.ts.in"), "index.d.ts")?;
    client.body.extend(interfaces);
    Ok((types, client))
}

fn export_pattern<'a>(ast: AstBuilder<'a>, schema: &Schema) -> BindingPattern<'a> {
    let names = schema
        .methods
        .iter()
        .map(|m| lower_first(&m.interface))
        .collect::<BTreeSet<_>>();
    let properties = ast.vec_from_iter(names.iter().map(|name| {
        ast.binding_property(
            SPAN,
            ast.property_key_static_identifier(SPAN, ast.ident(name)),
            ast.binding_pattern_binding_identifier(SPAN, ast.ident(name)),
            true,
            false,
        )
    }));
    ast.binding_pattern_object_pattern(SPAN, properties, NONE)
}

pub(super) fn generate(
    schema: &Schema,
    entries: &[Value],
    types: &BTreeMap<String, Value>,
    root: &str,
) -> Result<Bindings, String> {
    let allocator = Allocator::default();
    let ast = AstBuilder::new(&allocator);
    let mut client = String::new();
    for part in root.split('.') {
        client.push_str(&part[..1].to_ascii_uppercase());
        client.push_str(&part[1..]);
    }
    client.push_str("Api");
    let factory = format!("create{client}");
    let mut facade = Quote::new(ast)
        .bind("$Factory", Hole::Ident(ast.ident(&factory)))
        .bind(
            "$Transport",
            Hole::Ident(ast.ident(if root == "tuffite" {
                "invokeFrameworkMojom"
            } else {
                "invokeBinary"
            })),
        )
        .bind(
            "$Namespace",
            Hole::Expr(ast.expression_string_literal(SPAN, ast.atom(root), None)),
        )
        .bind("$Exports", Hole::Pattern(export_pattern(ast, schema)))
        .parse(include_str!("templates/facade.js.in"), "index.js")?;
    let mut metadata = Quote::new(ast)
        .bind(
            "$Types",
            Hole::Expr(json::literal(
                ast,
                &serde_json::to_value(types).map_err(|e| e.to_string())?,
            )?),
        )
        .bind(
            "$Methods",
            Hole::Expr(json::literal(ast, &Value::Array(entries.to_vec()))?),
        )
        .parse(include_str!("templates/metadata.js.in"), "mojom.js")?;
    for (name, values) in &schema.enums {
        let mut members = serde_json::Map::new();
        for (name, value) in values {
            members.insert(name.clone(), serde_json::json!(value));
            members.insert(value.to_string(), serde_json::json!(name));
        }
        let value = serde_json::Value::Object(members);
        let declaration = Quote::new(ast)
            .bind("$Name", Hole::Ident(ast.ident(name)))
            .bind("$Value", Hole::Expr(json::literal(ast, &value)?))
            .parse(include_str!("templates/enum.js.in"), "enum.js")?;
        metadata.body.extend(declaration.body);
    }
    let (mut types, client) = declarations(ast, schema, &client, &factory)?;
    types.body.extend(client.body);
    if root == "tuffite" {
        facade
            .body
            .retain(|s| !matches!(s, Statement::ImportDeclaration(_)));
        types
            .body
            .retain(|s| !matches!(s, Statement::ImportDeclaration(_)));
        let runtime_source = include_str!("runtime.mjs");
        let mut runtime = quote::parse(ast, runtime_source, "runtime.mjs")?;
        runtime.body.extend(metadata.body);
        metadata = runtime;
        let mut runtime_types = quote::parse(ast, include_str!("runtime.d.ts"), "runtime.d.ts")?;
        runtime_types.body.extend(types.body);
        types = runtime_types;
    }
    metadata.body.extend(facade.body);
    Ok(Bindings {
        javascript: quote::emit(&metadata, "index.js")?,
        declarations: quote::emit(&types, "index.d.ts")?,
    })
}

#[cfg(test)]
mod tests;
