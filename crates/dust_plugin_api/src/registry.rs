use std::collections::HashMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{LibraryIr, SymbolId};
use dust_parser_dart::ParsedLibrarySurface;

use crate::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};

/// The registered set of Dust plugins plus symbol ownership checks.
pub struct PluginRegistry {
    plugins: Vec<Box<dyn DustPlugin>>,
    trait_owners: HashMap<SymbolId, &'static str>,
    config_owners: HashMap<SymbolId, &'static str>,
}

impl PluginRegistry {
    /// Creates an empty plugin registry.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            trait_owners: HashMap::new(),
            config_owners: HashMap::new(),
        }
    }

    /// Registers one plugin, failing if it claims a symbol already owned by another plugin.
    pub fn register(&mut self, plugin: Box<dyn DustPlugin>) -> Result<(), Diagnostic> {
        let plugin_name = plugin.plugin_name();

        for symbol in plugin.claimed_traits() {
            if let Some(owner) = self.trait_owners.get(&symbol) {
                return Err(Diagnostic::error(format!(
                    "trait symbol `{}` is already owned by plugin `{owner}`",
                    symbol.0
                )));
            }
            self.trait_owners.insert(symbol, plugin_name);
        }

        for symbol in plugin.claimed_configs() {
            if let Some(owner) = self.config_owners.get(&symbol) {
                return Err(Diagnostic::error(format!(
                    "config symbol `{}` is already owned by plugin `{owner}`",
                    symbol.0
                )));
            }
            self.config_owners.insert(symbol, plugin_name);
        }

        self.plugins.push(plugin);
        Ok(())
    }

    /// Returns plugin names in registration order.
    pub fn plugin_names(&self) -> Vec<&'static str> {
        self.plugins
            .iter()
            .map(|plugin| plugin.plugin_name())
            .collect()
    }

    /// Returns all claimed trait symbols in registration order.
    pub fn claimed_trait_symbols(&self) -> Vec<SymbolId> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.claimed_traits())
            .collect()
    }

    /// Returns all claimed config symbols in registration order.
    pub fn claimed_config_symbols(&self) -> Vec<SymbolId> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.claimed_configs())
            .collect()
    }

    /// Builds one deterministic symbol plan for a lowered library.
    pub fn build_symbol_plan(&self, library: &LibraryIr) -> SymbolPlan {
        let mut plan = SymbolPlan::default();
        for plugin in &self.plugins {
            for symbol in plugin.requested_symbols(library) {
                plan.reserve(symbol);
            }
        }
        plan
    }

    /// Collects parse-only workspace facts from all plugins in registration order.
    pub fn collect_workspace_analysis(
        &self,
        library: &ParsedLibrarySurface,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        for plugin in &self.plugins {
            plugin.collect_workspace_analysis(library, analysis);
        }
    }

    /// Runs validation across all registered plugins in registration order.
    pub fn validate_library(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for plugin in &self.plugins {
            diagnostics.extend(plugin.validate(library));
        }
        diagnostics
    }

    /// Collects plugin contributions in registration order using one shared symbol plan.
    pub fn emit_contributions(
        &self,
        library: &LibraryIr,
        plan: &SymbolPlan,
    ) -> Vec<crate::PluginContribution> {
        self.plugins
            .iter()
            .map(|plugin| plugin.emit(library, plan))
            .collect()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
