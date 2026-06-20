use dust_db_plugin::{
    register_plugin_with_options as register_db_plugin_with_options,
    register_row_plugin as register_db_row_plugin,
};
use dust_diagnostics::Diagnostic;
use dust_http_client_plugin::register_plugin as register_http_client_plugin;
use dust_plugin_api::{DustPlugin, PluginContribution, PluginRegistry, SymbolPlan};
use dust_plugin_derive::register_plugin as register_derive_plugin;
use dust_plugin_serde::register_plugin as register_serde_plugin;
use dust_route_plugin::register_plugin as register_route_plugin;
use dust_state_plugin::register_plugin as register_state_plugin;

/// Selects which plugin registry should run for a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistrySelection {
    /// Full Dust registry for normal build and check commands.
    All,
    /// DB-only registry for SQL metadata generation.
    DbOnly {
        /// Whether DB analysis should run without connecting to a database.
        offline: bool,
        /// Whether DB metadata sidecar files should be written.
        write_metadata: bool,
    },
}

impl RegistrySelection {
    /// Builds registry selection for a writing build request.
    pub(crate) fn for_build(value: crate::request::DbRequestOptions) -> Self {
        Self::from_db_request(value, true)
    }

    /// Builds registry selection for a non-writing check request.
    pub(crate) fn for_check(value: crate::request::DbRequestOptions) -> Self {
        Self::from_db_request(value, false)
    }

    /// Converts shared DB request flags into a registry mode.
    fn from_db_request(value: crate::request::DbRequestOptions, write_metadata: bool) -> Self {
        if value.only_db {
            return Self::DbOnly {
                offline: value.offline,
                write_metadata,
            };
        }
        Self::All
    }

    /// Returns the cache salt that distinguishes registry modes.
    pub(crate) fn cache_salt(self) -> &'static str {
        match self {
            Self::All => "registry:all",
            Self::DbOnly {
                offline: false,
                write_metadata: true,
            } => "registry:db-only:online:write-metadata",
            Self::DbOnly {
                offline: false,
                write_metadata: false,
            } => "registry:db-only:online:no-metadata",
            Self::DbOnly {
                offline: true,
                write_metadata: true,
            } => "registry:db-only:offline:write-metadata",
            Self::DbOnly {
                offline: true,
                write_metadata: false,
            } => "registry:db-only:offline:no-metadata",
        }
    }
}
/// Builds the full default plugin registry.
pub(crate) fn default_registry() -> PluginRegistry {
    registry_for_selection(RegistrySelection::All)
}

/// Builds the plugin registry requested by a command.
pub(crate) fn registry_for_selection(selection: RegistrySelection) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    match selection {
        RegistrySelection::All => {}
        RegistrySelection::DbOnly {
            offline,
            write_metadata,
        } => {
            registry
                .register(Box::new(DbModePassThroughPlugin))
                .expect("db pass-through plugin symbol ownership must be valid");
            registry
                .register(Box::new(register_db_plugin_with_options(
                    offline,
                    write_metadata,
                )))
                .expect("db plugin symbol ownership must be valid");
            return registry;
        }
    }

    registry
        .register(Box::new(register_derive_plugin()))
        .expect("derive plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_serde_plugin()))
        .expect("serde plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_http_client_plugin()))
        .expect("http client plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_route_plugin()))
        .expect("route plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_state_plugin()))
        .expect("state plugin symbol ownership must be valid");
    registry
        .register(Box::new(register_db_row_plugin()))
        .expect("db plugin symbol ownership must be valid");
    registry
}

/// Claims non-DB symbols during DB-only generation so they are ignored cleanly.
struct DbModePassThroughPlugin;

/// Derive traits intentionally claimed by the DB-only pass-through plugin.
const DB_MODE_PASS_THROUGH_TRAITS: &[&str] = &[
    "dust_dart::ToString",
    "dust_dart::Debug",
    "dust_dart::Eq",
    "dust_dart::CopyWith",
    "dust_dart::Serialize",
    "dust_dart::Deserialize",
];

/// Config annotations intentionally claimed by the DB-only pass-through plugin.
const DB_MODE_PASS_THROUGH_CONFIGS: &[&str] = &[
    "dust_dart::SerDe",
    "dust_dart::HttpClient",
    "dust_dart::GenerateTest",
    "dust_dart::GET",
    "dust_dart::POST",
    "dust_dart::PUT",
    "dust_dart::PATCH",
    "dust_dart::DELETE",
    "dust_dart::HEAD",
    "dust_dart::OPTIONS",
    "dust_dart::Path",
    "dust_dart::Queries",
    "dust_dart::Header",
    "dust_dart::Headers",
    "dust_dart::HeaderMap",
    "dust_dart::Body",
    "dust_dart::Field",
    "dust_dart::Part",
    "dust_dart::Extra",
    "dust_dart::FormUrlEncoded",
    "dust_dart::MultiPart",
    "dust_dart::HttpParse",
    "dust_flutter::Router",
    "dust_flutter::Route",
    "dust_flutter::GeneratedRoute",
    "dust_flutter::ViewModel",
];

impl DustPlugin for DbModePassThroughPlugin {
    fn plugin_name(&self) -> &'static str {
        "DbModePassThrough"
    }

    fn claimed_traits(&self) -> &'static [&'static str] {
        DB_MODE_PASS_THROUGH_TRAITS
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        DB_MODE_PASS_THROUGH_CONFIGS
    }

    fn validate(&self, _library: &dust_ir::DartFileIr) -> Vec<Diagnostic> {
        Vec::new()
    }

    fn emit(&self, _library: &dust_ir::DartFileIr, _plan: &SymbolPlan) -> PluginContribution {
        PluginContribution::default()
    }
}
