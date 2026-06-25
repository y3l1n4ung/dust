#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Built-in Dust plugin providing JSON serialization and deserialization support for Dart classes and enums."]

/// Coordinates serde code emission for a library.
mod emit;
/// Renders class JSON helpers and mixins.
mod emit_class;
/// Renders enum JSON helpers.
mod emit_enum;
/// Renders sealed class JSON dispatch helpers.
mod emit_sealed;
/// Shared formatting helpers for serde emission.
mod emit_support;
/// Renders generated sealed variant classes.
mod emit_variant_class;
/// Plugin registration and Dust plugin implementation.
mod plugin;
/// Validates serde-compatible model shapes.
mod validate;
/// Low-level helpers for generating JSON-related Dart expressions.
pub mod writer;
/// Renders JSON encode and decode expressions.
mod writer_expr;
/// Renders model-level constructor and key helpers.
mod writer_model;
/// Renders type and receiver helpers.
mod writer_type;

pub use plugin::{SerdePlugin, register_plugin};
