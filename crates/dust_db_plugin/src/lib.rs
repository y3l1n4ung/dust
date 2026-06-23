#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Dust DB plugin for SQLx-validated sqlite3 generation."]

/// DB plugin implementation and registration surface.
mod plugin;

pub use plugin::{DbPlugin, register_plugin, register_plugin_with_options, register_row_plugin};
