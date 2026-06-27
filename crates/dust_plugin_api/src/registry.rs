use std::collections::HashMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{DartFileIr, SymbolId};
use dust_parser_dart::ParsedDartFileSurface;

use crate::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext};

/// One plugin registration plus its claimed symbols.
struct RegisteredPlugin {
    /// The plugin implementation.
    plugin: Box<dyn DustPlugin>,
    /// Trait symbols claimed by the plugin.
    trait_symbols: Vec<SymbolId>,
    /// Config symbols claimed by the plugin.
    config_symbols: Vec<SymbolId>,
    /// Surface annotation names supported by the plugin.
    supported_annotations: &'static [&'static str],
}

/// The registered set of Dust plugins plus symbol ownership checks.
pub struct PluginRegistry {
    /// Registered plugins in deterministic order.
    plugins: Vec<RegisteredPlugin>,
    /// Trait symbol owners.
    trait_owners: HashMap<SymbolId, &'static str>,
    /// Config symbol owners.
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
        let trait_symbols = plugin
            .claimed_traits()
            .iter()
            .map(|symbol| SymbolId::new(*symbol))
            .collect::<Vec<_>>();
        let config_symbols = plugin
            .claimed_configs()
            .iter()
            .map(|symbol| SymbolId::new(*symbol))
            .collect::<Vec<_>>();
        let supported_annotations = plugin.supported_annotations();

        for symbol in &trait_symbols {
            if let Some(owner) = self.trait_owners.get(symbol) {
                return Err(Diagnostic::error(format!(
                    "trait symbol `{}` is already owned by plugin `{owner}`",
                    symbol.0
                )));
            }
            self.trait_owners.insert(symbol.clone(), plugin_name);
        }

        for symbol in &config_symbols {
            if let Some(owner) = self.config_owners.get(symbol) {
                return Err(Diagnostic::error(format!(
                    "config symbol `{}` is already owned by plugin `{owner}`",
                    symbol.0
                )));
            }
            self.config_owners.insert(symbol.clone(), plugin_name);
        }

        self.plugins.push(RegisteredPlugin {
            plugin,
            trait_symbols,
            config_symbols,
            supported_annotations,
        });
        Ok(())
    }

    /// Returns plugin names in registration order.
    pub fn plugin_names(&self) -> Vec<&'static str> {
        self.plugins
            .iter()
            .map(|plugin| plugin.plugin.plugin_name())
            .collect()
    }

    /// Returns all claimed trait symbols in registration order.
    pub fn claimed_trait_symbols(&self) -> Vec<SymbolId> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.trait_symbols.iter().cloned())
            .collect()
    }

    /// Returns all claimed config symbols in registration order.
    pub fn claimed_config_symbols(&self) -> Vec<SymbolId> {
        self.plugins
            .iter()
            .flat_map(|plugin| plugin.config_symbols.iter().cloned())
            .collect()
    }

    /// Returns all unique surface-level annotation names supported by registered plugins.
    pub fn all_supported_annotations(&self) -> Vec<&'static str> {
        let mut names: Vec<_> = self
            .plugins
            .iter()
            .flat_map(|plugin| plugin.supported_annotations.iter().copied())
            .collect();
        names.sort_unstable();
        names.dedup();
        names
    }

    /// Returns all config symbols that do not require a generated part directive.
    pub fn all_partless_configs(&self) -> Vec<&'static str> {
        let mut symbols: Vec<_> = self
            .plugins
            .iter()
            .flat_map(|plugin| plugin.plugin.partless_configs().iter().copied())
            .collect();
        symbols.sort_unstable();
        symbols.dedup();
        symbols
    }

    /// Builds one deterministic symbol plan for a lowered library.
    pub fn build_symbol_plan(&self, file: &DartFileIr) -> SymbolPlan {
        let mut plan = SymbolPlan::default();
        for plugin in &self.plugins {
            for symbol in plugin.plugin.requested_symbols(file) {
                plan.reserve(symbol);
            }
        }
        plan
    }

    /// Collects parse-only workspace facts from all plugins in registration order.
    pub fn collect_workspace_analysis(
        &self,
        context: WorkspaceAnalysisContext<'_>,
        file: &ParsedDartFileSurface,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        for plugin in &self.plugins {
            plugin
                .plugin
                .collect_workspace_analysis(context, file, analysis);
        }
    }

    /// Runs validation across all registered plugins in registration order.
    pub fn validate_library(&self, file: &DartFileIr) -> Vec<Diagnostic> {
        self.validate_library_with_plan(file, &SymbolPlan::default())
    }

    /// Runs validation with one shared symbol plan in registration order.
    pub fn validate_library_with_plan(
        &self,
        file: &DartFileIr,
        plan: &SymbolPlan,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for plugin in &self.plugins {
            diagnostics.extend(plugin.plugin.validate_with_plan(file, plan));
        }
        diagnostics
    }

    /// Collects plugin contributions in registration order using one shared symbol plan.
    pub fn emit_contributions(
        &self,
        file: &DartFileIr,
        plan: &SymbolPlan,
    ) -> Vec<crate::PluginContribution> {
        let mut contributions = Vec::with_capacity(self.plugins.len());
        for plugin in &self.plugins {
            contributions.push(plugin.plugin.emit(file, plan));
        }
        contributions
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
