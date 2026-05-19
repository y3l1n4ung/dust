use dust_plugin_api::{PluginRegistry, short_symbol_name};
use dust_resolver::SymbolCatalog;

/// Builds a resolver symbol catalog from the registered plugin ownership set.
pub(crate) fn build_symbol_catalog(registry: &PluginRegistry) -> SymbolCatalog {
    let mut catalog = SymbolCatalog::new();

    for symbol in registry.claimed_trait_symbols() {
        let symbol_name = symbol.0;
        let annotation_name = short_symbol_name(&symbol_name).to_owned();
        catalog.register_trait(annotation_name, symbol_name);
    }
    for symbol in registry.claimed_config_symbols() {
        let symbol_name = symbol.0;
        let annotation_name = short_symbol_name(&symbol_name).to_owned();
        catalog.register_config(annotation_name, symbol_name);
    }

    catalog
}
