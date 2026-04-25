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
    by_name: BTreeMap<String, ResolvedSymbol>,
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
        self.by_name.insert(
            annotation_name.into(),
            ResolvedSymbol {
                symbol: SymbolId::new(symbol),
                kind: SymbolKind::Trait,
            },
        )
    }

    /// Registers one config symbol by short annotation name.
    pub fn register_config(
        &mut self,
        annotation_name: impl Into<String>,
        symbol: impl Into<String>,
    ) -> Option<ResolvedSymbol> {
        self.by_name.insert(
            annotation_name.into(),
            ResolvedSymbol {
                symbol: SymbolId::new(symbol),
                kind: SymbolKind::Config,
            },
        )
    }

    /// Looks up one annotation name.
    pub fn resolve(&self, annotation_name: &str) -> Option<&ResolvedSymbol> {
        self.by_name.get(annotation_name)
    }
}
