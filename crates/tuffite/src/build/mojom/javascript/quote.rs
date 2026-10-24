//! Typed AST holes in parsed syntax templates, not textual substitution.
//! Templates are controlled compiler sources; this is not a hygienic macro system.
use oxc_allocator::{CloneIn, Vec as ArenaVec};
use oxc_ast::{
    AstBuilder,
    ast::{
        BindingIdentifier, BindingPattern, Expression, FormalParameter, FormalParameters,
        IdentifierName, IdentifierReference, Program, PropertyKey, TSEnumBody, TSEnumMember,
        TSEnumMemberName, TSSignature, TSType, TSTypeName,
    },
};
use oxc_ast_visit::{VisitMut, walk_mut};
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions, IndentChar};
use oxc_parser::Parser;
use oxc_span::{Ident, SourceType};
use std::collections::{BTreeMap, BTreeSet};

pub(super) enum Hole<'a> {
    Ident(Ident<'a>),
    Expr(Expression<'a>),
    Type(TSType<'a>),
    Pattern(BindingPattern<'a>),
    Members(ArenaVec<'a, TSSignature<'a>>),
    EnumMembers(ArenaVec<'a, TSEnumMember<'a>>),
    Parameters(ArenaVec<'a, FormalParameter<'a>>),
}

pub(super) struct Quote<'a> {
    ast: AstBuilder<'a>,
    holes: BTreeMap<&'static str, Hole<'a>>,
    used: BTreeSet<&'static str>,
    errors: Vec<String>,
}

impl<'a> Quote<'a> {
    pub fn new(ast: AstBuilder<'a>) -> Self {
        Self {
            ast,
            holes: BTreeMap::new(),
            used: BTreeSet::new(),
            errors: Vec::new(),
        }
    }
    pub fn bind(mut self, name: &'static str, value: Hole<'a>) -> Self {
        if let Hole::Ident(value) = &value
            && !crate::build::is_identifier(value.as_str())
        {
            self.errors
                .push(format!("invalid AST identifier {value:?}"));
        }
        if self.holes.insert(name, value).is_some() {
            self.errors.push(format!("duplicate AST hole {name}"));
        }
        self
    }
    pub fn parse(mut self, source: &'static str, filename: &str) -> Result<Program<'a>, String> {
        let mut program = parse(self.ast, source, filename)?;
        self.visit_program(&mut program);
        for name in self.holes.keys() {
            if !self.used.contains(name) {
                self.errors.push(format!("unused AST hole {name}"));
            }
        }
        if !self.errors.is_empty() {
            return Err(self.errors.join("; "));
        }
        Ok(program)
    }
    fn rename(&mut self, name: &mut Ident<'a>) {
        if let Some((key, Hole::Ident(value))) = self.holes.get_key_value(name.as_str()) {
            self.used.insert(key);
            *name = *value;
        } else if name.starts_with('$') {
            self.errors
                .push(format!("unbound or wrong-kind AST hole {name}"));
        }
    }
}

impl<'a> VisitMut<'a> for Quote<'a> {
    fn visit_identifier_name(&mut self, node: &mut IdentifierName<'a>) {
        self.rename(&mut node.name);
    }
    fn visit_identifier_reference(&mut self, node: &mut IdentifierReference<'a>) {
        self.rename(&mut node.name);
    }
    fn visit_binding_identifier(&mut self, node: &mut BindingIdentifier<'a>) {
        self.rename(&mut node.name);
    }
    fn visit_expression(&mut self, node: &mut Expression<'a>) {
        if let Expression::Identifier(id) = node
            && let Some((key, Hole::Expr(value))) = self.holes.get_key_value(id.name.as_str())
        {
            self.used.insert(key);
            *node = value.clone_in(self.ast.allocator);
            return;
        }
        walk_mut::walk_expression(self, node);
    }
    fn visit_ts_type(&mut self, node: &mut TSType<'a>) {
        if let TSType::TSTypeReference(reference) = node
            && let TSTypeName::IdentifierReference(id) = &reference.type_name
            && let Some((key, Hole::Type(value))) = self.holes.get_key_value(id.name.as_str())
        {
            if reference.type_arguments.is_some() {
                self.errors.push(format!(
                    "type hole {key} cannot have template type arguments"
                ));
                return;
            }
            self.used.insert(key);
            *node = value.clone_in(self.ast.allocator);
            return;
        }
        walk_mut::walk_ts_type(self, node);
    }
    fn visit_binding_pattern(&mut self, node: &mut BindingPattern<'a>) {
        if let BindingPattern::BindingIdentifier(id) = node
            && let Some((key, Hole::Pattern(value))) = self.holes.get_key_value(id.name.as_str())
        {
            self.used.insert(key);
            *node = value.clone_in(self.ast.allocator);
            return;
        }
        walk_mut::walk_binding_pattern(self, node);
    }
    fn visit_ts_signatures(&mut self, nodes: &mut ArenaVec<'a, TSSignature<'a>>) {
        let previous = std::mem::replace(nodes, self.ast.vec());
        for mut node in previous {
            if let TSSignature::TSPropertySignature(property) = &node
                && let PropertyKey::StaticIdentifier(id) = &property.key
                && let Some((key, Hole::Members(values))) =
                    self.holes.get_key_value(id.name.as_str())
            {
                self.used.insert(key);
                nodes.extend(
                    values
                        .iter()
                        .map(|value| value.clone_in(self.ast.allocator)),
                );
                continue;
            }
            self.visit_ts_signature(&mut node);
            nodes.push(node);
        }
    }
    fn visit_ts_enum_body(&mut self, node: &mut TSEnumBody<'a>) {
        if node.members.len() == 1
            && let TSEnumMemberName::Identifier(id) = &node.members[0].id
            && let Some((key, Hole::EnumMembers(values))) =
                self.holes.get_key_value(id.name.as_str())
        {
            self.used.insert(key);
            node.members = self.ast.vec_from_iter(
                values
                    .iter()
                    .map(|value| value.clone_in(self.ast.allocator)),
            );
            return;
        }
        walk_mut::walk_ts_enum_body(self, node);
    }
    fn visit_formal_parameters(&mut self, node: &mut FormalParameters<'a>) {
        let previous = std::mem::replace(&mut node.items, self.ast.vec());
        for mut parameter in previous {
            if let BindingPattern::BindingIdentifier(id) = &parameter.pattern
                && let Some((key, Hole::Parameters(values))) =
                    self.holes.get_key_value(id.name.as_str())
            {
                self.used.insert(key);
                node.items.extend(
                    values
                        .iter()
                        .map(|value| value.clone_in(self.ast.allocator)),
                );
                continue;
            }
            self.visit_formal_parameter(&mut parameter);
            node.items.push(parameter);
        }
        if let Some(rest) = &mut node.rest {
            self.visit_formal_parameter_rest(rest);
        }
    }
}

pub(super) fn parse<'a>(
    ast: AstBuilder<'a>,
    source: &'a str,
    filename: &str,
) -> Result<Program<'a>, String> {
    let parsed = Parser::new(
        ast.allocator,
        source,
        SourceType::from_path(filename).map_err(|e| e.to_string())?,
    )
    .parse();
    if let Some(error) = parsed.errors.first() {
        return Err(format!("invalid {filename}: {error}"));
    }
    Ok(parsed.program)
}

pub(super) fn emit(program: &Program<'_>, filename: &str) -> Result<String, String> {
    let code = Codegen::new()
        .with_options(CodegenOptions {
            single_quote: true,
            comments: CommentOptions::disabled(),
            indent_char: IndentChar::Space,
            indent_width: 2,
            ..CodegenOptions::default()
        })
        .build(program)
        .code;
    // Reparse the printed result to catch invalid names in binding/type contexts.
    let allocator = oxc_allocator::Allocator::default();
    parse(AstBuilder::new(&allocator), &code, filename)?;
    Ok(format!("// Generated from Mojom v0. Do not edit.\n{code}"))
}
