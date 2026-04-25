use dust_diagnostics::Diagnostic;
use dust_ir::{LibraryIr, SymbolId};

use crate::{PluginContribution, SymbolPlan};

/// The contract implemented by every Dust generation plugin.
pub trait DustPlugin: Send + Sync {
    /// Returns the stable plugin name used in diagnostics and reports.
    fn plugin_name(&self) -> &'static str;

    /// Returns the trait symbols this plugin exclusively owns.
    fn claimed_traits(&self) -> Vec<SymbolId> {
        Vec::new()
    }

    /// Returns the config symbols this plugin exclusively owns.
    fn claimed_configs(&self) -> Vec<SymbolId> {
        Vec::new()
    }

    /// Returns generated helper symbol names this plugin wants reserved.
    fn requested_symbols(&self, _library: &LibraryIr) -> Vec<String> {
        Vec::new()
    }

    /// Validates the library from this plugin's point of view.
    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic>;

    /// Produces generated fragments for this plugin.
    fn emit(&self, library: &LibraryIr, plan: &SymbolPlan) -> PluginContribution;
}
