use dust_diagnostics::Diagnostic;
use dust_ir::{LibraryIr, SymbolId};
use dust_plugin_api::{DustPlugin, PluginContribution, SymbolPlan};

use crate::{emit::emit_library, validate::validate_library};

/// The built-in plugin that implements Dust's serde derive traits.
pub struct SerdePlugin;

/// Creates the built-in serde plugin instance.
pub fn register_plugin() -> SerdePlugin {
    SerdePlugin
}

impl DustPlugin for SerdePlugin {
    fn plugin_name(&self) -> &'static str {
        "dust_plugin_serde"
    }

    fn claimed_traits(&self) -> Vec<SymbolId> {
        vec![
            SymbolId::new("derive_serde_annotation::Serialize"),
            SymbolId::new("derive_serde_annotation::Deserialize"),
        ]
    }

    fn claimed_configs(&self) -> Vec<SymbolId> {
        vec![SymbolId::new("derive_serde_annotation::SerDe")]
    }

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
        symbols
    }

    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        validate_library(library)
    }

    fn emit(&self, library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        emit_library(library)
    }
}
