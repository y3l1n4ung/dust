#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "First-party derive plugin for ToString, Eq, and CopyWith."]

mod analysis;
mod emit;
mod features;
mod plugin;
mod validate;

pub use plugin::{DerivePlugin, register_plugin};
