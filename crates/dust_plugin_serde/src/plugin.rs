use dust_diagnostics::Diagnostic;
use dust_ir::{LibraryIr, SymbolId};
use dust_plugin_api::{DustPlugin, PluginContribution, SymbolPlan};

use crate::{emit::emit_library, validate::validate_library};

/// The built-in plugin that implements Dust's JSON serialization and deserialization.
///
/// This plugin handles the `Serialize`, `Deserialize`, and `SerDe` traits by
/// generating appropriate Dart mixins and top-level helper functions.
pub struct SerdePlugin;

/// Creates and registers the built-in serde plugin.
pub fn register_plugin() -> SerdePlugin {
    SerdePlugin
}

impl DustPlugin for SerdePlugin {
    fn plugin_name(&self) -> &'static str {
        "dust_plugin_serde"
    }

    /// Claims the core serde traits defined in the `derive_serde_annotation` package.
    fn claimed_traits(&self) -> Vec<SymbolId> {
        vec![
            SymbolId::new("derive_serde_annotation::Serialize"),
            SymbolId::new("derive_serde_annotation::Deserialize"),
        ]
    }

    /// Claims the core configuration trait for customizing serialization behavior.
    fn claimed_configs(&self) -> Vec<SymbolId> {
        vec![SymbolId::new("derive_serde_annotation::SerDe")]
    }

    /// Informs the resolution engine that this plugin will generate specific private
    /// helper functions for each annotated class or enum.
    ///
    /// This allows other plugins or the lowerer to recognize these symbols as "future"
    /// declarations that will be provided during the emission phase.
    fn requested_symbols(&self, library: &LibraryIr) -> Vec<String> {
        let mut symbols = Vec::new();
        for class in &library.classes {
            if class
                .traits
                .iter()
                .any(|item| item.symbol.0 == "derive_serde_annotation::Serialize")
            {
                symbols.push(format!("_${}ToJson", class.name));
            }
            if class
                .traits
                .iter()
                .any(|item| item.symbol.0 == "derive_serde_annotation::Deserialize")
            {
                symbols.push(format!("_${}FromJson", class.name));
            }
        }
        for e in &library.enums {
            if e.traits
                .iter()
                .any(|item| item.symbol.0 == "derive_serde_annotation::Serialize")
            {
                symbols.push(format!("_${}ToJson", e.name));
            }
            if e.traits
                .iter()
                .any(|item| item.symbol.0 == "derive_serde_annotation::Deserialize")
            {
                symbols.push(format!("_${}FromJson", e.name));
            }
        }
        symbols
    }

    /// Validates that the library is suitable for SerDe generation.
    ///
    /// This includes checking for abstract classes that want deserialization,
    /// unsupported field types (like function types), and ensuring appropriate
    /// constructors exist.
    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        validate_library(library)
    }

    /// Performs the actual code emission for the plugin.
    ///
    /// The plugin contributes mixins for `toJson` and top-level private helpers
    /// for the actual JSON mapping logic.
    fn emit(&self, library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        emit_library(library)
    }
}
