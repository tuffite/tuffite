//! Metadata values become literal AST nodes, without generating source fragments.
use oxc_ast::{
    AstBuilder,
    ast::{Expression, PropertyKind},
};
use oxc_span::SPAN;
use oxc_syntax::{number::NumberBase, operator::UnaryOperator};
use serde_json::Value;

pub(super) fn literal<'a>(ast: AstBuilder<'a>, value: &Value) -> Result<Expression<'a>, String> {
    Ok(match value {
        Value::Null => ast.expression_null_literal(SPAN),
        Value::Bool(value) => ast.expression_boolean_literal(SPAN, *value),
        Value::String(value) => ast.expression_string_literal(SPAN, ast.atom(value), None),
        Value::Number(value) => {
            let number = value
                .as_f64()
                .ok_or("metadata number is not representable in JS")?;
            let literal =
                ast.expression_numeric_literal(SPAN, number.abs(), None, NumberBase::Decimal);
            if number.is_sign_negative() {
                ast.expression_unary(SPAN, UnaryOperator::UnaryNegation, literal)
            } else {
                literal
            }
        }
        Value::Array(values) => {
            let elements = values
                .iter()
                .map(|value| Ok(literal(ast, value)?.into()))
                .collect::<Result<Vec<_>, String>>()?;
            ast.expression_array(SPAN, ast.vec_from_iter(elements))
        }
        Value::Object(values) => {
            let properties = values
                .iter()
                .map(|(name, value)| {
                    Ok(ast.object_property_kind_object_property(
                        SPAN,
                        PropertyKind::Init,
                        ast.expression_string_literal(SPAN, ast.atom(name), None)
                            .into(),
                        literal(ast, value)?,
                        false,
                        false,
                        // A literal __proto__ key is a JS prototype setter unless computed.
                        name == "__proto__",
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            ast.expression_object(SPAN, ast.vec_from_iter(properties))
        }
    })
}
