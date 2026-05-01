use std::sync::Arc;

use crate::WorkspaceAnalysis;

/// One requested generated symbol name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestedSymbol {
    /// The reserved symbol name.
    pub name: String,
}

impl RequestedSymbol {
    /// Creates one requested symbol.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// The deterministic set of reserved generated symbols for one library.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SymbolPlan {
    reserved: Vec<RequestedSymbol>,
    workspace_analysis: Arc<WorkspaceAnalysis>,
}

impl SymbolPlan {
    /// Reserves one symbol, keeping the first-seen order and ignoring duplicates.
    pub fn reserve(&mut self, name: impl Into<String>) {
        let symbol = RequestedSymbol::new(name);
        if !self.reserved.iter().any(|entry| entry.name == symbol.name) {
            self.reserved.push(symbol);
        }
    }

    /// Returns the reserved symbols in deterministic order.
    pub fn reserved(&self) -> &[RequestedSymbol] {
        &self.reserved
    }

    /// Returns `true` if the plan already contains the given symbol name.
    pub fn contains(&self, name: &str) -> bool {
        self.reserved.iter().any(|entry| entry.name == name)
    }

    /// Replaces the workspace-wide analysis facts attached to this plan.
    pub fn set_workspace_analysis(&mut self, analysis: Arc<WorkspaceAnalysis>) {
        self.workspace_analysis = analysis;
    }

    /// Returns the shared workspace analysis facts available to generators.
    pub fn workspace_analysis(&self) -> &WorkspaceAnalysis {
        self.workspace_analysis.as_ref()
    }

    /// Returns the workspace-wide string-set values recorded for one analysis key.
    pub fn workspace_string_set(&self, key: &str) -> Option<&[String]> {
        self.workspace_analysis.string_set(key)
    }
}
