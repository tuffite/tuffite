//! Source-distributed frontend for the handle-free Mojom v0 data profile.
//! Loads and validates contracts and computes wire layouts; SDK bindings live in Tuffite.
mod compiler;
mod diagnostic;
mod lexer;
mod loader;
mod parser;
mod schema;

#[cfg(test)]
mod tests;

pub use compiler::compile;
pub use diagnostic::Error;
pub use loader::read_sources;
pub use schema::{Field, Method, Schema, Structure, Type};
