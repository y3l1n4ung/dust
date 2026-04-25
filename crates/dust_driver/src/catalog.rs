use dust_plugin_api::PluginRegistry;
use dust_resolver::SymbolCatalog;

/// Builds a resolver symbol catalog from the registered plugin ownership set.
pub(crate) fn build_symbol_catalog(registry: &PluginRegistry) -> SymbolCatalog {
    let mut catalog = SymbolCatalog::new();

    for symbol in registry.claimed_trait_symbols() {
        catalog.register_trait(short_annotation_name(&symbol.0), symbol.0);
    }
    for symbol in registry.claimed_config_symbols() {
        catalog.register_config(short_annotation_name(&symbol.0), symbol.0);
    }

    catalog
}

fn short_annotation_name(symbol: &str) -> String {
    symbol.rsplit("::").next().unwrap_or(symbol).to_owned()
}
