use crate::Error;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
/// Supported wire field kinds. Scalars use lowered names such as `u32` and `f64`.
pub enum Type {
    Scalar(String),
    Bool,
    String,
    Bytes,
    Named(String),
    Enum(String),
    Array(Box<Type>),
    Nullable(Box<Type>),
}
impl Type {
    #[must_use]
    pub fn size(&self) -> usize {
        match self {
            Self::Bool => 1,
            Self::Enum(_) => 4,
            Self::Nullable(inner) if inner.is_value() => inner.size(),
            Self::Scalar(value) => match value.as_str() {
                "i8" | "u8" => 1,
                "i16" | "u16" => 2,
                "i32" | "u32" | "f32" => 4,
                _ => 8,
            },
            _ => 8,
        }
    }
    #[must_use]
    pub fn is_value(&self) -> bool {
        matches!(self, Self::Bool | Self::Scalar(_) | Self::Enum(_))
    }
}
#[derive(Clone, Debug)]
/// A field in ordinal order, with its resolved byte offset and boolean bit index.
pub struct Field {
    pub name: String,
    pub ty: Type,
    pub(crate) source_span: (usize, usize),
    pub ordinal: u32,
    pub offset: usize,
    pub bit: u8,
    pub presence: Option<(usize, u8)>,
}
#[derive(Clone, Debug)]
/// A wire structure, including synthesized method request and response types.
pub struct Structure {
    pub name: String,
    pub fields: Vec<Field>,
    pub size: usize,
}
#[derive(Clone, Debug)]
/// A unary interface method referencing its request and response structures.
pub struct Method {
    pub interface: String,
    pub name: String,
    pub ordinal: u32,
    pub request: String,
    pub response: String,
}
/// A module's resolved declarations and field layouts, without SDK bindings.
#[derive(Debug, Default)]
pub struct Schema {
    pub module: String,
    pub imports: Vec<PathBuf>,
    pub(crate) import_spans: Vec<(usize, usize)>,
    pub(crate) declaration_spans: BTreeMap<String, (usize, usize)>,
    pub enums: BTreeMap<String, Vec<(String, i32)>>,
    pub structures: BTreeMap<String, Structure>,
    pub methods: Vec<Method>,
}

fn place(occupied: &mut Vec<u8>, size: usize, boolean: bool) -> (usize, u8) {
    let mut offset = 0;
    loop {
        if boolean && occupied.get(offset).copied().unwrap_or(0) != 255 {
            if offset == occupied.len() {
                occupied.push(0);
            }
            let bit = u8::try_from(occupied[offset].trailing_ones()).expect("bit fits u8");
            occupied[offset] |= 1 << bit;
            return (offset + 8, bit);
        }
        if !boolean
            && offset % size == 0
            && (offset..offset + size).all(|i| occupied.get(i).copied().unwrap_or(0) == 0)
        {
            occupied.resize(occupied.len().max(offset + size), 0);
            occupied[offset..offset + size].fill(255);
            return (offset + 8, 0);
        }
        offset += 1;
    }
}

pub(crate) fn layout(structure: &mut Structure) {
    // Port of Mojom's first-fit packing: later fields fill earlier holes.
    let mut occupied: Vec<u8> = Vec::new();

    for field in &mut structure.fields {
        let inner = match &field.ty {
            Type::Nullable(inner) if inner.is_value() => {
                field.presence = Some(place(&mut occupied, 1, true));
                inner.as_ref()
            }
            ty => ty,
        };
        (field.offset, field.bit) = place(&mut occupied, inner.size(), *inner == Type::Bool);
    }
    structure.size = (occupied.len() + 15) & !7;
}

fn check(
    schema: &Schema,
    ty: &Type,
    stack: &mut Vec<String>,
    remaining: &mut usize,
) -> Result<(), Error> {
    match ty {
        Type::Named(target) => check_types(schema, target, stack, remaining),
        Type::Array(inner) | Type::Nullable(inner) => check(schema, inner, stack, remaining),
        _ => Ok(()),
    }
}

pub(crate) fn check_types(
    schema: &Schema,
    name: &str,
    stack: &mut Vec<String>,
    remaining: &mut usize,
) -> Result<(), Error> {
    *remaining = remaining
        .checked_sub(1)
        .ok_or("Mojom schema expansion limit exceeded")?;
    if stack.len() >= 64 || stack.iter().any(|item| item == name) {
        return Err(format!("recursive or excessively deep Mojom type {name}").into());
    }
    let structure = schema
        .structures
        .get(name)
        .ok_or_else(|| format!("unknown Mojom type {name}"))?;
    stack.push(name.into());

    for field in &structure.fields {
        check(schema, &field.ty, stack, remaining)
            .map_err(|error| error.in_declaration(name, field.source_span))?;
    }
    stack.pop();
    Ok(())
}
