#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Built-in Dust plugin providing JSON serialization and deserialization support for Dart classes and enums."]

mod emit;
mod emit_class;
mod emit_enum;
mod emit_support;
mod plugin;
mod validate;
/// Low-level helpers for generating JSON-related Dart expressions.
pub mod writer;
mod writer_expr;
mod writer_model;
mod writer_type;

pub use plugin::{SerdePlugin, register_plugin};
