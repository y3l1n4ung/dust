#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Built-in Dust plugin providing JSON serialization and deserialization support for Dart classes and enums."]

mod emit;
mod plugin;
mod validate;
/// Low-level helpers for generating JSON-related Dart expressions.
pub mod writer;

pub use plugin::{SerdePlugin, register_plugin};
