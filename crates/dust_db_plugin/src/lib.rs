#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Database plugin for SQLx-validated sqlite3 generation."]

/// DB plugin implementation and registration surface.
mod plugin;

pub use plugin::{
    DATABASE_PLUGIN_NAME, DbPlugin, register_plugin, register_row_plugin,
    register_validating_plugin,
};
