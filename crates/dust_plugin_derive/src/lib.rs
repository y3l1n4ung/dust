#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "First-party derive plugin for ToString, Eq, and CopyWith."]

/// Workspace analysis for copyWith-capable types.
mod analysis;
/// Orchestrates derive feature emission.
mod emit;
/// Individual derive feature implementations.
mod features;
/// Plugin registration and Dust plugin implementation.
mod plugin;
/// Derive feature validation entrypoint.
mod validate;

pub use plugin::{DerivePlugin, register_plugin};
