use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
#[cfg(test)]
use std::path::Path;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Declaration, Expression, FormalParameters, Function, PropertyKey, Statement,
    TSEnumDeclaration, TSEnumMemberName, TSInterfaceDeclaration, TSModuleDeclaration,
    TSModuleDeclarationBody, TSModuleDeclarationKind, TSModuleDeclarationName, TSSignature,
    TSType as AstType, TSTypeName, VariableDeclaration, VariableDeclarationKind,
};
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use oxc_syntax::operator::UnaryOperator;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use serde_json::{Value, json};
use syn::{LitInt, LitStr};

use super::{Facade, GeneratedBindings, command_id, is_identifier};

#[derive(Clone, Debug, PartialEq)]
enum Type {
    Bool,
    Number,
    BigInt,
    String,
    Bytes,
    Date,
    Array(Box<Type>),
    Optional(Box<Type>),
    Named(String),
    Object(Vec<Member>),
}

#[derive(Clone, Debug, PartialEq)]
struct Member {
    name: String,
    ty: Type,
}

#[derive(Clone, Debug, PartialEq)]
struct Field {
    name: String,
    ty: Type,
}

#[derive(Clone, Debug, PartialEq)]
enum NamespaceMember {
    Namespace(Namespace),
    Definition {
        name: String,
        definition: Definition,
    },
    Constant(Constant),
    Function {
        name: String,
        parameters: Vec<Field>,
        result: Type,
    },
}

#[derive(Clone, Debug, PartialEq)]
struct Namespace {
    name: String,
    members: Vec<NamespaceMember>,
}

#[derive(Clone, Debug, PartialEq)]
enum Definition {
    Interface(Vec<Member>),
    Alias(Type),
    Enum(EnumDefinition),
}

#[derive(Clone, Debug, PartialEq)]
struct EnumDefinition {
    representation: EnumRepresentation,
    members: Vec<EnumMember>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnumRepresentation {
    String,
    Integer,
}

#[derive(Clone, Debug, PartialEq)]
struct EnumMember {
    name: String,
    value: ConstantValue,
}

#[derive(Clone, Debug, PartialEq)]
struct Constant {
    name: String,
    value: ConstantValue,
}

#[derive(Clone, Debug, PartialEq)]
enum ConstantValue {
    Bool(bool),
    Number(f64),
    BigInt(i64),
    String(String),
}

struct Schema {
    definitions: BTreeMap<String, Definition>,
    namespaces: BTreeMap<String, Namespace>,
}

fn merge_namespace(destination: &mut Namespace, source: Namespace) {
    for member in source.members {
        if let NamespaceMember::Namespace(source_child) = member {
            if let Some(NamespaceMember::Namespace(destination_child)) =
                destination.members.iter_mut().find(|candidate| {
                    matches!(candidate, NamespaceMember::Namespace(child) if child.name == source_child.name)
                })
            {
                merge_namespace(destination_child, source_child);
            } else {
                destination
                    .members
                    .push(NamespaceMember::Namespace(source_child));
            }
        } else {
            destination.members.push(member);
        }
    }
}

fn merge_schema(destination: &mut Schema, source: Schema) -> Result<(), String> {
    for (name, definition) in source.definitions {
        if destination
            .definitions
            .insert(name.clone(), definition)
            .is_some()
        {
            return Err(format!("duplicate TypeScript type {name}"));
        }
    }
    for (name, namespace) in source.namespaces {
        if let Some(existing) = destination.namespaces.get_mut(&name) {
            merge_namespace(existing, namespace);
        } else {
            destination.namespaces.insert(name, namespace);
        }
    }
    Ok(())
}

fn parse_schema(source: &str) -> Result<Schema, String> {
    let allocator = Allocator::default();
    let parsed = OxcParser::new(
        &allocator,
        source,
        SourceType::default()
            .with_typescript(true)
            .with_typescript_definition(true),
    )
    .parse();
    if !parsed.errors.is_empty() {
        return Err(format!("invalid TypeScript: {}", parsed.errors[0]));
    }

    let mut schema = Schema {
        definitions: BTreeMap::new(),
        namespaces: BTreeMap::new(),
    };
    for statement in &parsed.program.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            return Err(
                "top-level ShellAPI declarations must be exported interfaces, type aliases, enums, or namespaces"
                    .to_owned(),
            );
        };
        if !export.specifiers.is_empty() || export.source.is_some() {
            return Err("ShellAPI re-exports are not supported".to_owned());
        }
        let declaration = export
            .declaration
            .as_ref()
            .ok_or_else(|| "ShellAPI export must contain a declaration".to_owned())?;
        match declaration {
            Declaration::TSInterfaceDeclaration(interface) => {
                let name = interface.id.name.as_str().to_owned();
                let members = parse_interface(interface)?;
                if schema
                    .definitions
                    .insert(name.clone(), Definition::Interface(members))
                    .is_some()
                {
                    return Err(format!("duplicate TypeScript type {name}"));
                }
            }
            Declaration::TSTypeAliasDeclaration(alias) => {
                let name = alias.id.name.as_str().to_owned();
                if alias.type_parameters.is_some() {
                    return Err(format!("type alias {name} may not be generic"));
                }
                let ty = parse_type(&alias.type_annotation)?;
                if schema
                    .definitions
                    .insert(name.clone(), Definition::Alias(ty))
                    .is_some()
                {
                    return Err(format!("duplicate TypeScript type {name}"));
                }
            }
            Declaration::TSEnumDeclaration(declaration) => {
                let name = declaration.id.name.as_str().to_owned();
                let definition = parse_enum(declaration)?;
                if schema
                    .definitions
                    .insert(name.clone(), Definition::Enum(definition))
                    .is_some()
                {
                    return Err(format!("duplicate TypeScript type {name}"));
                }
            }
            Declaration::TSModuleDeclaration(module) => {
                let namespace = parse_namespace(module)?;
                if schema
                    .namespaces
                    .insert(namespace.name.clone(), namespace)
                    .is_some()
                {
                    return Err("duplicate TypeScript namespace".to_owned());
                }
            }
            _ => {
                return Err(
                    "top-level ShellAPI declarations may contain only interfaces, type aliases, enums, and namespaces"
                        .to_owned(),
                );
            }
        }
    }
    Ok(schema)
}

fn parse_enum(declaration: &TSEnumDeclaration<'_>) -> Result<EnumDefinition, String> {
    let enum_name = declaration.id.name.as_str();
    let mut members = Vec::new();
    let mut representation = None;
    for member in &declaration.body.members {
        let name = match &member.id {
            TSEnumMemberName::Identifier(identifier) => identifier.name.as_str().to_owned(),
            TSEnumMemberName::String(value) => value.value.as_str().to_owned(),
            TSEnumMemberName::ComputedString(_) | TSEnumMemberName::ComputedTemplateString(_) => {
                return Err(format!("enum {enum_name} requires static member names"));
            }
        };
        let initializer = member.initializer.as_ref().ok_or_else(|| {
            format!("enum {enum_name}.{name} requires an explicit string or integer value")
        })?;
        let value = parse_enum_value(initializer)?;
        let member_representation = match value {
            ConstantValue::String(_) => EnumRepresentation::String,
            ConstantValue::BigInt(_) => EnumRepresentation::Integer,
            ConstantValue::Bool(_) | ConstantValue::Number(_) => {
                return Err(format!(
                    "enum {enum_name}.{name} must use a string or integer value"
                ));
            }
        };
        if representation.is_some_and(|current| current != member_representation) {
            return Err(format!(
                "enum {enum_name} may not mix string and integer values"
            ));
        }
        representation = Some(member_representation);
        members.push(EnumMember { name, value });
    }
    let representation = representation
        .ok_or_else(|| format!("enum {enum_name} must contain at least one member"))?;
    Ok(EnumDefinition {
        representation,
        members,
    })
}

fn parse_enum_value(expression: &Expression<'_>) -> Result<ConstantValue, String> {
    match expression {
        Expression::StringLiteral(value) => {
            Ok(ConstantValue::String(value.value.as_str().to_owned()))
        }
        Expression::NumericLiteral(value)
            if value.value.is_finite()
                && value.value.fract() == 0.0
                && value.value.abs() <= 9_007_199_254_740_991.0 =>
        {
            parse_integer_literal(value.raw.as_ref().map_or("", |raw| raw.as_str()))
                .map(ConstantValue::BigInt)
        }
        Expression::UnaryExpression(unary)
            if unary.operator == UnaryOperator::UnaryNegation
                && matches!(&unary.argument, Expression::NumericLiteral(_)) =>
        {
            let Expression::NumericLiteral(value) = &unary.argument else {
                unreachable!();
            };
            if value.value.is_finite()
                && value.value.fract() == 0.0
                && value.value <= 9_007_199_254_740_991.0
            {
                parse_integer_literal(value.raw.as_ref().map_or("", |raw| raw.as_str())).and_then(
                    |value| {
                        value
                            .checked_neg()
                            .map(ConstantValue::BigInt)
                            .ok_or_else(|| "integer enum value is outside the i64 range".to_owned())
                    },
                )
            } else {
                Err("integer enum value is outside the i64 range".to_owned())
            }
        }
        _ => Err("enum values must be explicit string or integer literals".to_owned()),
    }
}

fn parse_integer_literal(raw: &str) -> Result<i64, String> {
    let normalized = raw.replace('_', "");
    let value = if let Some(value) = normalized.strip_prefix("0x") {
        i64::from_str_radix(value, 16)
    } else if let Some(value) = normalized.strip_prefix("0o") {
        i64::from_str_radix(value, 8)
    } else if let Some(value) = normalized.strip_prefix("0b") {
        i64::from_str_radix(value, 2)
    } else {
        normalized.parse::<i64>()
    };
    value.map_err(|_| "integer enum value is outside the i64 range".to_owned())
}

fn parse_interface(interface: &TSInterfaceDeclaration<'_>) -> Result<Vec<Member>, String> {
    let name = interface.id.name.as_str();
    if interface.type_parameters.is_some() || !interface.extends.is_empty() {
        return Err(format!(
            "interface {name} may not use generics or inheritance"
        ));
    }
    parse_data_members(&interface.body.body, name)
}

fn parse_data_members(signatures: &[TSSignature<'_>], owner: &str) -> Result<Vec<Member>, String> {
    let mut members = Vec::new();
    for signature in signatures {
        let TSSignature::TSPropertySignature(property) = signature else {
            if let TSSignature::TSMethodSignature(method) = signature {
                let name = property_name(&method.key).unwrap_or_else(|_| "<computed>".to_owned());
                return Err(format!(
                    "interface method `{name}` is not supported; declare ShellAPI commands with namespace functions"
                ));
            }
            return Err(format!(
                "interface {owner} may contain only named data properties"
            ));
        };
        if property.computed {
            return Err(format!(
                "interface {owner} may contain only named data properties"
            ));
        }
        let name = property_name(&property.key)?;
        let annotation = property
            .type_annotation
            .as_ref()
            .ok_or_else(|| format!("interface property {owner}.{name} requires a type"))?;
        let mut ty = parse_type(&annotation.type_annotation)?;
        if property.optional {
            ty = optional(ty);
        }
        members.push(Member { name, ty });
    }
    Ok(members)
}

fn parse_namespace(module: &TSModuleDeclaration<'_>) -> Result<Namespace, String> {
    if module.kind != TSModuleDeclarationKind::Namespace {
        return Err("TypeScript modules are not supported; use namespace".to_owned());
    }
    let TSModuleDeclarationName::Identifier(identifier) = &module.id else {
        return Err("ShellAPI namespace requires an identifier".to_owned());
    };
    let name = identifier.name.as_str().to_owned();
    let Some(TSModuleDeclarationBody::TSModuleBlock(body)) = module.body.as_ref() else {
        return Err(format!("namespace {name} requires a block body"));
    };

    let mut members = Vec::new();
    for statement in &body.body {
        let Statement::ExportNamedDeclaration(export) = statement else {
            return Err(format!(
                "namespace {name} members must be explicitly exported"
            ));
        };
        if !export.specifiers.is_empty() || export.source.is_some() {
            return Err(format!("namespace {name} may not contain re-exports"));
        }
        let declaration = export
            .declaration
            .as_ref()
            .ok_or_else(|| format!("namespace {name} export requires a declaration"))?;
        match declaration {
            Declaration::TSModuleDeclaration(child) => {
                members.push(NamespaceMember::Namespace(parse_namespace(child)?));
            }
            Declaration::FunctionDeclaration(function) => {
                members.push(parse_function(function, &name)?);
            }
            Declaration::TSInterfaceDeclaration(interface) => {
                members.push(NamespaceMember::Definition {
                    name: interface.id.name.as_str().to_owned(),
                    definition: Definition::Interface(parse_interface(interface)?),
                });
            }
            Declaration::TSTypeAliasDeclaration(alias) => {
                let alias_name = alias.id.name.as_str().to_owned();
                if alias.type_parameters.is_some() {
                    return Err(format!("type alias {alias_name} may not be generic"));
                }
                members.push(NamespaceMember::Definition {
                    name: alias_name,
                    definition: Definition::Alias(parse_type(&alias.type_annotation)?),
                });
            }
            Declaration::TSEnumDeclaration(declaration) => {
                members.push(NamespaceMember::Definition {
                    name: declaration.id.name.as_str().to_owned(),
                    definition: Definition::Enum(parse_enum(declaration)?),
                });
            }
            Declaration::VariableDeclaration(declaration) => {
                for constant in parse_constants(declaration, &name)? {
                    members.push(NamespaceMember::Constant(constant));
                }
            }
            _ => {
                return Err(format!(
                    "namespace {name} may contain only namespaces, functions, data types, enums, and constants"
                ));
            }
        }
    }
    Ok(Namespace { name, members })
}

fn parse_constants(
    declaration: &VariableDeclaration<'_>,
    namespace: &str,
) -> Result<Vec<Constant>, String> {
    if declaration.kind != VariableDeclarationKind::Const {
        return Err(format!("namespace {namespace} variables must use const"));
    }
    declaration
        .declarations
        .iter()
        .map(|declarator| {
            let BindingPattern::BindingIdentifier(identifier) = &declarator.id else {
                return Err("ShellAPI constants must use identifier names".to_owned());
            };
            let name = identifier.name.as_str().to_owned();
            let annotation = declarator.type_annotation.as_ref();
            let value = if let Some(initializer) = declarator.init.as_ref() {
                let value = parse_constant_value(initializer)?;
                if let Some(annotation) = annotation {
                    let ty = parse_type(&annotation.type_annotation)?;
                    if !constant_matches_type(&value, &ty) {
                        return Err(format!(
                            "ShellAPI constant {namespace}.{name} value does not match its primitive type"
                        ));
                    }
                }
                value
            } else {
                let annotation = annotation.ok_or_else(|| {
                    format!("ShellAPI constant {namespace}.{name} requires a literal type or value")
                })?;
                parse_literal_type_value(&annotation.type_annotation)?
            };
            Ok(Constant { name, value })
        })
        .collect()
}

fn parse_literal_type_value(ty: &AstType<'_>) -> Result<ConstantValue, String> {
    let AstType::TSLiteralType(literal) = ty else {
        return Err("ambient ShellAPI constants require a literal type".to_owned());
    };
    match &literal.literal {
        oxc_ast::ast::TSLiteral::BooleanLiteral(value) => Ok(ConstantValue::Bool(value.value)),
        oxc_ast::ast::TSLiteral::NumericLiteral(value) => Ok(ConstantValue::Number(value.value)),
        oxc_ast::ast::TSLiteral::BigIntLiteral(value) => value
            .value
            .as_str()
            .parse::<i64>()
            .map(ConstantValue::BigInt)
            .map_err(|_| "bigint constant is outside the i64 range".to_owned()),
        oxc_ast::ast::TSLiteral::StringLiteral(value) => {
            Ok(ConstantValue::String(value.value.as_str().to_owned()))
        }
        oxc_ast::ast::TSLiteral::UnaryExpression(unary)
            if unary.operator == UnaryOperator::UnaryNegation =>
        {
            match &unary.argument {
                Expression::NumericLiteral(value) => Ok(ConstantValue::Number(-value.value)),
                Expression::BigIntLiteral(value) => value
                    .value
                    .as_str()
                    .parse::<i64>()
                    .ok()
                    .and_then(i64::checked_neg)
                    .map(ConstantValue::BigInt)
                    .ok_or_else(|| "bigint constant is outside the i64 range".to_owned()),
                _ => Err("ambient ShellAPI constants require a primitive literal type".to_owned()),
            }
        }
        _ => Err("ambient ShellAPI constants require a primitive literal type".to_owned()),
    }
}

fn parse_constant_value(expression: &Expression<'_>) -> Result<ConstantValue, String> {
    match expression {
        Expression::BooleanLiteral(value) => Ok(ConstantValue::Bool(value.value)),
        Expression::NumericLiteral(value) if value.value.is_finite() => {
            Ok(ConstantValue::Number(value.value))
        }
        Expression::BigIntLiteral(value) => value
            .value
            .as_str()
            .parse::<i64>()
            .map(ConstantValue::BigInt)
            .map_err(|_| "bigint constant is outside the i64 range".to_owned()),
        Expression::StringLiteral(value) => {
            Ok(ConstantValue::String(value.value.as_str().to_owned()))
        }
        Expression::UnaryExpression(unary) if unary.operator == UnaryOperator::UnaryNegation => {
            match &unary.argument {
                Expression::NumericLiteral(value) if value.value.is_finite() => {
                    Ok(ConstantValue::Number(-value.value))
                }
                Expression::BigIntLiteral(value) => value
                    .value
                    .as_str()
                    .parse::<i64>()
                    .ok()
                    .and_then(i64::checked_neg)
                    .map(ConstantValue::BigInt)
                    .ok_or_else(|| "bigint constant is outside the i64 range".to_owned()),
                _ => Err("ShellAPI constants require primitive literal values".to_owned()),
            }
        }
        _ => Err("ShellAPI constants require primitive literal values".to_owned()),
    }
}

fn constant_matches_type(value: &ConstantValue, ty: &Type) -> bool {
    matches!(
        (value, ty),
        (ConstantValue::Bool(_), Type::Bool)
            | (ConstantValue::Number(_), Type::Number)
            | (ConstantValue::BigInt(_), Type::BigInt)
            | (ConstantValue::String(_), Type::String)
    )
}

fn parse_function(function: &Function<'_>, namespace: &str) -> Result<NamespaceMember, String> {
    let name = function
        .id
        .as_ref()
        .map(|identifier| identifier.name.as_str().to_owned())
        .ok_or_else(|| format!("namespace {namespace} contains an unnamed function"))?;
    if function.body.is_some()
        || function.generator
        || function.r#async
        || function.type_parameters.is_some()
        || function.this_param.is_some()
    {
        return Err(format!(
            "ShellAPI function {namespace}.{name} must be a non-generic declaration without an implementation"
        ));
    }
    let return_type = function
        .return_type
        .as_ref()
        .ok_or_else(|| format!("ShellAPI function {namespace}.{name} requires a return type"))?;
    Ok(NamespaceMember::Function {
        name,
        parameters: parse_parameters(&function.params)?,
        result: parse_promise(&return_type.type_annotation)?,
    })
}

fn parse_parameters(parameters: &FormalParameters<'_>) -> Result<Vec<Field>, String> {
    if parameters.rest.is_some() {
        return Err("ShellAPI rest parameters are not supported".to_owned());
    }
    let mut fields = Vec::new();
    let mut names = BTreeSet::new();
    let mut saw_optional = false;
    for parameter in &parameters.items {
        if !parameter.decorators.is_empty()
            || parameter.initializer.is_some()
            || parameter.accessibility.is_some()
            || parameter.readonly
            || parameter.r#override
        {
            return Err("ShellAPI parameters must be plain typed identifiers".to_owned());
        }
        let BindingPattern::BindingIdentifier(identifier) = &parameter.pattern else {
            return Err("ShellAPI parameter destructuring is not supported".to_owned());
        };
        let name = identifier.name.as_str().to_owned();
        if !is_identifier(&name) || !names.insert(name.clone()) {
            return Err(format!("invalid or duplicate parameter {name}"));
        }
        if saw_optional && !parameter.optional {
            return Err(format!(
                "required parameter {name} follows an optional parameter"
            ));
        }
        saw_optional |= parameter.optional;
        let annotation = parameter
            .type_annotation
            .as_ref()
            .ok_or_else(|| format!("ShellAPI parameter {name} requires a type"))?;
        let mut ty = parse_type(&annotation.type_annotation)?;
        if parameter.optional {
            ty = optional(ty);
        }
        fields.push(Field { name, ty });
    }
    Ok(fields)
}

fn parse_promise(ty: &AstType<'_>) -> Result<Type, String> {
    let AstType::TSTypeReference(reference) = ty else {
        return Err("ShellAPI functions must return Promise<T>".to_owned());
    };
    let Some(name) = simple_type_name(&reference.type_name) else {
        return Err("ShellAPI functions must return Promise<T>".to_owned());
    };
    let Some(arguments) = reference.type_arguments.as_ref() else {
        return Err("ShellAPI functions must return Promise<T>".to_owned());
    };
    if name != "Promise" || arguments.params.len() != 1 {
        return Err("ShellAPI functions must return Promise<T>".to_owned());
    }
    parse_type(&arguments.params[0])
}

fn parse_type(ty: &AstType<'_>) -> Result<Type, String> {
    match ty {
        AstType::TSBooleanKeyword(_) => Ok(Type::Bool),
        AstType::TSNumberKeyword(_) => Ok(Type::Number),
        AstType::TSBigIntKeyword(_) => Ok(Type::BigInt),
        AstType::TSStringKeyword(_) => Ok(Type::String),
        AstType::TSVoidKeyword(_) => Ok(Type::Object(Vec::new())),
        AstType::TSArrayType(array) => Ok(Type::Array(Box::new(parse_type(&array.element_type)?))),
        AstType::TSParenthesizedType(parenthesized) => parse_type(&parenthesized.type_annotation),
        AstType::TSTypeLiteral(literal) => Ok(Type::Object(parse_data_members(
            &literal.members,
            "inline object",
        )?)),
        AstType::TSTypeReference(reference) => {
            let name = simple_type_name(&reference.type_name)
                .ok_or_else(|| "qualified TypeScript type names are not supported".to_owned())?;
            match (name, reference.type_arguments.as_ref()) {
                ("Uint8Array", None) => Ok(Type::Bytes),
                ("Date", None) => Ok(Type::Date),
                ("Array", Some(arguments)) if arguments.params.len() == 1 => {
                    Ok(Type::Array(Box::new(parse_type(&arguments.params[0])?)))
                }
                (_, None) => Ok(Type::Named(name.to_owned())),
                _ => Err(format!("unsupported generic TypeScript type {name}")),
            }
        }
        AstType::TSUnionType(union) => {
            let mut value = None;
            let mut nullish = false;
            for member in &union.types {
                if matches!(
                    member,
                    AstType::TSNullKeyword(_) | AstType::TSUndefinedKeyword(_)
                ) {
                    nullish = true;
                } else if value.is_some() {
                    return Err("only nullable unions are supported".to_owned());
                } else {
                    value = Some(parse_type(member)?);
                }
            }
            if !nullish {
                return Err("only nullable unions are supported".to_owned());
            }
            value
                .map(optional)
                .ok_or_else(|| "nullable union requires a value type".to_owned())
        }
        _ => Err("unsupported TypeScript type in ShellAPI contract".to_owned()),
    }
}

fn optional(ty: Type) -> Type {
    if matches!(ty, Type::Optional(_)) {
        ty
    } else {
        Type::Optional(Box::new(ty))
    }
}

fn simple_type_name<'a>(name: &'a TSTypeName<'a>) -> Option<&'a str> {
    match name {
        TSTypeName::IdentifierReference(identifier) => Some(identifier.name.as_str()),
        TSTypeName::QualifiedName(_) | TSTypeName::ThisExpression(_) => None,
    }
}

fn property_name(key: &PropertyKey<'_>) -> Result<String, String> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Ok(identifier.name.as_str().to_owned()),
        _ => Err("ShellAPI data properties require identifier names".to_owned()),
    }
}

struct Endpoint {
    path: String,
    namespace: String,
    name: String,
    parameters: Vec<Field>,
    result: Type,
}

struct ApiConstant {
    path: String,
    value: ConstantValue,
}

/// One .d.ts input file with the C++ types, endpoints, and human-readable
/// display path derived from it. Used to emit a matching `.h`/`.cc` pair.
struct FileScope {
    stem: String,
    display: String,
    source: String,
    root_namespace: Option<Namespace>,
    definitions: BTreeMap<String, Definition>,
    endpoints: Vec<Endpoint>,
    constants: Vec<ApiConstant>,
}

fn file_stem(path: &std::path::Path) -> String {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    name.strip_suffix(".d.ts")
        .or_else(|| name.strip_suffix(".ts"))
        .unwrap_or(name)
        .to_owned()
}

#[allow(clippy::too_many_lines)]
#[cfg(test)]
pub(super) fn generate(
    sources: &[(std::path::PathBuf, String)],
    root: &str,
    application_id: &str,
    global: &str,
    facade: Facade,
) -> Result<GeneratedBindings, String> {
    generate_with_cpp_include_root(
        sources,
        root,
        application_id,
        global,
        facade,
        "tuffite/framework/shell_api",
    )
}

#[allow(clippy::too_many_lines)]
pub(super) fn generate_with_cpp_include_root(
    sources: &[(std::path::PathBuf, String)],
    root: &str,
    application_id: &str,
    global: &str,
    facade: Facade,
    cpp_include_root: &str,
) -> Result<GeneratedBindings, String> {
    if !is_identifier(root) {
        return Err("shellApi.namespace is invalid".to_owned());
    }
    if sources.is_empty() {
        return Err("ShellAPI declarations must not be empty".to_owned());
    }
    let source_names = sources
        .iter()
        .map(|(path, _)| path.to_string_lossy().replace('\\', "/"))
        .collect::<Vec<_>>();
    let mut schema = Schema {
        definitions: BTreeMap::new(),
        namespaces: BTreeMap::new(),
    };
    let mut declaration_imports = Vec::new();
    let mut file_scopes: Vec<FileScope> = Vec::new();
    let mut file_stems = BTreeSet::new();
    for (path, source) in sources {
        let parsed =
            parse_schema(source).map_err(|error| format!("{}: {error}", path.display()))?;
        let stem = file_stem(path);
        if !file_stems.insert(stem.clone()) {
            return Err(format!("duplicate ShellAPI definition stem {stem:?}"));
        }
        let display = path.to_string_lossy().replace('\\', "/");
        let root_namespace = parsed.namespaces.get(root).cloned();
        let mut file_definitions = parsed.definitions.clone();
        let mut file_endpoints: Vec<Endpoint> = Vec::new();
        let mut file_constants: Vec<ApiConstant> = Vec::new();
        if let Some(root_ns) = root_namespace.as_ref() {
            collect_definitions(root_ns, "", &mut file_definitions)?;
            collect_api(root_ns, "", &mut file_endpoints, &mut file_constants)?;
            declaration_imports.push(format!("./{stem}"));
        }
        file_scopes.push(FileScope {
            stem,
            display,
            source: source.clone(),
            root_namespace,
            definitions: file_definitions,
            endpoints: file_endpoints,
            constants: file_constants,
        });
        merge_schema(&mut schema, parsed)?;
    }
    let mut definitions = schema.definitions;
    let root_namespace = schema
        .namespaces
        .get(root)
        .ok_or_else(|| format!("TypeScript root namespace {root} was not found"))?;
    if facade == Facade::Framework {
        const NATIVE_BRIDGE_MEMBERS: [&str; 5] = [
            "invoke",
            "networkTestUrl",
            "postMessage",
            "addEventListener",
            "removeEventListener",
        ];
        for member in &root_namespace.members {
            let name = match member {
                NamespaceMember::Namespace(namespace) => &namespace.name,
                NamespaceMember::Definition { name, .. }
                | NamespaceMember::Function { name, .. } => name,
                NamespaceMember::Constant(constant) => &constant.name,
            };
            if NATIVE_BRIDGE_MEMBERS.contains(&name.as_str()) {
                return Err(format!(
                    "ShellAPI path tuffite.{name} is reserved by the native bridge"
                ));
            }
        }
    }
    collect_definitions(root_namespace, "", &mut definitions)?;
    validate_types(&definitions)?;
    validate_rust_names(root_namespace, &definitions)?;
    let mut endpoints = Vec::new();
    let mut constants = Vec::new();
    collect_api(root_namespace, "", &mut endpoints, &mut constants)?;
    if endpoints.is_empty() {
        return Err("TypeScript ShellAPI has no functions".to_owned());
    }
    for endpoint in &endpoints {
        for parameter in &endpoint.parameters {
            validate_type(&parameter.ty, &endpoint.namespace, &definitions)?;
        }
        validate_type(&endpoint.result, &endpoint.namespace, &definitions)?;
    }

    let source_name = source_names.join(", ");
    let mut rust_tokens = generate_root_types(&definitions)?;
    rust_tokens.extend(generate_namespace_modules(
        root_namespace,
        "",
        &definitions,
    )?);
    let mut ids = BTreeSet::new();
    let mut js_entries = Vec::new();
    for endpoint in &endpoints {
        let canonical = format!("{application_id}:{}", endpoint.path);
        let id = command_id(&canonical);
        if !ids.insert(id) {
            return Err(format!("command ID collision for {canonical}"));
        }
        let request = Value::Array(
            endpoint
                .parameters
                .iter()
                .map(|field| {
                    descriptor(
                        &field.ty,
                        &endpoint.namespace,
                        &definitions,
                        &mut Vec::new(),
                    )
                })
                .collect::<Result<_, _>>()?,
        );
        let response = descriptor(
            &endpoint.result,
            &endpoint.namespace,
            &definitions,
            &mut Vec::new(),
        )?;
        let fingerprint = schema_fingerprint(&request, &response)?;
        js_entries.push(match facade {
            Facade::Application => {
                json!([
                    endpoint.path,
                    format!("0x{id:016x}"),
                    format!("0x{fingerprint:016x}"),
                    request,
                    response
                ])
            }
            Facade::Framework => json!([endpoint.path, request, response]),
        });
    }
    for endpoint in &endpoints {
        let canonical = format!("{application_id}:{}", endpoint.path);
        let request = Value::Array(
            endpoint
                .parameters
                .iter()
                .map(|field| {
                    descriptor(
                        &field.ty,
                        &endpoint.namespace,
                        &definitions,
                        &mut Vec::new(),
                    )
                })
                .collect::<Result<_, _>>()?,
        );
        let response = descriptor(
            &endpoint.result,
            &endpoint.namespace,
            &definitions,
            &mut Vec::new(),
        )?;
        rust_tokens.extend(generate_endpoint_impl(
            endpoint,
            global,
            command_id(&canonical),
            schema_fingerprint(&request, &response)?,
            &definitions,
        )?);
    }

    let syntax = syn::parse2::<syn::File>(rust_tokens)
        .map_err(|error| format!("unable to parse generated Rust bindings: {error}"))?;
    let rust = format!(
        "// Generated from {source_name}. Do not edit.\n{}",
        prettyplease::unparse(&syntax)
    );

    let entries = serde_json::to_string_pretty(&js_entries).map_err(|error| error.to_string())?;
    let constants = javascript_constants(&constants)?;
    let global_json = serde_json::to_string(global).map_err(|error| error.to_string())?;
    let (bootstrap, factory) = match facade {
        Facade::Application => ("import '/tuffite-shell-api.js';\n", "createTypedCommandApi"),
        Facade::Framework => ("", "createFrameworkApi"),
    };
    let install = match facade {
        Facade::Application => format!(
            "Object.defineProperty(window, {global_json}, {{ configurable: false, enumerable: true, writable: false, value: api }});\n"
        ),
        Facade::Framework => String::new(),
    };
    let namespace = match facade {
        Facade::Application => format!(", {global_json}"),
        Facade::Framework => String::new(),
    };
    let javascript = format!(
        "// Generated from {source_name}. Do not edit.\n{bootstrap}import {{ {factory} }} from '/tuffite-runtime.js';\n\nconst api = {factory}({entries}, {constants}{namespace});\n{install}"
    );
    let mut declaration_type = declaration_imports
        .iter()
        .map(|path| format!("typeof import('{path}').{root}"))
        .collect::<Vec<_>>()
        .join(" & ");
    if facade == Facade::Framework {
        declaration_type.push_str(" & import('../runtime').NativeBridge");
    }
    let declarations = format!(
        "// Generated from {source_name}. Do not edit.\n\ndeclare global {{\n  interface Window {{\n    readonly {global}: {declaration_type};\n  }}\n}}\n\nexport {{}};\n"
    );
    let rust_files = generate_rust_per_file(&file_scopes, &definitions, application_id, global)?;
    let javascript_files =
        generate_javascript_per_file(&file_scopes, &definitions, application_id, facade)?;
    let declaration_files = generate_declarations_per_file(&file_scopes, &definitions)?;

    let mut cpp_files = BTreeMap::new();
    for (name, content) in generate_cpp_per_file(&file_scopes, &definitions, cpp_include_root)? {
        cpp_files.insert(name, content);
    }
    let command_ids = endpoints
        .iter()
        .map(|endpoint| {
            let canonical = format!("{application_id}:{}", endpoint.path);
            (
                format!("{global}.{}", endpoint.path),
                format!("{:016x}", command_id(&canonical)),
            )
        })
        .collect();
    Ok(GeneratedBindings {
        command_ids,
        rust,
        javascript,
        declarations,
        rust_files,
        javascript_files,
        declaration_files,
        cpp_files,
    })
}

fn schema_fingerprint(request: &Value, response: &Value) -> Result<u64, String> {
    let canonical = serde_json::to_vec(&json!({"request": request, "response": response}))
        .map_err(|error| format!("serialize normalized ShellAPI schema: {error}"))?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"tuffite.schema.v1\0");
    hasher.update(&canonical);
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&hasher.finalize().as_bytes()[..8]);
    Ok(u64::from_le_bytes(bytes))
}

fn generate_rust_per_file(
    scopes: &[FileScope],
    definitions: &BTreeMap<String, Definition>,
    application_id: &str,
    global: &str,
) -> Result<BTreeMap<String, String>, String> {
    let mut files = BTreeMap::new();
    for scope in scopes {
        let mut tokens = scope
            .definitions
            .iter()
            .filter(|(name, _)| !name.contains('.'))
            .map(|(name, definition)| generate_definition(name, definition, "", definitions))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .fold(TokenStream::new(), |mut output, item| {
                output.extend(item);
                output
            });
        if let Some(namespace) = scope.root_namespace.as_ref() {
            tokens.extend(generate_namespace_modules(namespace, "", definitions)?);
        }
        for endpoint in &scope.endpoints {
            let canonical = format!("{application_id}:{}", endpoint.path);
            let request = Value::Array(
                endpoint
                    .parameters
                    .iter()
                    .map(|field| {
                        descriptor(&field.ty, &endpoint.namespace, definitions, &mut Vec::new())
                    })
                    .collect::<Result<_, _>>()?,
            );
            let response = descriptor(
                &endpoint.result,
                &endpoint.namespace,
                definitions,
                &mut Vec::new(),
            )?;
            tokens.extend(generate_endpoint_impl(
                endpoint,
                global,
                command_id(&canonical),
                schema_fingerprint(&request, &response)?,
                definitions,
            )?);
        }
        let syntax = syn::parse2::<syn::File>(tokens).map_err(|error| {
            format!(
                "unable to parse generated Rust fragment for {}: {error}",
                scope.display
            )
        })?;
        files.insert(
            format!("{}.rs", scope.stem),
            format!(
                "// Generated from {}. Do not edit.\n{}",
                scope.display,
                prettyplease::unparse(&syntax)
            ),
        );
    }
    Ok(files)
}

fn javascript_entries(
    endpoints: &[Endpoint],
    definitions: &BTreeMap<String, Definition>,
    application_id: &str,
    facade: Facade,
) -> Result<Vec<Value>, String> {
    endpoints
        .iter()
        .map(|endpoint| {
            let canonical = format!("{application_id}:{}", endpoint.path);
            let request = Value::Array(
                endpoint
                    .parameters
                    .iter()
                    .map(|field| {
                        descriptor(&field.ty, &endpoint.namespace, definitions, &mut Vec::new())
                    })
                    .collect::<Result<_, _>>()?,
            );
            let response = descriptor(
                &endpoint.result,
                &endpoint.namespace,
                definitions,
                &mut Vec::new(),
            )?;
            Ok(match facade {
                Facade::Application => json!([
                    endpoint.path,
                    format!("0x{:016x}", command_id(&canonical)),
                    request,
                    response
                ]),
                Facade::Framework => json!([endpoint.path, request, response]),
            })
        })
        .collect()
}

fn generate_javascript_per_file(
    scopes: &[FileScope],
    definitions: &BTreeMap<String, Definition>,
    application_id: &str,
    facade: Facade,
) -> Result<BTreeMap<String, String>, String> {
    let mut files = BTreeMap::new();
    for scope in scopes {
        let entries = serde_json::to_string_pretty(&javascript_entries(
            &scope.endpoints,
            definitions,
            application_id,
            facade,
        )?)
        .map_err(|error| error.to_string())?;
        let constants = javascript_constants(&scope.constants)?;
        files.insert(
            format!("{}.js", scope.stem),
            format!(
                "// Generated from {}. Do not edit.\n\nexport const entries = {entries};\nexport const constants = {constants};\n",
                scope.display
            ),
        );
    }
    Ok(files)
}

fn generate_declarations_per_file(
    scopes: &[FileScope],
    definitions: &BTreeMap<String, Definition>,
) -> Result<BTreeMap<String, String>, String> {
    let stem_index = build_stem_index(scopes);
    let mut files = BTreeMap::new();
    for scope in scopes {
        let mut dependency_stems = BTreeSet::new();
        for (qualified, definition) in &scope.definitions {
            let context = parent_path(qualified);
            match definition {
                Definition::Interface(members) => {
                    for member in members {
                        collect_type_stems(
                            &member.ty,
                            context,
                            definitions,
                            &stem_index,
                            &mut dependency_stems,
                        );
                    }
                }
                Definition::Alias(ty) => {
                    collect_type_stems(
                        ty,
                        context,
                        definitions,
                        &stem_index,
                        &mut dependency_stems,
                    );
                }
                Definition::Enum(_) => {}
            }
        }
        for endpoint in &scope.endpoints {
            for parameter in &endpoint.parameters {
                collect_type_stems(
                    &parameter.ty,
                    &endpoint.namespace,
                    definitions,
                    &stem_index,
                    &mut dependency_stems,
                );
            }
            collect_type_stems(
                &endpoint.result,
                &endpoint.namespace,
                definitions,
                &stem_index,
                &mut dependency_stems,
            );
        }
        dependency_stems.remove(&scope.stem);

        let mut imports = String::new();
        for dependency_stem in dependency_stems {
            let names = definitions
                .keys()
                .filter(|name| {
                    !name.contains('.') && stem_index.get(*name) == Some(&dependency_stem)
                })
                .cloned()
                .collect::<Vec<_>>();
            if names.is_empty() {
                return Err(format!(
                    "{} uses a nested type from {dependency_stem}.d.ts; cross-file declarations must export shared types at module scope",
                    scope.display
                ));
            }
            writeln!(
                imports,
                "import type {{ {} }} from './{dependency_stem}';",
                names.join(", ")
            )
            .unwrap();
        }
        if !imports.is_empty() {
            imports.push('\n');
        }
        files.insert(
            format!("{}.d.ts", scope.stem),
            format!(
                "// Generated from {}. Do not edit.\n\n{imports}{}",
                scope.display, scope.source
            ),
        );
    }
    Ok(files)
}

#[allow(clippy::too_many_lines)]
fn build_stem_index(scopes: &[FileScope]) -> BTreeMap<String, String> {
    let mut index = BTreeMap::new();
    for scope in scopes {
        for name in scope.definitions.keys() {
            index.insert(name.clone(), scope.stem.clone());
        }
    }
    index
}

fn collect_type_stems(
    ty: &Type,
    context: &str,
    global_definitions: &BTreeMap<String, Definition>,
    stem_index: &BTreeMap<String, String>,
    stems: &mut BTreeSet<String>,
) {
    match ty {
        Type::Array(inner) | Type::Optional(inner) => {
            collect_type_stems(inner, context, global_definitions, stem_index, stems);
        }
        Type::Named(name) => {
            if let Some((qualified, _)) = resolve_definition(name, context, global_definitions) {
                if let Some(stem) = stem_index.get(qualified) {
                    stems.insert(stem.clone());
                }
                let qualified = qualified.to_owned();
                if let Some(inner_definition) = global_definitions.get(&qualified) {
                    let inner_context = parent_path(&qualified).to_owned();
                    match inner_definition {
                        Definition::Alias(alias) => collect_type_stems(
                            alias,
                            &inner_context,
                            global_definitions,
                            stem_index,
                            stems,
                        ),
                        Definition::Interface(members) => {
                            for member in members {
                                collect_type_stems(
                                    &member.ty,
                                    &inner_context,
                                    global_definitions,
                                    stem_index,
                                    stems,
                                );
                            }
                        }
                        Definition::Enum(_) => {}
                    }
                }
            }
        }
        Type::Object(members) => {
            for member in members {
                collect_type_stems(&member.ty, context, global_definitions, stem_index, stems);
            }
        }
        Type::Bool | Type::Number | Type::BigInt | Type::String | Type::Bytes | Type::Date => {}
    }
}

fn generate_cpp_per_file(
    scopes: &[FileScope],
    global_definitions: &BTreeMap<String, Definition>,
    cpp_include_root: &str,
) -> Result<Vec<(String, String)>, String> {
    let stem_index = build_stem_index(scopes);
    let mut files = Vec::new();
    for scope in scopes {
        let (header, source) =
            generate_cpp_scope_files(scope, &stem_index, global_definitions, cpp_include_root)?;
        files.push((format!("{}.h", scope.stem), header));
        files.push((format!("{}.cc", scope.stem), source));
    }
    Ok(files)
}

#[allow(clippy::too_many_lines)]
fn generate_cpp_scope_files(
    scope: &FileScope,
    stem_index: &BTreeMap<String, String>,
    global_definitions: &BTreeMap<String, Definition>,
    cpp_include_root: &str,
) -> Result<(String, String), String> {
    let stem = &scope.stem;
    let guard = format!(
        "TUFFITE_FRAMEWORK_SHELL_API_{}_H_",
        stem.to_uppercase().replace(['.', '-'], "_")
    );

    // Determine cross-file dependencies from types referenced in this scope.
    let mut dep_stems: BTreeSet<String> = BTreeSet::new();
    for (qualified, definition) in &scope.definitions {
        let context = parent_path(qualified).to_owned();
        match definition {
            Definition::Interface(members) => {
                for member in members {
                    collect_type_stems(
                        &member.ty,
                        &context,
                        global_definitions,
                        stem_index,
                        &mut dep_stems,
                    );
                }
            }
            Definition::Alias(ty) => {
                collect_type_stems(ty, &context, global_definitions, stem_index, &mut dep_stems);
            }
            Definition::Enum(_) => {}
        }
    }
    for endpoint in &scope.endpoints {
        for parameter in &endpoint.parameters {
            collect_type_stems(
                &parameter.ty,
                &endpoint.namespace,
                global_definitions,
                stem_index,
                &mut dep_stems,
            );
        }
        collect_type_stems(
            &endpoint.result,
            &endpoint.namespace,
            global_definitions,
            stem_index,
            &mut dep_stems,
        );
    }
    dep_stems.remove(stem);

    let mut header = format!(
        "// Generated from {}. Do not edit.\n\
         // Copyright 2026 The Tuffite Authors\n\
         // SPDX-License-Identifier: MIT OR Apache-2.0\n\n\
         #ifndef {guard}\n\
         #define {guard}\n\n\
         #include \"tuffite/components/dispatch/browser/shell_api_protocol.h\"\n",
        scope.display,
    );
    for dep in &dep_stems {
        writeln!(header, "#include \"{cpp_include_root}/{dep}.h\"").unwrap();
    }
    header.push_str("\nnamespace tuffite::shell_api::generated {\n\n");

    // Emit forward declarations for structs and enums.
    for (qualified, definition) in &scope.definitions {
        let name = cpp_type_name(qualified);
        let namespace = cpp_definition_namespace(qualified);
        if !namespace.is_empty() {
            writeln!(header, "namespace {namespace} {{").unwrap();
        }
        match definition {
            Definition::Interface(_) => writeln!(header, "struct {name};").unwrap(),
            Definition::Enum(_) => writeln!(header, "enum class {name};").unwrap(),
            Definition::Alias(_) => {}
        }
        if !namespace.is_empty() {
            writeln!(header, "}}  // namespace {namespace}").unwrap();
        }
    }
    header.push('\n');

    // Emit full type definitions in dependency order. A forward declaration is
    // not sufficient for a by-value field, alias, optional, or container.
    for qualified in cpp_definition_order(&scope.definitions, global_definitions)? {
        let definition = scope
            .definitions
            .get(qualified)
            .expect("definition order only contains local definitions");
        let name = cpp_type_name(qualified);
        let namespace = cpp_definition_namespace(qualified);
        let context = parent_path(qualified);
        if !namespace.is_empty() {
            writeln!(header, "namespace {namespace} {{\n").unwrap();
        }
        match definition {
            Definition::Interface(members) => {
                writeln!(header, "struct {name} {{").unwrap();
                for member in members {
                    writeln!(
                        header,
                        "  {} {}{{}};",
                        cpp_type(&member.ty, context, global_definitions)?,
                        cpp_field_name(&member.name)
                    )
                    .unwrap();
                }
                header.push_str("};\n\n");
            }
            Definition::Alias(ty) => {
                writeln!(
                    header,
                    "using {name} = {};\n",
                    cpp_type(ty, context, global_definitions)?
                )
                .unwrap();
            }
            Definition::Enum(enumeration) => {
                writeln!(header, "enum class {name} {{").unwrap();
                for member in &enumeration.members {
                    writeln!(header, "  {},", cpp_type_name(&member.name)).unwrap();
                }
                header.push_str("};\n\n");
            }
        }
        if !namespace.is_empty() {
            writeln!(header, "}}  // namespace {namespace}\n").unwrap();
        }
    }

    // Emit endpoint declarations.
    for endpoint in &scope.endpoints {
        let name = cpp_endpoint_name(endpoint);
        let namespace = cpp_endpoint_namespace(endpoint);
        let reflection = cpp_reflection_name(endpoint);
        if !namespace.is_empty() {
            writeln!(header, "namespace {namespace} {{\n").unwrap();
        }
        writeln!(header, "struct {name}Request {{").unwrap();
        for parameter in &endpoint.parameters {
            writeln!(
                header,
                "  {} {}{{}};",
                cpp_type(&parameter.ty, &endpoint.namespace, global_definitions)?,
                cpp_field_name(&parameter.name)
            )
            .unwrap();
        }
        header.push_str("};\n");
        writeln!(
            header,
            "using {name}Result = {};\n\
             using {name}Callback =\n\
             \x20   base::OnceCallback<void(rpc::Status, {name}Result)>;\n\
             using {name}Method = Method<{name}Request, {name}Result>;\n\n\
             namespace reflection {{\n\
             extern const {name}Method {reflection};\n\
             }}  // namespace reflection\n",
            cpp_type(&endpoint.result, &endpoint.namespace, global_definitions)?
        )
        .unwrap();
        if !namespace.is_empty() {
            writeln!(header, "}}  // namespace {namespace}\n").unwrap();
        }
    }

    // Emit Encode/Decode declarations for named types in this file.
    for (qualified, definition) in &scope.definitions {
        if matches!(definition, Definition::Interface(_) | Definition::Enum(_)) {
            let name = cpp_named_type_name(qualified);
            writeln!(
                header,
                "bool DecodeValue(base::span<const uint8_t>* input, {name}* value);\n\
                 void EncodeValue(std::vector<uint8_t>* output, const {name}& value);"
            )
            .unwrap();
            if matches!(definition, Definition::Enum(enumeration) if enumeration.representation == EnumRepresentation::String)
            {
                writeln!(header, "std::string_view ToString({name} value);").unwrap();
            }
        }
    }

    header.push_str("\n}  // namespace tuffite::shell_api::generated\n\n");
    writeln!(header, "#endif  // {guard}").unwrap();

    // Source file.
    let mut source = format!(
        "// Generated from {}. Do not edit.\n\
         // Copyright 2026 The Tuffite Authors\n\
         // SPDX-License-Identifier: MIT OR Apache-2.0\n\n\
         #include \"{cpp_include_root}/{stem}.h\"\n\n\
         #include <string_view>\n\
         #include <utility>\n\n\
         #include \"base/functional/bind.h\"\n\n\
         namespace tuffite::shell_api::generated {{\n\n",
        scope.display,
    );

    // Encode/Decode definitions for each named type.
    for (qualified, definition) in &scope.definitions {
        let name = cpp_named_type_name(qualified);
        match definition {
            Definition::Interface(members) => {
                let decode = if members.is_empty() {
                    "true".to_owned()
                } else {
                    members
                        .iter()
                        .map(|member| {
                            format!(
                                "DecodeValue(input, &value->{})",
                                cpp_field_name(&member.name)
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" &&\n         ")
                };
                writeln!(
                    source,
                    "bool DecodeValue(base::span<const uint8_t>* input, {name}* value) {{\n\
                     \x20 return {decode};\n\
                     }}"
                )
                .unwrap();
                writeln!(
                    source,
                    "void EncodeValue(std::vector<uint8_t>* output, const {name}& value) {{"
                )
                .unwrap();
                for member in members {
                    writeln!(
                        source,
                        "  EncodeValue(output, value.{});",
                        cpp_field_name(&member.name)
                    )
                    .unwrap();
                }
                source.push_str("}\n\n");
            }
            Definition::Enum(enumeration) => {
                let raw_type = match enumeration.representation {
                    EnumRepresentation::String => "std::string",
                    EnumRepresentation::Integer => "int64_t",
                };
                writeln!(
                    source,
                    "bool DecodeValue(base::span<const uint8_t>* input, {name}* value) {{\n\
                     \x20 {raw_type} raw{{}};\n\
                     \x20 if (!DecodeValue(input, &raw)) {{\n\
                     \x20   return false;\n\
                     \x20 }}"
                )
                .unwrap();
                for member in &enumeration.members {
                    writeln!(
                        source,
                        "  if (raw == {}) {{\n\
                         \x20   *value = {name}::{};\n\
                         \x20   return true;\n\
                         \x20 }}",
                        cpp_constant(&member.value)?,
                        cpp_type_name(&member.name)
                    )
                    .unwrap();
                }
                source.push_str("  return false;\n}\n");
                if enumeration.representation == EnumRepresentation::String {
                    writeln!(
                        source,
                        "std::string_view ToString({name} value) {{\n  switch (value) {{"
                    )
                    .unwrap();
                    for member in &enumeration.members {
                        writeln!(
                            source,
                            "    case {name}::{}:\n      return {};",
                            cpp_type_name(&member.name),
                            cpp_constant(&member.value)?
                        )
                        .unwrap();
                    }
                    source.push_str("  }\n  return {};\n}\n");
                    writeln!(
                        source,
                        "void EncodeValue(std::vector<uint8_t>* output, const {name}& value) {{\n\
                         \x20 EncodeValue(output, std::string(ToString(value)));\n\
                         }}\n"
                    )
                    .unwrap();
                    continue;
                }
                writeln!(
                    source,
                    "void EncodeValue(std::vector<uint8_t>* output, const {name}& value) {{\n\
                     \x20 switch (value) {{"
                )
                .unwrap();
                for member in &enumeration.members {
                    writeln!(
                        source,
                        "    case {name}::{}:\n\
                         \x20     return EncodeValue(output, {});",
                        cpp_type_name(&member.name),
                        cpp_constant(&member.value)?
                    )
                    .unwrap();
                }
                source.push_str("  }\n}\n\n");
            }
            Definition::Alias(_) => {}
        }
    }

    if !scope.endpoints.is_empty() {
        source.push_str("namespace {\n\n");
        for endpoint in &scope.endpoints {
            let name = cpp_endpoint_name(endpoint);
            let namespace = cpp_endpoint_namespace(endpoint);
            let scope_prefix = if namespace.is_empty() {
                String::new()
            } else {
                format!("{namespace}::")
            };
            let route_name = cpp_route_name(endpoint);
            writeln!(
                source,
                "void {route_name}({scope_prefix}{name}Method::Handler implementation,"
            )
            .unwrap();
            source.push_str("                const ::tuffite::dispatch::Context& context,\n");
            source.push_str("                std::string_view,\n");
            source.push_str("                base::span<const uint8_t> input,\n");
            source.push_str("                rpc::Reply reply) {\n");
            writeln!(source, "  {scope_prefix}{name}Request request{{}};").unwrap();
            for parameter in &endpoint.parameters {
                writeln!(
                    source,
                    "  if (!DecodeValue(&input, &request.{})) {{",
                    cpp_field_name(&parameter.name)
                )
                .unwrap();
                source.push_str("    std::move(reply).Run(\n");
                source.push_str(
                    "        static_cast<int32_t>(rpc::Status::kInvalidArgument), {});\n",
                );
                source.push_str("    return;\n");
                source.push_str("  }\n");
            }
            source.push_str("  if (!input.empty()) {\n");
            source.push_str("    std::move(reply).Run(\n");
            source.push_str("        static_cast<int32_t>(rpc::Status::kInvalidArgument), {});\n");
            source.push_str("    return;\n");
            source.push_str("  }\n");
            writeln!(
                source,
                "  implementation.Run(\n\
                 \x20     context, std::move(request),\n\
                 \x20     base::BindOnce(\n\
                 \x20         [](rpc::Reply reply, rpc::Status status,\n\
                 \x20            {scope_prefix}{name}Result result) {{\n\
                 \x20           std::vector<uint8_t> encoded;\n\
                 \x20           if (status == rpc::Status::kOk) {{\n\
                 \x20             EncodeValue(&encoded, result);\n\
                 \x20           }}\n\
                 \x20           std::move(reply).Run(static_cast<int32_t>(status),\n\
                 \x20                                std::move(encoded));\n\
                 \x20         }},\n\
                 \x20         std::move(reply)));\n\
                 }}\n\n\
                 rpc::Handler Bind{route_name}({scope_prefix}{name}Method::Handler implementation) {{\n\
                 \x20 return base::BindRepeating(&{route_name}, std::move(implementation));\n\
                 }}\n"
            )
            .unwrap();
        }
        source.push_str("}  // namespace\n\n");
        for endpoint in &scope.endpoints {
            let name = cpp_endpoint_name(endpoint);
            let namespace = cpp_endpoint_namespace(endpoint);
            let reflection = cpp_reflection_name(endpoint);
            let scope_prefix = if namespace.is_empty() {
                String::new()
            } else {
                format!("{namespace}::")
            };
            let reflection_namespace = if namespace.is_empty() {
                "reflection".to_owned()
            } else {
                format!("{namespace}::reflection")
            };
            writeln!(
                source,
                "namespace {reflection_namespace} {{\n\n\
                 const {scope_prefix}{name}Method {reflection}{{\"tuffite.{}\", &Bind{}}};\n\n\
                 }}  // namespace {reflection_namespace}\n",
                endpoint.path,
                cpp_route_name(endpoint)
            )
            .unwrap();
        }
    }

    source.push_str("}  // namespace tuffite::shell_api::generated\n");
    Ok((header, source))
}

fn cpp_definition_order<'a>(
    local: &'a BTreeMap<String, Definition>,
    global: &BTreeMap<String, Definition>,
) -> Result<Vec<&'a String>, String> {
    fn collect_dependencies(
        ty: &Type,
        context: &str,
        global: &BTreeMap<String, Definition>,
        dependencies: &mut BTreeSet<String>,
    ) {
        match ty {
            Type::Array(inner) | Type::Optional(inner) => {
                collect_dependencies(inner, context, global, dependencies);
            }
            Type::Named(name) => {
                if let Some((qualified, _)) = resolve_definition(name, context, global) {
                    dependencies.insert(qualified.to_owned());
                }
            }
            Type::Object(members) => {
                for member in members {
                    collect_dependencies(&member.ty, context, global, dependencies);
                }
            }
            Type::Bool | Type::Number | Type::BigInt | Type::String | Type::Bytes | Type::Date => {}
        }
    }

    fn visit<'a>(
        name: &'a String,
        local: &'a BTreeMap<String, Definition>,
        global: &BTreeMap<String, Definition>,
        visiting: &mut Vec<&'a String>,
        visited: &mut BTreeSet<String>,
        ordered: &mut Vec<&'a String>,
    ) -> Result<(), String> {
        if visited.contains(name) {
            return Ok(());
        }
        if let Some(position) = visiting.iter().position(|candidate| *candidate == name) {
            let mut cycle = visiting[position..]
                .iter()
                .map(|value| value.as_str())
                .collect::<Vec<_>>();
            cycle.push(name);
            return Err(format!(
                "recursive C++ value type dependency is unsupported: {}",
                cycle.join(" -> ")
            ));
        }
        visiting.push(name);
        let definition = local.get(name).expect("local definition exists");
        let context = parent_path(name);
        let mut dependencies = BTreeSet::new();
        match definition {
            Definition::Interface(members) => {
                for member in members {
                    collect_dependencies(&member.ty, context, global, &mut dependencies);
                }
            }
            Definition::Alias(ty) => {
                collect_dependencies(ty, context, global, &mut dependencies);
            }
            Definition::Enum(_) => {}
        }
        for dependency in dependencies {
            if dependency == *name {
                return Err(format!(
                    "recursive C++ value type dependency is unsupported: {name} -> {name}"
                ));
            }
            if let Some(local_name) = local.get_key_value(&dependency).map(|(key, _)| key) {
                visit(local_name, local, global, visiting, visited, ordered)?;
            }
        }
        visiting.pop();
        visited.insert(name.clone());
        ordered.push(name);
        Ok(())
    }

    let mut visiting = Vec::new();
    let mut visited = BTreeSet::new();
    let mut ordered = Vec::new();
    for name in local.keys() {
        visit(
            name,
            local,
            global,
            &mut visiting,
            &mut visited,
            &mut ordered,
        )?;
    }
    Ok(ordered)
}

fn cpp_type(
    ty: &Type,
    context: &str,
    definitions: &BTreeMap<String, Definition>,
) -> Result<String, String> {
    Ok(match ty {
        Type::Bool => "bool".to_owned(),
        Type::Number => "double".to_owned(),
        Type::BigInt | Type::Date => "int64_t".to_owned(),
        Type::String => "std::string".to_owned(),
        Type::Bytes => "std::vector<uint8_t>".to_owned(),
        Type::Array(inner) => format!("std::vector<{}>", cpp_type(inner, context, definitions)?),
        Type::Optional(inner) => {
            format!("std::optional<{}>", cpp_type(inner, context, definitions)?)
        }
        Type::Named(name) => {
            let (qualified, _) = resolve_definition(name, context, definitions)
                .ok_or_else(|| format!("unknown TypeScript type {name}"))?;
            cpp_named_type_name(qualified)
        }
        Type::Object(members) if members.is_empty() => "std::monostate".to_owned(),
        Type::Object(_) => return Err("inline data objects are not supported".to_owned()),
    })
}

fn cpp_type_name(value: &str) -> String {
    rust_type_ident(value.rsplit('.').next().unwrap_or(value))
        .to_string()
        .replace("r#", "")
}

fn cpp_definition_namespace(value: &str) -> String {
    let parent = parent_path(value);
    if parent.is_empty() {
        return String::new();
    }
    parent
        .split('.')
        .map(|segment| rust_ident(segment).to_string().replace("r#", ""))
        .collect::<Vec<_>>()
        .join("::")
}

fn cpp_endpoint_name(endpoint: &Endpoint) -> String {
    cpp_type_name(&endpoint.name)
}

fn cpp_endpoint_namespace(endpoint: &Endpoint) -> String {
    if endpoint.namespace.is_empty() {
        return String::new();
    }
    endpoint
        .namespace
        .split('.')
        .map(|segment| rust_ident(segment).to_string().replace("r#", ""))
        .collect::<Vec<_>>()
        .join("::")
}

fn cpp_reflection_name(endpoint: &Endpoint) -> String {
    endpoint.name.replace('$', "_dollar_")
}

fn cpp_route_name(endpoint: &Endpoint) -> String {
    format!(
        "Route{}",
        endpoint
            .path
            .split('.')
            .map(|segment| rust_type_ident(segment).to_string().replace("r#", ""))
            .collect::<String>()
    )
}

fn cpp_named_type_name(value: &str) -> String {
    let namespace = cpp_definition_namespace(value);
    let name = cpp_type_name(value);
    if namespace.is_empty() {
        name
    } else {
        format!("{namespace}::{name}")
    }
}

fn cpp_field_name(value: &str) -> String {
    rust_ident(value).to_string().replace("r#", "")
}

fn cpp_constant(value: &ConstantValue) -> Result<String, String> {
    match value {
        ConstantValue::String(value) => {
            serde_json::to_string(value).map_err(|error| error.to_string())
        }
        ConstantValue::BigInt(value) => Ok(value.to_string()),
        ConstantValue::Bool(value) => Ok(value.to_string()),
        ConstantValue::Number(value) if value.is_finite() => Ok(value.to_string()),
        ConstantValue::Number(_) => Err("non-finite C++ constant".to_owned()),
    }
}

#[cfg(feature = "cli")]
pub(super) fn command_paths(
    sources: &[(std::path::PathBuf, String)],
    root: &str,
) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for (_, source) in sources {
        let schema = parse_schema(source)?;
        if let Some(namespace) = schema.namespaces.get(root) {
            let mut endpoints = Vec::new();
            collect_api(namespace, "", &mut endpoints, &mut Vec::new())?;
            paths.extend(endpoints.into_iter().map(|e| format!("{root}.{}", e.path)));
        }
    }
    Ok(paths)
}

fn collect_api(
    namespace: &Namespace,
    prefix: &str,
    endpoints: &mut Vec<Endpoint>,
    constants: &mut Vec<ApiConstant>,
) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for member in &namespace.members {
        let name = match member {
            NamespaceMember::Namespace(child) => &child.name,
            NamespaceMember::Definition { name, .. } | NamespaceMember::Function { name, .. } => {
                name
            }
            NamespaceMember::Constant(constant) => &constant.name,
        };
        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        if !path.split('.').all(is_identifier)
            || matches!(
                name.as_str(),
                "then" | "constructor" | "prototype" | "__proto__"
            )
            || !names.insert(name)
        {
            return Err(format!("unsafe or duplicate ShellAPI path {path}"));
        }
        match member {
            NamespaceMember::Definition { .. } => {}
            NamespaceMember::Function {
                parameters, result, ..
            } => endpoints.push(Endpoint {
                path,
                namespace: prefix.to_owned(),
                name: name.clone(),
                parameters: parameters.clone(),
                result: result.clone(),
            }),
            NamespaceMember::Namespace(child) => {
                collect_api(child, &path, endpoints, constants)?;
            }
            NamespaceMember::Constant(constant) => constants.push(ApiConstant {
                path,
                value: constant.value.clone(),
            }),
        }
    }
    Ok(())
}

fn collect_definitions(
    namespace: &Namespace,
    prefix: &str,
    definitions: &mut BTreeMap<String, Definition>,
) -> Result<(), String> {
    for member in &namespace.members {
        match member {
            NamespaceMember::Namespace(child) => {
                collect_definitions(child, &join_path(prefix, &child.name), definitions)?;
            }
            NamespaceMember::Definition { name, definition } => {
                let qualified = join_path(prefix, name);
                if definitions
                    .insert(qualified.clone(), definition.clone())
                    .is_some()
                {
                    return Err(format!("duplicate TypeScript type {qualified}"));
                }
            }
            NamespaceMember::Constant(_) | NamespaceMember::Function { .. } => {}
        }
    }
    Ok(())
}

fn validate_types(definitions: &BTreeMap<String, Definition>) -> Result<(), String> {
    for (name, definition) in definitions {
        let context = parent_path(name);
        match definition {
            Definition::Interface(members) => {
                validate_members(name, members, context, definitions)?;
            }
            Definition::Alias(ty) => {
                if matches!(ty, Type::Object(_)) {
                    return Err(format!(
                        "type alias {name} may not use an inline object; use interface"
                    ));
                }
                validate_type(ty, context, definitions)?;
            }
            Definition::Enum(enumeration) => validate_enum(name, enumeration)?,
        }
        descriptor(
            &Type::Named(leaf_name(name).to_owned()),
            context,
            definitions,
            &mut Vec::new(),
        )?;
    }
    Ok(())
}

fn validate_enum(name: &str, enumeration: &EnumDefinition) -> Result<(), String> {
    let mut names = BTreeSet::new();
    let mut values = BTreeSet::new();
    for member in &enumeration.members {
        let value = match &member.value {
            ConstantValue::String(value) => format!("s:{value}"),
            ConstantValue::BigInt(value) => format!("i:{value}"),
            ConstantValue::Bool(_) | ConstantValue::Number(_) => unreachable!(),
        };
        if !is_identifier(&member.name) || !names.insert(&member.name) || !values.insert(value) {
            return Err(format!("enum {name} has an invalid or duplicate member"));
        }
    }
    Ok(())
}

fn validate_members(
    name: &str,
    members: &[Member],
    context: &str,
    definitions: &BTreeMap<String, Definition>,
) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for member in members {
        if !is_identifier(&member.name) || !names.insert(&member.name) {
            return Err(format!(
                "invalid or duplicate member in {name}: {}",
                member.name
            ));
        }
        validate_type(&member.ty, context, definitions)?;
    }
    Ok(())
}

fn validate_type(
    ty: &Type,
    context: &str,
    definitions: &BTreeMap<String, Definition>,
) -> Result<(), String> {
    match ty {
        Type::Named(name) => resolve_definition(name, context, definitions)
            .map(|_| ())
            .ok_or_else(|| format!("unknown TypeScript type {name}")),
        Type::Array(inner) | Type::Optional(inner) => validate_type(inner, context, definitions),
        Type::Object(members) if members.is_empty() => Ok(()),
        Type::Object(_) => Err(
            "inline data objects are not supported; declare and reference an interface".to_owned(),
        ),
        _ => Ok(()),
    }
}

fn validate_rust_names(
    root_namespace: &Namespace,
    definitions: &BTreeMap<String, Definition>,
) -> Result<(), String> {
    for (qualified, definition) in definitions {
        match definition {
            Definition::Interface(members) => {
                validate_rust_field_names(qualified, members.iter().map(|member| &member.name))?;
            }
            Definition::Enum(enumeration) => {
                validate_rust_type_names(
                    qualified,
                    enumeration.members.iter().map(|member| &member.name),
                )?;
            }
            Definition::Alias(_) => {}
        }
    }

    let mut root_types = BTreeMap::new();
    for qualified in definitions.keys().filter(|name| !name.contains('.')) {
        insert_rust_name(
            &mut root_types,
            &rust_type_ident(qualified).to_string(),
            qualified,
            "root type",
        )?;
    }
    validate_rust_namespace_names(root_namespace, "", &mut root_types)
}

fn validate_rust_namespace_names(
    namespace: &Namespace,
    scope: &str,
    inherited_types: &mut BTreeMap<String, String>,
) -> Result<(), String> {
    let mut types = if scope.is_empty() {
        std::mem::take(inherited_types)
    } else {
        BTreeMap::new()
    };
    let mut values = BTreeMap::new();
    for member in &namespace.members {
        let source = match member {
            NamespaceMember::Namespace(child) => &child.name,
            NamespaceMember::Definition { name, .. } | NamespaceMember::Function { name, .. } => {
                name
            }
            NamespaceMember::Constant(constant) => &constant.name,
        };
        match member {
            NamespaceMember::Namespace(child) => {
                insert_rust_name(
                    &mut types,
                    &rust_ident(&child.name).to_string(),
                    source,
                    scope,
                )?;
                validate_rust_namespace_names(
                    child,
                    &join_path(scope, &child.name),
                    &mut BTreeMap::new(),
                )?;
            }
            NamespaceMember::Definition { .. } if scope.is_empty() => {
                // Root definitions were inserted from the complete definition map.
            }
            NamespaceMember::Definition { name, .. } | NamespaceMember::Function { name, .. } => {
                insert_rust_name(
                    &mut types,
                    &rust_type_ident(name).to_string(),
                    source,
                    scope,
                )?;
            }
            NamespaceMember::Constant(constant) => {
                insert_rust_name(
                    &mut values,
                    &rust_constant_ident(&constant.name).to_string(),
                    source,
                    scope,
                )?;
            }
        }
        if let NamespaceMember::Function {
            name, parameters, ..
        } = member
        {
            validate_rust_field_names(name, parameters.iter().map(|field| &field.name))?;
        }
    }
    Ok(())
}

fn validate_rust_type_names<'a>(
    owner: &str,
    names: impl Iterator<Item = &'a String>,
) -> Result<(), String> {
    let mut generated = BTreeMap::new();
    for name in names {
        insert_rust_name(
            &mut generated,
            &rust_type_ident(name).to_string(),
            name,
            owner,
        )?;
    }
    Ok(())
}

fn validate_rust_field_names<'a>(
    owner: &str,
    names: impl Iterator<Item = &'a String>,
) -> Result<(), String> {
    let mut generated = BTreeMap::new();
    for name in names {
        insert_rust_name(&mut generated, &rust_ident(name).to_string(), name, owner)?;
    }
    Ok(())
}

fn insert_rust_name(
    generated: &mut BTreeMap<String, String>,
    rust_name: &str,
    source_name: &str,
    owner: &str,
) -> Result<(), String> {
    if let Some(existing) = generated.insert(rust_name.to_owned(), source_name.to_owned()) {
        return Err(format!(
            "ShellAPI names {existing:?} and {source_name:?} collide as Rust identifier {rust_name:?} in {owner}"
        ));
    }
    Ok(())
}

fn generate_root_types(definitions: &BTreeMap<String, Definition>) -> Result<TokenStream, String> {
    let definitions = definitions
        .iter()
        .filter(|(name, _)| !name.contains('.'))
        .map(|(name, definition)| generate_definition(name, definition, "", definitions))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(quote!(#(#definitions)*))
}

fn generate_namespace_modules(
    namespace: &Namespace,
    scope: &str,
    definitions: &BTreeMap<String, Definition>,
) -> Result<TokenStream, String> {
    let mut output = TokenStream::new();
    for member in &namespace.members {
        let tokens = match member {
            NamespaceMember::Namespace(child) => {
                let name = rust_ident(&child.name);
                let body =
                    generate_namespace_modules(child, &join_path(scope, &child.name), definitions)?;
                quote!(pub mod #name { #body })
            }
            NamespaceMember::Definition { name, definition } if !scope.is_empty() => {
                generate_definition(name, definition, scope, definitions)?
            }
            NamespaceMember::Definition { .. } => TokenStream::new(),
            NamespaceMember::Constant(constant) => generate_rust_constant(constant),
            NamespaceMember::Function {
                name,
                parameters,
                result,
            } => generate_request(name, parameters, result, scope, definitions)?,
        };
        output.extend(tokens);
    }
    Ok(output)
}

fn generate_definition(
    name: &str,
    definition: &Definition,
    scope: &str,
    definitions: &BTreeMap<String, Definition>,
) -> Result<TokenStream, String> {
    let name = rust_type_ident(name);
    Ok(match definition {
        Definition::Interface(members) => {
            let fields = members
                .iter()
                .map(|member| {
                    let name = rust_ident(&member.name);
                    let ty = rust_type(&member.ty, scope, scope, definitions)?;
                    Ok(quote!(pub #name: #ty,))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let codec = generate_struct_codec(&name, members);
            quote! {
                #[derive(Clone, Debug, PartialEq, ::tuffite::__private::serde::Serialize, ::tuffite::__private::serde::Deserialize)]
                #[serde(crate = "::tuffite::__private::serde", rename_all = "camelCase")]
                pub struct #name { #(#fields)* }
                #codec
            }
        }
        Definition::Alias(ty) => {
            let ty = rust_type(ty, scope, scope, definitions)?;
            quote!(pub type #name = #ty;)
        }
        Definition::Enum(enumeration) => generate_enum(&name, enumeration),
    })
}

fn generate_struct_codec(name: &Ident, members: &[Member]) -> TokenStream {
    let names = members
        .iter()
        .map(|member| rust_ident(&member.name))
        .collect::<Vec<_>>();
    quote! {
        impl ::tuffite::__private::codec::Encode for #name {
            fn encode(
                &self,
                encoder: &mut ::tuffite::__private::codec::Encoder,
            ) -> Result<(), ::tuffite::__private::codec::CodecError> {
                #(::tuffite::__private::codec::Encode::encode(&self.#names, encoder)?;)*
                Ok(())
            }
        }
        impl ::tuffite::__private::codec::Decode for #name {
            fn decode(
                decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
            ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
                Ok(Self {
                    #(#names: ::tuffite::__private::codec::Decode::decode(decoder)?,)*
                })
            }
        }
    }
}

fn generate_enum(name: &Ident, enumeration: &EnumDefinition) -> TokenStream {
    let variants = enumeration
        .members
        .iter()
        .map(|member| rust_type_ident(&member.name))
        .collect::<Vec<_>>();
    let values = enumeration
        .members
        .iter()
        .map(|member| rust_constant_value(&member.value))
        .collect::<Vec<_>>();
    let primitive = match enumeration.representation {
        EnumRepresentation::String => quote!(String),
        EnumRepresentation::Integer => quote!(i64),
    };
    quote! {
        #[derive(Clone, Debug, Eq, PartialEq)]
        pub enum #name { #(#variants,)* }
        impl ::tuffite::__private::serde::Serialize for #name {
            fn serialize<S: ::tuffite::__private::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let value: #primitive = match self { #(Self::#variants => #values,)* };
                ::tuffite::__private::serde::Serialize::serialize(&value, serializer)
            }
        }
        impl<'de> ::tuffite::__private::serde::Deserialize<'de> for #name {
            fn deserialize<D: ::tuffite::__private::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = <#primitive as ::tuffite::__private::serde::Deserialize>::deserialize(deserializer)?;
                match value {
                    #(value if value == #values => Ok(Self::#variants),)*
                    _ => Err(::tuffite::__private::serde::de::Error::custom("invalid Shell API enum value")),
                }
            }
        }
        impl ::tuffite::__private::codec::Encode for #name {
            fn encode(
                &self,
                encoder: &mut ::tuffite::__private::codec::Encoder,
            ) -> Result<(), ::tuffite::__private::codec::CodecError> {
                let value: #primitive = match self {
                    #(Self::#variants => #values,)*
                };
                ::tuffite::__private::codec::Encode::encode(&value, encoder)
            }
        }
        impl ::tuffite::__private::codec::Decode for #name {
            fn decode(
                decoder: &mut ::tuffite::__private::codec::Decoder<'_>,
            ) -> Result<Self, ::tuffite::__private::codec::CodecError> {
                let value = <#primitive as ::tuffite::__private::codec::Decode>::decode(decoder)?;
                match value {
                    #(value if value == #values => Ok(Self::#variants),)*
                    _ => Err(::tuffite::__private::codec::CodecError),
                }
            }
        }
    }
}

fn generate_request(
    name: &str,
    parameters: &[Field],
    result: &Type,
    scope: &str,
    definitions: &BTreeMap<String, Definition>,
) -> Result<TokenStream, String> {
    let request = rust_type_ident(name);
    let fields = parameters
        .iter()
        .map(|field| rust_ident(&field.name))
        .collect::<Vec<_>>();
    let field_types = parameters
        .iter()
        .map(|field| rust_type(&field.ty, scope, scope, definitions))
        .collect::<Result<Vec<_>, _>>()?;
    let response_type = rust_type(result, scope, scope, definitions)?;
    Ok(quote! {
        #[derive(Clone, Debug, PartialEq)]
        pub struct #request { #(pub #fields: #field_types,)* }
        impl #request {
            pub fn respond(
                response: Result<#response_type, ::tuffite::CommandError>,
            ) -> ::tuffite::CommandResult {
                use ::tuffite::__private::codec::Encode as _;
                let response = response?;
                let mut encoder = ::tuffite::__private::codec::Encoder::new();
                response.encode(&mut encoder).map_err(|_| {
                    ::tuffite::CommandError::Failed(
                        "unable to encode ShellAPI response".to_owned(),
                    )
                })?;
                Ok(encoder.finish())
            }
        }
    })
}

fn rust_type(
    ty: &Type,
    output_scope: &str,
    lookup_scope: &str,
    definitions: &BTreeMap<String, Definition>,
) -> Result<TokenStream, String> {
    Ok(match ty {
        Type::Bool => quote!(bool),
        Type::Number => quote!(f64),
        Type::BigInt => quote!(i64),
        Type::String => quote!(String),
        Type::Bytes => quote!(::tuffite::Bytes),
        Type::Date => quote!(::tuffite::Timestamp),
        Type::Array(inner) => {
            let inner = rust_type(inner, output_scope, lookup_scope, definitions)?;
            quote!(Vec<#inner>)
        }
        Type::Optional(inner) => {
            let inner = rust_type(inner, output_scope, lookup_scope, definitions)?;
            quote!(Option<#inner>)
        }
        Type::Named(name) => {
            let (qualified, _) = resolve_definition(name, lookup_scope, definitions)
                .ok_or_else(|| format!("unknown TypeScript type {name}"))?;
            rust_qualified_type(qualified, output_scope)
        }
        Type::Object(members) if members.is_empty() => quote!(()),
        Type::Object(_) => return Err("inline data objects are not supported".to_owned()),
    })
}

fn descriptor(
    ty: &Type,
    context: &str,
    definitions: &BTreeMap<String, Definition>,
    stack: &mut Vec<String>,
) -> Result<Value, String> {
    Ok(match ty {
        Type::Bool => json!({"kind":"boolean"}),
        Type::Number => json!({"kind":"number"}),
        Type::BigInt => json!({"kind":"bigint"}),
        Type::String => json!({"kind":"string"}),
        Type::Bytes => json!({"kind":"bytes"}),
        Type::Date => json!({"kind":"date"}),
        Type::Array(inner) => {
            json!({"kind":"array","item":descriptor(inner, context, definitions, stack)?})
        }
        Type::Optional(inner) => {
            json!({"kind":"optional","value":descriptor(inner, context, definitions, stack)?})
        }
        Type::Named(name) => {
            let (qualified, definition) = resolve_definition(name, context, definitions)
                .ok_or_else(|| format!("unknown type {name}"))?;
            if stack.iter().any(|item| item == qualified) {
                return Err(format!(
                    "recursive ShellAPI data type: {} -> {qualified}",
                    stack.join(" -> ")
                ));
            }
            stack.push(qualified.to_owned());
            let value = match definition {
                Definition::Alias(alias) => {
                    descriptor(alias, parent_path(qualified), definitions, stack)?
                }
                Definition::Interface(members) => descriptor(
                    &Type::Object(members.clone()),
                    parent_path(qualified),
                    definitions,
                    stack,
                )?,
                Definition::Enum(enumeration) => match enumeration.representation {
                    EnumRepresentation::String => json!({"kind":"string"}),
                    EnumRepresentation::Integer => json!({"kind":"integer"}),
                },
            };
            stack.pop();
            value
        }
        Type::Object(members) if members.is_empty() => json!({"kind":"void"}),
        Type::Object(members) => {
            let fields = members
                .iter()
                .map(|member| {
                    Ok::<Value, String>(json!([
                        member.name,
                        descriptor(&member.ty, context, definitions, stack)?
                    ]))
                })
                .collect::<Result<Vec<_>, _>>()?;
            json!({"kind":"object","fields":fields})
        }
    })
}

fn resolve_definition<'a>(
    name: &str,
    context: &str,
    definitions: &'a BTreeMap<String, Definition>,
) -> Option<(&'a str, &'a Definition)> {
    let mut scope = context;
    loop {
        let candidate = join_path(scope, name);
        if let Some((qualified, definition)) = definitions.get_key_value(&candidate) {
            return Some((qualified.as_str(), definition));
        }
        let Some((parent, _)) = scope.rsplit_once('.') else {
            break;
        };
        scope = parent;
    }
    definitions
        .get_key_value(name)
        .map(|(qualified, definition)| (qualified.as_str(), definition))
}

fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}.{name}")
    }
}

fn parent_path(path: &str) -> &str {
    path.rsplit_once('.').map_or("", |(parent, _)| parent)
}

fn leaf_name(path: &str) -> &str {
    path.rsplit_once('.').map_or(path, |(_, name)| name)
}

fn generate_endpoint_impl(
    endpoint: &Endpoint,
    global: &str,
    command_id: u64,
    schema_fingerprint: u64,
    definitions: &BTreeMap<String, Definition>,
) -> Result<TokenStream, String> {
    let request_type = endpoint_request_type(endpoint);
    let command_name = LitStr::new(&format!("{global}.{}", endpoint.path), Span::call_site());
    let command_id = rust_u64_literal(command_id);
    let schema_fingerprint = rust_u64_literal(schema_fingerprint);
    let fields = endpoint
        .parameters
        .iter()
        .map(|field| rust_ident(&field.name))
        .collect::<Vec<_>>();
    let field_types = endpoint
        .parameters
        .iter()
        .map(|field| rust_type(&field.ty, "", &endpoint.namespace, definitions))
        .collect::<Result<Vec<_>, _>>()?;
    let result_type = rust_type(&endpoint.result, "", &endpoint.namespace, definitions)?;
    let decoder_setup = if endpoint.parameters.is_empty() {
        quote! {
            if !command.payload().is_empty() {
                return Err(::tuffite::CommandError::InvalidArgument(
                    "trailing ShellAPI request data".to_owned(),
                ));
            }
        }
    } else {
        quote! {
            let mut decoder = ::tuffite::__private::codec::Decoder::new(command.payload());
        }
    };
    let decodes = fields.iter().zip(&field_types).map(|(field, ty)| {
        quote! {
            let #field: #ty = ::tuffite::__private::codec::Decode::decode(&mut decoder)
                .map_err(|_| ::tuffite::CommandError::InvalidArgument(
                    "invalid ShellAPI request".to_owned(),
                ))?;
        }
    });
    let decoder_finish = (!endpoint.parameters.is_empty()).then(|| {
        quote! {
            decoder.finish().map_err(|_| ::tuffite::CommandError::InvalidArgument(
                "trailing ShellAPI request data".to_owned(),
            ))?;
        }
    });
    let decode_request = if endpoint.parameters.is_empty() {
        quote!(Self::__tuffite_decode(command)?;)
    } else {
        quote!(let request = Self::__tuffite_decode(command)?;)
    };
    let arguments = fields.iter().map(|field| quote!(request.#field));

    Ok(quote! {
        impl ::tuffite::ShellApiRequest for #request_type {
            type Response = #result_type;
            const CHANNEL: &'static str = #command_name;
            fn encode(&self, encoder: &mut ::tuffite::__private::codec::Encoder) -> Result<(), ::tuffite::__private::codec::CodecError> {
                let _ = &encoder;
                #(::tuffite::__private::codec::Encode::encode(&self.#fields, encoder)?;)*
                Ok(())
            }
        }
        impl #request_type {
            pub const COMMAND: ::tuffite::CommandDescriptor =
                ::tuffite::CommandDescriptor::new(
                    #command_name,
                    #command_id,
                    #schema_fingerprint,
                );

            #[doc(hidden)]
            pub fn __tuffite_decode(
                command: ::tuffite::Command<'_>,
            ) -> Result<Self, ::tuffite::CommandError> {
                if command.id() != Self::COMMAND.id() {
                    return Err(::tuffite::CommandError::UnknownCommand);
                }
                if command.schema_fingerprint() != Self::COMMAND.schema_fingerprint() {
                    return Err(::tuffite::CommandError::SchemaMismatch(
                        "ShellAPI schema fingerprint mismatch; regenerate application bindings"
                            .to_owned(),
                    ));
                }
                #decoder_setup
                #(#decodes)*
                #decoder_finish
                Ok(Self { #(#fields,)* })
            }

            #[doc(hidden)]
            pub fn __tuffite_dispatch<A>(
                command: ::tuffite::Command<'_>,
                application: &mut A,
                handler: fn(&mut A #(, #field_types)*)
                    -> Result<#result_type, ::tuffite::CommandError>,
            ) -> ::tuffite::CommandResult {
                #decode_request
                Self::respond(handler(application #(, #arguments)*))
            }
        }
    })
}

fn rust_qualified_type(qualified: &str, output_scope: &str) -> TokenStream {
    if parent_path(qualified) == output_scope {
        let name = rust_type_ident(leaf_name(qualified));
        return quote!(#name);
    }
    let mut segments = qualified.split('.').collect::<Vec<_>>();
    let leaf = rust_type_ident(segments.pop().expect("qualified type is non-empty"));
    let modules = segments.into_iter().map(rust_ident).collect::<Vec<_>>();
    let path = quote!(#(#modules::)* #leaf);
    if output_scope.is_empty() {
        return path;
    }
    let parents = (0..=output_scope.matches('.').count())
        .map(|_| quote!(super::))
        .collect::<Vec<_>>();
    quote!(#(#parents)* #path)
}

fn rust_type_ident(name: &str) -> Ident {
    let mut result = String::new();
    let mut uppercase = true;
    for character in name.chars() {
        if character == '_' || character == '-' || character == '.' {
            uppercase = true;
        } else if character == '$' {
            result.push_str("Dollar");
            uppercase = true;
        } else if uppercase {
            result.push(character.to_ascii_uppercase());
            uppercase = false;
        } else {
            result.push(character);
        }
    }
    if result.is_empty() {
        result.push_str("Underscore");
    }
    rust_token_ident(&result)
}

fn endpoint_request_type(endpoint: &Endpoint) -> TokenStream {
    let request = rust_type_ident(&endpoint.name);
    if endpoint.namespace.is_empty() {
        quote!(#request)
    } else {
        let modules = endpoint
            .namespace
            .split('.')
            .map(rust_ident)
            .collect::<Vec<_>>();
        quote!(#(#modules::)* #request)
    }
}

fn generate_rust_constant(constant: &Constant) -> TokenStream {
    let name = rust_constant_ident(&constant.name);
    match &constant.value {
        ConstantValue::Bool(value) => quote!(pub const #name: bool = #value;),
        ConstantValue::Number(value) => quote!(pub const #name: f64 = #value;),
        ConstantValue::BigInt(value) => quote!(pub const #name: i64 = #value;),
        ConstantValue::String(value) => {
            let value = LitStr::new(value, Span::call_site());
            quote!(pub const #name: &str = #value;)
        }
    }
}

fn rust_constant_ident(name: &str) -> Ident {
    let mut output = String::new();
    for (index, character) in name.chars().enumerate() {
        if character == '$' {
            if !output.is_empty() && !output.ends_with('_') {
                output.push('_');
            }
            output.push_str("DOLLAR_");
        } else if character.is_ascii_uppercase() && index != 0 && !output.ends_with('_') {
            output.push('_');
            output.push(character);
        } else {
            output.push(character.to_ascii_uppercase());
        }
    }
    rust_token_ident(&output)
}

fn rust_constant_value(value: &ConstantValue) -> TokenStream {
    match value {
        ConstantValue::String(value) => {
            let value = LitStr::new(value, Span::call_site());
            quote!(String::from(#value))
        }
        ConstantValue::BigInt(value) => quote!(#value),
        ConstantValue::Bool(value) => quote!(#value),
        ConstantValue::Number(value) => quote!(#value),
    }
}

fn javascript_constants(constants: &[ApiConstant]) -> Result<String, String> {
    let mut output = String::from("[\n");
    for constant in constants {
        writeln!(
            output,
            "  [{}, {}],",
            serde_json::to_string(&constant.path).map_err(|error| error.to_string())?,
            javascript_constant_value(&constant.value)?
        )
        .unwrap();
    }
    output.push(']');
    Ok(output)
}

fn javascript_constant_value(value: &ConstantValue) -> Result<String, String> {
    match value {
        ConstantValue::Bool(value) => Ok(value.to_string()),
        ConstantValue::Number(value) => {
            serde_json::to_string(value).map_err(|error| error.to_string())
        }
        ConstantValue::BigInt(value) => Ok(format!("{value}n")),
        ConstantValue::String(value) => {
            serde_json::to_string(value).map_err(|error| error.to_string())
        }
    }
}

fn rust_ident(value: &str) -> Ident {
    let mut output = String::new();
    for (index, character) in value.chars().enumerate() {
        if character == '.' {
            output.push('_');
            continue;
        }
        if character == '$' {
            if !output.is_empty() && !output.ends_with('_') {
                output.push('_');
            }
            output.push_str("dollar_");
            continue;
        }
        if character.is_ascii_uppercase() {
            if index != 0 && !output.ends_with('_') {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
        } else {
            output.push(character);
        }
    }
    rust_token_ident(&output)
}

fn rust_token_ident(value: &str) -> Ident {
    const RAW_IDENTIFIERS: &[&str] = &[
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "do",
        "dyn", "else", "enum", "extern", "final", "fn", "for", "gen", "if", "impl", "in", "let",
        "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref", "return",
        "static", "struct", "trait", "try", "type", "typeof", "union", "unsafe", "unsized", "use",
        "virtual", "where", "while", "yield",
    ];
    if RAW_IDENTIFIERS.contains(&value) {
        Ident::new_raw(value, Span::call_site())
    } else if matches!(
        value,
        "_" | "crate" | "false" | "self" | "Self" | "super" | "true"
    ) {
        Ident::new(&format!("{value}_"), Span::call_site())
    } else {
        Ident::new(value, Span::call_site())
    }
}

fn rust_u64_literal(value: u64) -> LitInt {
    let hex = format!("{value:016x}");
    LitInt::new(
        &format!(
            "0x{}_{}_{}_{}u64",
            &hex[0..4],
            &hex[4..8],
            &hex[8..12],
            &hex[12..16]
        ),
        Span::call_site(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typed_nested_api_and_generates_both_codecs() {
        let source = "export interface Reply { message: string; count?: number; } export namespace api { export namespace commands { export function doSomething(input: string, bytes: Uint8Array): Promise<Reply>; } }";
        let sources = vec![(Path::new("shell_api.d.ts").to_owned(), source.to_owned())];
        let generated = generate(
            &sources,
            "api",
            "com.example.app",
            "example",
            Facade::Application,
        )
        .unwrap();
        assert!(generated.rust.contains("pub mod commands"));
        assert!(generated.rust.contains("pub struct DoSomething"));
        assert!(
            generated
                .rust
                .contains("impl ::tuffite::ShellApiRequest for commands::DoSomething")
        );
        assert!(generated.rust.contains("type Response = Reply"));
        assert!(!generated.rust.contains("trait Handler"));
        assert!(
            generated
                .rust
                .contains("pub const COMMAND: ::tuffite::CommandDescriptor")
        );
        assert!(generated.rust.contains("pub fn __tuffite_decode"));
        assert!(!generated.rust.contains("macro_rules!"));
        let scope_h = generated.cpp_files.get("shell_api.h").expect("shell_api.h");
        assert!(scope_h.contains("struct Reply;"));
        assert!(scope_h.contains("namespace commands"));
        assert!(scope_h.contains("using DoSomethingCallback"));
        assert!(scope_h.contains("extern const DoSomethingMethod doSomething;"));
        assert!(!scope_h.contains("void DoSomething("));
        assert!(!scope_h.contains("namespace types"));
        assert!(generated.javascript.contains("createTypedCommandApi"));
        assert!(generated.rust_files.contains_key("shell_api.rs"));
        assert!(
            generated
                .javascript_files
                .get("shell_api.js")
                .expect("shell_api.js")
                .contains("export const entries")
        );
        assert!(
            generated
                .declaration_files
                .get("shell_api.d.ts")
                .expect("shell_api.d.ts")
                .contains("export namespace api")
        );
        assert!(
            generated
                .rust
                .contains("Result<super::Reply, ::tuffite::CommandError>")
        );
        assert!(
            generated
                .declarations
                .contains("readonly example: typeof import('./shell_api').api")
        );
    }

    #[test]
    fn emits_cpp_value_types_in_dependency_order() {
        let source = r"
            export interface A { value: Z; }
            export interface Z { value: string; }
            export type ResultAlias = A;
            export namespace api {
              export function check(value: ResultAlias): Promise<void>;
            }
        ";
        let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
        let generated = generate(
            &sources,
            "api",
            "com.example.app",
            "example",
            Facade::Application,
        )
        .unwrap();
        let header = generated.cpp_files.get("api.h").expect("api.h");
        let z = header.find("struct Z {").expect("Z definition");
        let a = header.find("struct A {").expect("A definition");
        let alias = header
            .find("using ResultAlias = A;")
            .expect("alias definition");
        assert!(z < a, "dependency must precede by-value user\n{header}");
        assert!(a < alias, "alias target must precede alias\n{header}");
    }

    #[test]
    fn schema_fingerprint_changes_without_changing_command_identity() {
        fn identifiers(source: &str) -> Vec<String> {
            let generated = generate(
                &[(Path::new("api.d.ts").to_owned(), source.to_owned())],
                "api",
                "com.example.app",
                "example",
                Facade::Application,
            )
            .unwrap();
            generated
                .javascript
                .match_indices("0x")
                .map(|(offset, _)| generated.javascript[offset..offset + 18].to_owned())
                .collect()
        }
        let first = identifiers(
            "export interface Input { first: string; second: string; } export namespace api { export function check(value: Input): Promise<void>; }",
        );
        let reordered = identifiers(
            "export interface Input { second: string; first: string; } export namespace api { export function check(value: Input): Promise<void>; }",
        );
        assert_eq!(first[0], reordered[0], "command ID must remain stable");
        assert_ne!(first[1], reordered[1], "schema fingerprint must change");
    }

    #[test]
    fn rejects_dynamic_types() {
        let source = "export namespace api { export function bad(value: any): Promise<string>; }";
        let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
        assert!(
            generate(
                &sources,
                "api",
                "com.example.app",
                "example",
                Facade::Application,
            )
            .is_err()
        );
    }

    #[test]
    fn quotes_and_formats_rust_identifiers_and_literals() {
        let source = r#"
            export interface $reply { type: string; $value: string; _: string; }
            export declare namespace api {
              export const $protocolVersion: "quoted\nvalue";
              export function match($input: $reply): Promise<$reply>;
            }
        "#;
        let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
        let generated = generate(
            &sources,
            "api",
            "com.example.app",
            "example",
            Facade::Application,
        )
        .unwrap();
        syn::parse_file(&generated.rust).expect("generated Rust parses");
        assert!(generated.rust.contains("pub struct DollarReply"));
        assert!(generated.rust.contains("pub r#type: String"));
        assert!(generated.rust.contains("pub dollar_value: String"));
        assert!(generated.rust.contains("pub __: String"));
        assert!(generated.rust.contains("pub struct Match"));
        assert!(generated.rust.contains("pub const DOLLAR_PROTOCOL_VERSION"));
        assert!(generated.rust.contains("\"quoted\\nvalue\""));
    }

    #[test]
    fn rejects_names_that_collide_after_rust_normalization() {
        for source in [
            "export interface Pair { fooBar: string; foo_bar: string; } export namespace api { export function call(value: Pair): Promise<void>; }",
            "export interface FooBar { value: string; } export interface foo_bar { value: string; } export namespace api { export function call(): Promise<void>; }",
            "export namespace api { export const fooBar = 1; export const foo_bar = 2; export function call(): Promise<void>; }",
            "export interface Pair { $value: string; dollarValue: string; } export namespace api { export function call(value: Pair): Promise<void>; }",
        ] {
            let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
            let error = generate(
                &sources,
                "api",
                "com.example.app",
                "example",
                Facade::Application,
            )
            .unwrap_err();
            assert!(error.contains("collide as Rust identifier"), "{error}");
        }
    }

    #[test]
    fn rejects_generic_shell_api_streams() {
        let source = "export namespace api { export function count(): AsyncIterable<number>; }";
        let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
        let error = generate(
            &sources,
            "api",
            "com.example.app",
            "example",
            Facade::Application,
        )
        .unwrap_err();
        assert!(error.ends_with("ShellAPI functions must return Promise<T>"));
    }

    #[test]
    fn rejects_inline_and_recursive_data_types_during_generation() {
        for source in [
            "export namespace api { export function call(value: { name: string }): Promise<void>; }",
            "export interface Node { next?: Node; } export namespace api { export function call(value: Node): Promise<void>; }",
        ] {
            let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
            let error = generate(
                &sources,
                "api",
                "com.example.app",
                "example",
                Facade::Application,
            )
            .unwrap_err();
            assert!(
                error.contains("inline data objects")
                    || error.contains("recursive ShellAPI data type")
            );
        }
    }

    #[test]
    fn rejects_interface_methods() {
        let source = "export interface Api { call(value: string): Promise<void>; }";
        let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
        let error = generate(
            &sources,
            "Api",
            "com.example.app",
            "example",
            Facade::Application,
        )
        .unwrap_err();
        assert!(error.contains("interface method"));
    }

    #[test]
    fn supports_alias_enum_and_namespace_constant() {
        let source = r#"
            export type ProjectId = string;
            export enum Mode { Read = "read", Write = "write" }
            export declare namespace api {
              export namespace commands {
                export const protocolVersion: 1;
                export function open(id: ProjectId, mode: Mode): Promise<ProjectId>;
              }
            }
        "#;
        let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
        let generated = generate(
            &sources,
            "api",
            "com.example.app",
            "example",
            Facade::Application,
        )
        .unwrap();
        assert!(generated.rust.contains("pub type ProjectId = String"));
        assert!(generated.rust.contains("pub enum Mode"));
        assert!(
            generated
                .rust
                .contains("pub const PROTOCOL_VERSION: f64 = 1")
        );
        assert!(generated.javascript.contains("commands.protocolVersion"));
        let api_h = generated.cpp_files.get("api.h").expect("api.h");
        let api_cc = generated.cpp_files.get("api.cc").expect("api.cc");
        assert!(api_h.contains("std::string_view ToString(Mode value)"));
        assert!(api_cc.contains("std::string(ToString(value))"));
        assert!(!api_cc.contains("EncodeValue(output, \"read\")"));
    }

    #[test]
    fn generated_cpp_source_uses_configured_include_root() {
        let sources = vec![(
            Path::new("api.d.ts").to_owned(),
            "export namespace app { export function ping(): Promise<void>; }".to_owned(),
        )];
        let generated = generate_with_cpp_include_root(
            &sources,
            "app",
            "org.test.app",
            "app",
            Facade::Application,
            "application/protocol",
        )
        .unwrap();
        assert!(generated.cpp_files["api.cc"].contains("#include \"application/protocol/api.h\""));
    }

    #[test]
    fn generates_framework_facade_with_builtin_transport() {
        let source = "export namespace tuffite { export namespace net { export function request(url: string): Promise<Uint8Array>; } }";
        let sources = vec![(Path::new("shell_api.d.ts").to_owned(), source.to_owned())];
        let generated = generate(
            &sources,
            "tuffite",
            "org.tuffite.framework",
            "tuffite",
            Facade::Framework,
        )
        .unwrap();
        assert!(generated.javascript.contains("createFrameworkApi"));
        let shell_api_cc = generated
            .cpp_files
            .get("shell_api.cc")
            .expect("shell_api.cc");
        assert!(shell_api_cc.contains("\"tuffite.net.request\""));
        assert!(generated.javascript.contains("\"net.request\""));
        assert!(!generated.javascript.contains("0x"));
        assert!(!generated.javascript.contains("/tuffite-shell-api.js"));
        assert!(
            generated
                .declarations
                .contains("& import('../runtime').NativeBridge")
        );
    }

    #[test]
    fn rejects_reserved_framework_native_bridge_members() {
        for name in [
            "invoke",
            "networkTestUrl",
            "postMessage",
            "addEventListener",
            "removeEventListener",
        ] {
            let source = format!(
                "export namespace tuffite {{ export namespace {name} {{ export function call(): Promise<void>; }} }}"
            );
            let sources = vec![(Path::new("shell_api.d.ts").to_owned(), source)];
            let error = generate(
                &sources,
                "tuffite",
                "org.tuffite.framework",
                "tuffite",
                Facade::Framework,
            )
            .unwrap_err();
            assert!(
                error.contains(&format!("tuffite.{name} is reserved")),
                "{error}"
            );
        }
    }

    #[test]
    fn allows_native_bridge_member_names_on_application_facades() {
        let source = "export namespace api { export namespace channel { export function subscribe(): Promise<void>; } }";
        let sources = vec![(Path::new("api.d.ts").to_owned(), source.to_owned())];
        let generated = generate(
            &sources,
            "api",
            "com.example.app",
            "example",
            Facade::Application,
        )
        .unwrap();
        assert!(generated.javascript.contains("channel.subscribe"));
    }
}
