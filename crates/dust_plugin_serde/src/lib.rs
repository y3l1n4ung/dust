#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "First-party serde plugin for Serialize, Deserialize, and SerDe."]

mod emit;
mod plugin;
mod validate;
mod writer;

pub use plugin::{SerdePlugin, register_plugin};
