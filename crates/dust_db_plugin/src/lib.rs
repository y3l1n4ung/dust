#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Dust DB plugin for SQLx-validated sqflite generation."]

mod plugin;

pub use plugin::{DbPlugin, register_plugin, register_plugin_with_options};
