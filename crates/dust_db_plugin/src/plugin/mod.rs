use dust_diagnostics::Diagnostic;
use dust_ir::LibraryIr;
use dust_plugin_api::{DustPlugin, PluginContribution, SymbolPlan};

mod constants;
mod emit;
mod model;
mod parse;
mod validate;

use self::constants::{
    CLAIMED_DATABASE_CONFIG_SYMBOLS, CLAIMED_ROW_CONFIG_SYMBOLS, SUPPORTED_DATABASE_ANNOTATIONS,
    SUPPORTED_ROW_ANNOTATIONS,
};
use self::emit::emit_db_library;
use self::validate::validate_db_library;

/// Runtime options for the Dust DB plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbPluginOptions {
    /// Whether SQL validation must use cached metadata only.
    pub offline: bool,
    /// Whether successful online validation should update query metadata cache.
    pub write_metadata: bool,
    /// Whether database generation and SQL validation are enabled.
    pub databases: bool,
}

impl Default for DbPluginOptions {
    fn default() -> Self {
        Self {
            offline: false,
            write_metadata: true,
            databases: true,
        }
    }
}

/// Dust plugin for SQLx-validated sqlite3 database generation.
pub struct DbPlugin {
    options: DbPluginOptions,
}

impl DbPlugin {
    /// Creates a DB plugin with default online validation behavior.
    pub const fn new() -> Self {
        Self {
            options: DbPluginOptions {
                offline: false,
                write_metadata: true,
                databases: true,
            },
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
pub fn register_plugin_with_options(offline: bool, write_metadata: bool) -> DbPlugin {
    DbPlugin::with_options(DbPluginOptions {
        offline,
        write_metadata,
        databases: true,
    })
}

/// Creates the Dust DB plugin in row-mapper-only mode.
pub fn register_row_plugin() -> DbPlugin {
    DbPlugin::with_options(DbPluginOptions {
        offline: false,
        write_metadata: false,
        databases: false,
    })
}

impl DustPlugin for DbPlugin {
    fn plugin_name(&self) -> &'static str {
        "DustDb"
    }

    fn claimed_traits(&self) -> &'static [&'static str] {
        self::constants::CLAIMED_TRAIT_SYMBOLS
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        if self.options.databases {
            CLAIMED_DATABASE_CONFIG_SYMBOLS
        } else {
            CLAIMED_ROW_CONFIG_SYMBOLS
        }
    }

    fn supported_annotations(&self) -> &'static [&'static str] {
        if self.options.databases {
            SUPPORTED_DATABASE_ANNOTATIONS
        } else {
            SUPPORTED_ROW_ANNOTATIONS
        }
    }

    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        validate_db_library(library, self.options)
    }

    fn emit(&self, library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        emit_db_library(library, self.options)
    }
}

#[cfg(test)]
mod tests {
    use dust_plugin_api::{DustPlugin, SymbolPlan};

    use super::{
        DbPlugin, DbPluginOptions, register_plugin, register_plugin_with_options,
        register_row_plugin,
    };

    fn empty_library() -> dust_ir::LibraryIr {
        dust_ir::LibraryIr {
            package_root: ".".to_owned(),
            package_name: "db_test".to_owned(),
            source_path: "lib/db.dart".to_owned(),
            output_path: "lib/db.g.dart".to_owned(),
            imports: Vec::new(),
            span: dust_ir::SpanIr::new(
                dust_text::FileId::new(1),
                dust_text::TextRange::new(0_u32, 1_u32),
            ),
            classes: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        }
    }

    #[test]
    fn plugin_options_default_to_online_database_mode() {
        assert_eq!(
            DbPluginOptions::default(),
            DbPluginOptions {
                offline: false,
                write_metadata: true,
                databases: true,
            }
        );
    }

    #[test]
    fn database_and_row_modes_claim_different_symbols() {
        let database = register_plugin();
        assert_eq!(database.plugin_name(), "DustDb");
        assert!(
            database
                .claimed_configs()
                .contains(&"dust_db_annotation::SqlxDatabase")
        );
        assert!(
            database
                .claimed_configs()
                .contains(&"dust_db_annotation::SqlxDao")
        );
        assert!(database.supported_annotations().contains(&"SqlxDatabase"));
        assert!(database.supported_annotations().contains(&"SqlxDao"));

        let row_only = register_row_plugin();
        assert!(
            !row_only
                .claimed_configs()
                .contains(&"dust_db_annotation::SqlxDao")
        );
        assert!(
            row_only
                .claimed_configs()
                .contains(&"dust_db_annotation::Sqlx")
        );
        assert_eq!(row_only.supported_annotations(), ["FromRow"]);
    }

    #[test]
    fn explicit_options_round_trip_through_plugin_contract() {
        let plugin = register_plugin_with_options(true, false);
        let library = empty_library();

        assert!(plugin.validate(&library).is_empty());
        assert_eq!(
            plugin.emit(&library, &SymbolPlan::default()),
            dust_plugin_api::PluginContribution::default()
        );

        let custom = DbPlugin::with_options(DbPluginOptions {
            offline: true,
            write_metadata: false,
            databases: false,
        });
        assert_eq!(custom.supported_annotations(), ["FromRow"]);
    }
}
