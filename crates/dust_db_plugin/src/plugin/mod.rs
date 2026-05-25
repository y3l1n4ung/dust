use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;
use dust_plugin_api::{DustPlugin, PluginContribution, SymbolPlan};

mod constants;
mod emit;
mod model;
mod parse;
mod validate;

use self::constants::{CLAIMED_CONFIG_SYMBOLS, SUPPORTED_ANNOTATIONS};
use self::emit::emit_db_library;
use self::validate::validate_db_library;

/// Runtime options for the Dust DB plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DbPluginOptions {
    /// Whether SQL validation must use cached metadata only.
    pub offline: bool,
}

/// Dust plugin for SQLx-validated sqflite repository generation.
pub struct DbPlugin {
    options: DbPluginOptions,
}

impl DbPlugin {
    /// Creates a DB plugin with default online validation behavior.
    pub const fn new() -> Self {
        Self {
            options: DbPluginOptions { offline: false },
        }
    }

    /// Creates a DB plugin with explicit options.
    pub const fn with_options(options: DbPluginOptions) -> Self {
        Self { options }
    }
}

impl Default for DbPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates the default Dust DB plugin.
pub fn register_plugin() -> DbPlugin {
    DbPlugin::new()
}

/// Creates the Dust DB plugin with explicit options.
pub fn register_plugin_with_options(offline: bool) -> DbPlugin {
    DbPlugin::with_options(DbPluginOptions { offline })
}

impl DustPlugin for DbPlugin {
    fn plugin_name(&self) -> &'static str {
        "DustDb"
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        CLAIMED_CONFIG_SYMBOLS
    }

    fn supported_annotations(&self) -> &'static [&'static str] {
        SUPPORTED_ANNOTATIONS
    }

    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        validate_db_library(library, self.options)
    }

    fn emit(&self, library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        emit_db_library(library)
    }
}
