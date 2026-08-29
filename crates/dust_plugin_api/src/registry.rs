use std::collections::HashMap;

use crate::{DustPlugin, PluginContext, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_diagnostics::Diagnostic;
use dust_ir::{DartFileIr, SymbolId};

/// One plugin registration plus its claimed symbols.
struct RegisteredPlugin {
    /// The plugin implementation.
    plugin: Box<dyn DustPlugin>,
    /// Whether the plugin participates in analysis, validation, and generation.
    executes: bool,
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
        self.register_with_execution(plugin, true, false)
    }

    /// Registers only a plugin's symbol claims without executing the plugin.
    ///
    /// This keeps non-selected annotations resolvable in focused plugin modes
    /// without discovering files for, validating, or generating with the
    /// non-selected plugin. Existing active ownership takes precedence over
    /// overlapping fallback claims.
    pub fn register_symbols_only(&mut self, plugin: Box<dyn DustPlugin>) -> Result<(), Diagnostic> {
        self.register_with_execution(plugin, false, true)
    }

    /// Registers one plugin with an explicit execution policy.
    fn register_with_execution(
        &mut self,
        plugin: Box<dyn DustPlugin>,
        executes: bool,
        skip_owned_symbols: bool,
    ) -> Result<(), Diagnostic> {
        let plugin_name = plugin.plugin_name();
        let mut trait_symbols = Vec::new();
        let mut config_symbols = Vec::new();
        let supported_annotations = if executes {
            plugin.supported_annotations()
        } else {
            &[]
        };

        for symbol in plugin
            .claimed_traits()
            .iter()
            .map(|symbol| SymbolId::new(*symbol))
        {
            if let Some(owner) = self.trait_owners.get(&symbol) {
                if skip_owned_symbols {
                    continue;
                }
                return Err(Diagnostic::error(format!(
                    "trait symbol `{}` is already owned by plugin `{owner}`",
                    symbol.0
                )));
            }
            self.trait_owners.insert(symbol.clone(), plugin_name);
            trait_symbols.push(symbol);
        }

        for symbol in plugin
            .claimed_configs()
            .iter()
            .map(|symbol| SymbolId::new(*symbol))
        {
            if let Some(owner) = self.config_owners.get(&symbol) {
                if skip_owned_symbols {
                    continue;
                }
                return Err(Diagnostic::error(format!(
                    "config symbol `{}` is already owned by plugin `{owner}`",
                    symbol.0
                )));
            }
            self.config_owners.insert(symbol.clone(), plugin_name);
            config_symbols.push(symbol);
        }

        self.plugins.push(RegisteredPlugin {
            plugin,
            executes,
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
            .filter(|plugin| plugin.executes)
            .flat_map(|plugin| plugin.plugin.partless_configs().iter().copied())
            .collect();
        symbols.sort_unstable();
        symbols.dedup();
        symbols
    }

    /// Builds one deterministic symbol plan for a lowered library.
    pub fn build_symbol_plan(&self, file: &DartFileIr) -> SymbolPlan {
        let mut plan = SymbolPlan::default();
        for plugin in self.plugins.iter().filter(|plugin| plugin.executes) {
            for symbol in plugin.plugin.requested_symbols(file) {
                plan.reserve(symbol);
            }
        }
        plan
    }

    /// Collects canonical IR workspace facts from all registered plugins.
    pub fn collect_workspace_analysis_ir(
        &self,
        file: &DartFileIr,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        for plugin in self.plugins.iter().filter(|plugin| plugin.executes) {
            plugin.plugin.collect_workspace_analysis_ir(file, analysis);
        }
    }

    /// Returns whether every registered plugin executes.
    ///
    /// A focused registry — `dust db build` selecting the Database plugin —
    /// registers the others for symbol ownership only. Their annotations are
    /// still in the source, but nothing in this run will regenerate what they
    /// own, so what they own must be left exactly as it is rather than
    /// rewritten from a registry that cannot produce it.
    pub fn executes_all(&self) -> bool {
        self.plugins.iter().all(|plugin| plugin.executes)
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
        for plugin in self.plugins.iter().filter(|plugin| plugin.executes) {
            diagnostics.extend(plugin.plugin.validate_with_plan(file, plan));
        }
        diagnostics
    }

    /// Collects generated plugin contributions in registration order using one shared symbol plan.
    pub fn generate_units(
        &self,
        file: &DartFileIr,
        plan: &SymbolPlan,
    ) -> Vec<crate::PluginContribution> {
        let mut contributions = Vec::with_capacity(self.plugins.len());
        let context = PluginContext { symbol_plan: plan };
        for plugin in self.plugins.iter().filter(|plugin| plugin.executes) {
            contributions.extend(plugin.plugin.generate(file, &context));
        }
        contributions
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
