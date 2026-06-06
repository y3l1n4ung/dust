use std::collections::BTreeMap;

use dust_ir::SymbolId;

/// The semantic role owned by a registered Dust symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A trait-like symbol applied to a class.
    Trait,
    /// A configuration-like symbol applied to a class.
    Config,
}

/// One registered symbol plus its semantic role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSymbol {
    /// The fully qualified semantic symbol identifier.
    pub symbol: SymbolId,
    /// The role this symbol plays in the pipeline.
    pub kind: SymbolKind,
}

/// A lookup table from surface annotation names to fully qualified Dust symbols.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SymbolCatalog {
    by_name: BTreeMap<String, ResolvedSymbols>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ResolvedSymbols {
    trait_symbol: Option<ResolvedSymbol>,
    config_symbol: Option<ResolvedSymbol>,
}

impl SymbolCatalog {
    /// Creates an empty symbol catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one trait symbol by short annotation name.
    pub fn register_trait(
        &mut self,
        annotation_name: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Option<ResolvedSymbol> {
        self.by_name
            .entry(annotation_name.into())
            .or_default()
            .trait_symbol
            .replace(ResolvedSymbol {
                symbol: SymbolId::new(symbol),
                kind: SymbolKind::Trait,
            })
    }

    /// Registers one config symbol by short annotation name.
    pub fn register_config(
        &mut self,
        annotation_name: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Option<ResolvedSymbol> {
        self.by_name
            .entry(annotation_name.into())
            .or_default()
            .config_symbol
            .replace(ResolvedSymbol {
                symbol: SymbolId::new(symbol),
                kind: SymbolKind::Config,
            })
    }

    /// Looks up one annotation name.
    pub fn resolve(&self, annotation_name: &str) -> Option<&ResolvedSymbol> {
        self.resolve_trait(annotation_name)
            .or_else(|| self.resolve_config(annotation_name))
    }

    /// Looks up one annotation name as a trait symbol.
    pub fn resolve_trait(&self, annotation_name: &str) -> Option<&ResolvedSymbol> {
        self.by_name
            .get(annotation_name)
            .and_then(|symbols| symbols.trait_symbol.as_ref())
    }

    /// Looks up one annotation name as a config symbol.
    pub fn resolve_config(&self, annotation_name: &str) -> Option<&ResolvedSymbol> {
        self.by_name
            .get(annotation_name)
            .and_then(|symbols| symbols.config_symbol.as_ref())
    }
}
