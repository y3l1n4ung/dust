use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_plugin_api::{
    DustPlugin, MetadataOutput, PluginContext, PluginContribution, PluginExecutionMode,
};

/// Shared annotation names and claimed symbol lists.
mod constants;
/// Renders generated DB, DAO, and row-mapping Dart code.
mod emit;
/// Shared migration discovery rules.
mod migrations;
/// Internal DB plugin model used by parsing, validation, and emission.
mod model;
/// Parses DB annotations and query call sites.
mod parse;
/// SQL string rewriting helpers.
mod sql;
/// Validates DB annotations and SQL query metadata.
mod validate;

use self::constants::{
    CLAIMED_DATABASE_CONFIG_SYMBOLS, CLAIMED_ROW_CONFIG_SYMBOLS, SUPPORTED_DATABASE_ANNOTATIONS,
    SUPPORTED_ROW_ANNOTATIONS,
};
use self::emit::emit_db_library;
use self::validate::validate_db_library;

/// Stable name used to select the Database validating-plugin profile.
pub const DATABASE_PLUGIN_NAME: &str = "Database";

/// Runtime options for the Database plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DbPluginOptions {
    /// Shared validation and metadata execution policy.
    execution: PluginExecutionMode,
    /// Whether database generation and SQL validation are enabled.
    databases: bool,
}

impl Default for DbPluginOptions {
    fn default() -> Self {
        Self {
            execution: PluginExecutionMode::online(MetadataOutput::Write),
            databases: true,
        }
    }
}

/// Dust plugin for SQLx-validated sqlite3 database generation.
pub struct DbPlugin {
    /// Runtime validation and emission options for this plugin instance.
    options: DbPluginOptions,
}

impl DbPlugin {
    /// Creates a DB plugin with default online validation behavior.
    pub const fn new() -> Self {
        Self {
            options: DbPluginOptions {
                execution: PluginExecutionMode::online(MetadataOutput::Write),
                databases: true,
            },
        }
    }

    /// Creates a DB plugin with explicit options.
    const fn with_options(options: DbPluginOptions) -> Self {
        Self { options }
    }
}

impl Default for DbPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates the default Database plugin.
pub fn register_plugin() -> DbPlugin {
    DbPlugin::new()
}

/// Creates the Database plugin for a shared validating-plugin execution mode.
pub fn register_validating_plugin(execution: PluginExecutionMode) -> DbPlugin {
    DbPlugin::with_options(DbPluginOptions {
        execution,
        databases: true,
    })
}

/// Creates the Database plugin in row-mapper-only mode.
pub fn register_row_plugin() -> DbPlugin {
    DbPlugin::with_options(DbPluginOptions {
        execution: PluginExecutionMode::online(MetadataOutput::ReadOnly),
        databases: false,
    })
}

impl DustPlugin for DbPlugin {
    fn plugin_name(&self) -> &'static str {
        DATABASE_PLUGIN_NAME
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

    fn validate(&self, library: &DartFileIr) -> Vec<Diagnostic> {
        validate_db_library(library, self.options)
    }

    fn generate(
        &self,
        library: &DartFileIr,
        _context: &PluginContext<'_>,
    ) -> Vec<PluginContribution> {
        vec![emit_db_library(library, self.options)]
    }
}

#[cfg(test)]
mod tests {
    use dust_plugin_api::{DustPlugin, MetadataOutput, PluginExecutionMode, SymbolPlan};

    use super::{
        DbPlugin, DbPluginOptions, register_plugin, register_row_plugin, register_validating_plugin,
    };

    fn empty_library() -> dust_ir::DartFileIr {
        dust_ir::DartFileIr {
            package_root: ".".to_owned(),
            package_name: "db_test".to_owned(),
            source_path: "lib/db.dart".to_owned(),
            output_path: "lib/db.g.dart".to_owned(),
            imports: Vec::new(),
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span: dust_ir::SpanIr::new(
                dust_text::FileId::new(1),
                dust_text::TextRange::new(0_u32, 1_u32),
            ),
            classes: Vec::new(),
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        }
    }

    #[test]
    fn plugin_options_default_to_online_database_mode() {
        assert_eq!(
            DbPluginOptions::default(),
            DbPluginOptions {
                execution: PluginExecutionMode::online(MetadataOutput::Write),
                databases: true,
            }
        );
    }

    #[test]
    fn database_and_row_modes_claim_different_symbols() {
        let database = register_plugin();
        assert_eq!(database.plugin_name(), "Database");
        assert!(
            database
                .claimed_configs()
                .contains(&"dust_dart::SqlxDatabase")
        );
        assert!(database.claimed_configs().contains(&"dust_dart::SqlxDao"));
        assert!(database.supported_annotations().contains(&"SqlxDatabase"));
        assert!(database.supported_annotations().contains(&"SqlxDao"));

        let row_only = register_row_plugin();
        assert!(!row_only.claimed_configs().contains(&"dust_dart::SqlxDao"));
        assert!(row_only.claimed_configs().contains(&"dust_dart::Sqlx"));
        assert_eq!(row_only.supported_annotations(), ["FromRow"]);
    }

    #[test]
    fn explicit_options_round_trip_through_plugin_contract() {
        let plugin =
            register_validating_plugin(PluginExecutionMode::offline(MetadataOutput::ReadOnly));
        let library = empty_library();

        assert!(plugin.validate(&library).is_empty());
        assert_eq!(
            plugin
                .generate(
                    &library,
                    &dust_plugin_api::PluginContext {
                        symbol_plan: &SymbolPlan::default()
                    }
                )
                .into_iter()
                .next()
                .expect("plugin must generate one contribution"),
            dust_plugin_api::PluginContribution::default()
        );

        let custom = DbPlugin::with_options(DbPluginOptions {
            execution: PluginExecutionMode::offline(MetadataOutput::ReadOnly),
            databases: false,
        });
        assert_eq!(custom.supported_annotations(), ["FromRow"]);
    }
}
