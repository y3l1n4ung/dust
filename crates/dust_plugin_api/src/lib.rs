#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![doc = "Plugin contracts and registry types for Dust generation plugins."]

mod analysis;
mod contribution;
mod plugin;
mod registry;
mod symbols;

pub use analysis::{LibraryAnalysisSnapshot, WorkspaceAnalysis, WorkspaceAnalysisBuilder};
pub use contribution::{ClassMixinContribution, PluginContribution};
pub use plugin::DustPlugin;
pub use registry::PluginRegistry;
pub use symbols::{RequestedSymbol, SymbolPlan};
