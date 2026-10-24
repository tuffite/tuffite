use crate::Error;
use crate::lexer::{Token, is_identifier, lex};
use crate::schema::{Field, Method, Schema, Structure, Type};

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}
impl Parser {
    fn error(&self, message: &str) -> Error {
        let token = &self.tokens[self.at];
        let error = Error::at(
            format!("{message}, found {:?}", token.text),
            token.offset,
            token.length,
        );
        if let Some(expected) = message.strip_prefix("expected ") {
            error.with_help(format!(
                "Insert or replace the highlighted token with {expected}."
            ))
        } else {
            error
        }
    }
    fn eat(&mut self, value: &str) -> bool {
        if self.tokens[self.at].text == value {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, value: &str) -> Result<(), Error> {
        if self.eat(value) {
            Ok(())
        } else {
            Err(self.error(&format!("expected {value:?}")))
        }
    }
    fn name(&mut self) -> Result<String, Error> {
        let value = self.tokens[self.at].text.clone();
        if !is_identifier(&value) || value.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(self.error("expected identifier"));
        }
        self.at += 1;
        Ok(value)
    }
    fn ty(&mut self) -> Result<Type, Error> {
        self.ty_at_depth(0)
    }
    fn ty_at_depth(&mut self, depth: usize) -> Result<Type, Error> {
        if depth >= 64 {
            return Err(self.error("Mojom type nesting limit exceeded"));
        }
        let value = self.name()?;
        let ty = match value.as_str() {
            "bool" => Type::Bool,
            "string" => Type::String,
            "int8" => Type::Scalar("i8".into()),
            "uint8" => Type::Scalar("u8".into()),
            "int16" => Type::Scalar("i16".into()),
            "uint16" => Type::Scalar("u16".into()),
            "int32" => Type::Scalar("i32".into()),
            "uint32" => Type::Scalar("u32".into()),
            "int64" => Type::Scalar("i64".into()),
            "uint64" => Type::Scalar("u64".into()),
            "float" => Type::Scalar("f32".into()),
            "double" => Type::Scalar("f64".into()),
            "array" => {
                self.expect("<")?;
                let inner = self.ty_at_depth(depth + 1)?;
                self.expect(">")?;
                if inner == Type::Scalar("u8".into()) {
                    Type::Bytes
                } else {
                    Type::Array(Box::new(inner))
                }
            }
            "handle"
            | "pending_remote"
            | "pending_receiver"
            | "pending_associated_remote"
            | "pending_associated_receiver" => {
                return Err(
                    self.error("resource types are not supported by the Mojom data profile")
                );
            }
            _ => Type::Named(value),
        };
        Ok(if self.eat("?") {
            Type::Nullable(Box::new(ty))
        } else {
            ty
        })
    }
    fn ordinal(&mut self, default: u32) -> Result<u32, Error> {
        if !self.eat("@") {
            return Ok(default);
        }
        let ordinal = self.tokens[self.at]
            .text
            .parse()
            .map_err(|_| self.error("expected u32 ordinal"))?;
        self.at += 1;
        Ok(ordinal)
    }
    fn fields(&mut self, end: &str, separator: &str) -> Result<Vec<Field>, Error> {
        let mut fields = Vec::new();
        while !self.eat(end) {
            let start = self.tokens[self.at].offset;
            let ty = self.ty()?;
            let type_end = self.tokens[self.at - 1].offset + self.tokens[self.at - 1].length;
            let name = self.name()?;
            let ordinal = self
                .ordinal(u32::try_from(fields.len()).map_err(|_| self.error("too many fields"))?)?;
            if fields.iter().any(|field: &Field| field.name == name) {
                return Err(self.error("duplicate field"));
            }
            fields.push(Field {
                name,
                ty,
                source_span: (start, type_end - start),
                ordinal,
                offset: 0,
                bit: 0,
                presence: None,
            });
            if separator == ";" {
                self.expect(separator)?;
            } else if !self.eat(separator) {
                self.expect(end)?;
                break;
            }
        }
        if fields.len() > 1024 {
            return Err(self.error("too many fields"));
        }
        fields.sort_by_key(|field| field.ordinal);
        for (ordinal, field) in fields.iter().enumerate() {
            if field.ordinal as usize != ordinal {
                return Err(
                    self.error("v0 fields require unique contiguous ordinals starting at zero")
                );
            }
        }
        Ok(fields)
    }
    fn enum_declaration(&mut self, schema: &mut Schema) -> Result<(), Error> {
        let token = &self.tokens[self.at];
        let span = (token.offset, token.length);
        let name = self.name()?;
        self.expect("{")?;
        let mut values = Vec::new();
        let mut next = 0i32;
        while !self.eat("}") {
            let member = self.name()?;
            if self.eat("=") {
                next = self.tokens[self.at]
                    .text
                    .parse()
                    .map_err(|_| self.error("expected i32 enum value"))?;
                self.at += 1;
            }
            if values.iter().any(|(n, _)| n == &member) {
                return Err(self.error("duplicate enum member"));
            }
            values.push((member, next));
            next = next
                .checked_add(1)
                .ok_or_else(|| self.error("enum overflow"))?;
            if !self.eat(",") {
                self.expect("}")?;
                break;
            }
        }
        self.expect(";")?;
        schema.declaration_spans.insert(name.clone(), span);
        if values.is_empty() || schema.enums.insert(name, values).is_some() {
            return Err(self.error("empty or duplicate enum"));
        }
        Ok(())
    }
    fn struct_declaration(&mut self, schema: &mut Schema) -> Result<(), Error> {
        let token = &self.tokens[self.at];
        let span = (token.offset, token.length);
        let name = self.name()?;
        self.expect("{")?;
        let fields = self.fields("}", ";")?;
        self.expect(";")?;
        insert_structure(schema, &name, fields, span)?;
        Ok(())
    }
    fn interface_declaration(&mut self, schema: &mut Schema) -> Result<(), Error> {
        let interface = self.name()?;
        self.expect("{")?;
        let mut next = 0;
        while !self.eat("}") {
            let token = &self.tokens[self.at];
            let span = (token.offset, token.length);
            let name = self.name()?;
            let ordinal = self.ordinal(next)?;
            next = ordinal
                .checked_add(1)
                .ok_or_else(|| self.error("method ordinal overflow"))?;
            self.expect("(")?;
            let fields = self.fields(")", ",")?;
            self.expect("=")?;
            self.expect(">")?;
            self.expect("(")?;
            let results = self.fields(")", ",")?;
            if results.len() > 1 {
                return Err(self.error("v0 methods support at most one result"));
            }
            self.expect(";")?;
            let request = format!("{interface}{name}Request");
            let response = format!("{interface}{name}Response");
            insert_structure(schema, &request, fields, span)?;
            insert_structure(schema, &response, results, span)?;
            if schema.methods.iter().any(|method| {
                method.interface == interface && (method.ordinal == ordinal || method.name == name)
            }) {
                return Err(self.error("duplicate method or ordinal"));
            }
            schema.methods.push(Method {
                interface: interface.clone(),
                name,
                ordinal,
                request,
                response,
            });
        }
        self.expect(";")?;
        Ok(())
    }
    fn parse(mut self) -> Result<Schema, Error> {
        let mut schema = Schema::default();
        self.expect("module")?;
        schema.module = self.name()?;
        while self.eat(".") {
            schema.module.push('.');
            schema.module.push_str(&self.name()?);
        }
        self.expect(";")?;
        while self.eat("import") {
            let path: String = serde_json::from_str(&self.tokens[self.at].text)
                .map_err(|_| self.error("expected import string"))?;
            let token = &self.tokens[self.at];
            schema.import_spans.push((token.offset, token.length));
            schema.imports.push(path.into());
            self.at += 1;
            self.expect(";")?;
        }
        while !self.eat("<eof>") {
            if self.eat("enum") {
                self.enum_declaration(&mut schema)?;
            } else if self.eat("struct") {
                self.struct_declaration(&mut schema)?;
            } else if self.eat("interface") {
                self.interface_declaration(&mut schema)?;
            } else {
                return Err(self.error(
                    "unsupported Mojom declaration (v0 supports struct and unary interface)",
                ));
            }
        }
        Ok(schema)
    }
}

fn insert_structure(
    schema: &mut Schema,
    name: &str,
    fields: Vec<Field>,
    span: (usize, usize),
) -> Result<(), Error> {
    if schema
        .structures
        .insert(
            name.to_owned(),
            Structure {
                name: name.to_owned(),
                fields,
                size: 0,
            },
        )
        .is_some()
    {
        return Err(Error::at(format!("duplicate type {name}"), span.0, span.1));
    }
    schema.declaration_spans.insert(name.to_owned(), span);
    Ok(())
}

pub(crate) fn parse(source: &str) -> Result<Schema, Error> {
    Parser {
        tokens: lex(source)?,
        at: 0,
    }
    .parse()
    .map_err(|error| error.with_source(std::path::Path::new("<input>"), source))
}
