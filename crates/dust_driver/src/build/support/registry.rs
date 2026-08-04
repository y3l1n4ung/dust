use dust_db_plugin::{
    DATABASE_PLUGIN_NAME, register_row_plugin as register_db_row_plugin,
    register_validating_plugin as register_db_validating_plugin,
};
use dust_http_client_plugin::register_plugin as register_http_client_plugin;
use dust_plugin_api::{DustPlugin, MetadataOutput, PluginExecutionMode, PluginRegistry};
use dust_plugin_derive::register_plugin as register_derive_plugin;
use dust_plugin_serde::register_plugin as register_serde_plugin;
use dust_route_plugin::register_plugin as register_route_plugin;
use dust_state_plugin::register_plugin as register_state_plugin;

/// Selects which plugin registry should run for a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegistrySelection {
    /// Full Dust registry for normal build and check commands.
    All,
    /// One validating plugin plus symbol claims from all other plugins.
    ValidatingPlugin {
        /// Stable name of the selected validating plugin.
        plugin: &'static str,
        /// Validation and metadata execution policy.
        mode: PluginExecutionMode,
    },
}

impl RegistrySelection {
    /// Builds registry selection for a writing build request.
    pub(crate) fn for_build(value: crate::request::DbRequestOptions) -> Self {
        Self::from_db_request(value, MetadataOutput::Write)
    }

    /// Builds registry selection for a non-writing check request.
    pub(crate) fn for_check(value: crate::request::DbRequestOptions) -> Self {
        Self::from_db_request(value, MetadataOutput::ReadOnly)
    }

    /// Converts DB CLI compatibility flags into the shared validating-plugin mode.
    fn from_db_request(value: crate::request::DbRequestOptions, metadata: MetadataOutput) -> Self {
        if !value.only_db {
            return Self::All;
        }
        let mode = if value.offline {
            PluginExecutionMode::offline(metadata)
        } else {
            PluginExecutionMode::online(metadata)
        };
        Self::ValidatingPlugin {
            plugin: DATABASE_PLUGIN_NAME,
            mode,
        }
    }

    /// Returns the cache salt that distinguishes registry modes.
    pub(crate) fn cache_salt(self) -> String {
        match self {
            Self::All => "registry:all".to_owned(),
            Self::ValidatingPlugin { plugin, mode } => {
                format!("registry:validating:{plugin}:{}", mode.cache_key())
            }
        }
    }
}

/// One built-in plugin's normal and optional validating-mode factories.
struct PluginRegistration {
    /// Creates the plugin used by the full registry.
    default_factory: fn() -> Box<dyn DustPlugin>,
    /// Creates the plugin used when selected for focused validation.
    validating_factory: Option<fn(PluginExecutionMode) -> Box<dyn DustPlugin>>,
}

/// Built-in plugins in deterministic execution and symbol-ownership order.
const BUILTIN_PLUGINS: &[PluginRegistration] = &[
    PluginRegistration {
        default_factory: boxed_derive_plugin,
        validating_factory: None,
    },
    PluginRegistration {
        default_factory: boxed_serde_plugin,
        validating_factory: None,
    },
    PluginRegistration {
        default_factory: boxed_http_client_plugin,
        validating_factory: None,
    },
    PluginRegistration {
        default_factory: boxed_route_plugin,
        validating_factory: None,
    },
    PluginRegistration {
        default_factory: boxed_state_plugin,
        validating_factory: None,
    },
    PluginRegistration {
        default_factory: boxed_db_row_plugin,
        validating_factory: Some(boxed_db_validating_plugin),
    },
];

/// Builds the full default plugin registry.
pub(crate) fn default_registry() -> PluginRegistry {
    registry_for_selection(RegistrySelection::All)
}

/// Builds the plugin registry requested by a command.
pub(crate) fn registry_for_selection(selection: RegistrySelection) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    match selection {
        RegistrySelection::All => {
            for registration in BUILTIN_PLUGINS {
                registry
                    .register((registration.default_factory)())
                    .expect("built-in plugin symbol ownership must be valid");
            }
        }
        RegistrySelection::ValidatingPlugin { plugin, mode } => {
            let selected = BUILTIN_PLUGINS
                .iter()
                .find(|registration| (registration.default_factory)().plugin_name() == plugin)
                .unwrap_or_else(|| panic!("validating plugin `{plugin}` is not registered"));
            let validating_factory = selected
                .validating_factory
                .unwrap_or_else(|| panic!("plugin `{plugin}` does not support focused validation"));
            registry
                .register(validating_factory(mode))
                .expect("validating plugin symbol ownership must be valid");

            for registration in BUILTIN_PLUGINS {
                let default_plugin = (registration.default_factory)();
                if default_plugin.plugin_name() != plugin {
                    registry
                        .register_symbols_only(default_plugin)
                        .expect("fallback plugin symbol ownership must be valid");
                }
            }
        }
    }
    registry
}

/// Creates the default Derive plugin behind the shared factory contract.
fn boxed_derive_plugin() -> Box<dyn DustPlugin> {
    Box::new(register_derive_plugin())
}

/// Creates the default SerDe plugin behind the shared factory contract.
fn boxed_serde_plugin() -> Box<dyn DustPlugin> {
    Box::new(register_serde_plugin())
}

/// Creates the default HTTP client plugin behind the shared factory contract.
fn boxed_http_client_plugin() -> Box<dyn DustPlugin> {
    Box::new(register_http_client_plugin())
}

/// Creates the default Route plugin behind the shared factory contract.
fn boxed_route_plugin() -> Box<dyn DustPlugin> {
    Box::new(register_route_plugin())
}

/// Creates the default State plugin behind the shared factory contract.
fn boxed_state_plugin() -> Box<dyn DustPlugin> {
    Box::new(register_state_plugin())
}

/// Creates the default row-mapping DB plugin behind the shared factory contract.
fn boxed_db_row_plugin() -> Box<dyn DustPlugin> {
    Box::new(register_db_row_plugin())
}

/// Creates the full DB plugin for focused validation and generation.
fn boxed_db_validating_plugin(mode: PluginExecutionMode) -> Box<dyn DustPlugin> {
    Box::new(register_db_validating_plugin(mode))
}

#[cfg(test)]
mod tests {
    use dust_ir::SymbolId;

    use super::{RegistrySelection, registry_for_selection};
    use crate::request::DbRequestOptions;

    #[test]
    fn validating_selection_uses_generic_deterministic_cache_salt() {
        let selection = RegistrySelection::for_build(DbRequestOptions {
            only_db: true,
            offline: true,
        });

        assert_eq!(
            selection.cache_salt(),
            "registry:validating:Database:offline:write-metadata"
        );
    }

    #[test]
    fn focused_registry_claims_but_does_not_discover_other_plugins() {
        let registry = registry_for_selection(RegistrySelection::for_check(DbRequestOptions {
            only_db: true,
            offline: false,
        }));

        assert!(
            registry
                .claimed_config_symbols()
                .contains(&SymbolId::new("dust_dart::SerDe"))
        );
        assert!(
            registry
                .all_supported_annotations()
                .contains(&"SqlxDatabase")
        );
        assert!(!registry.all_supported_annotations().contains(&"HttpClient"));
    }
}
