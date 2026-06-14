use dust_diagnostics::Diagnostic;
use dust_ir::DartFileIr;
use dust_parser_dart::ParsedDartFileSurface;
use dust_plugin_api::{
    DustPlugin, PluginContribution, SymbolPlan, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext,
};

mod analysis;
mod build;
mod constants;
mod emit;
mod model;
mod parse;
mod validate;

use self::analysis::collect_route_workspace_analysis;
use self::build::build_router_spec;
use self::constants::{CLAIMED_CONFIG_SYMBOLS, SUPPORTED_ANNOTATIONS};
use self::emit::render_route_generated;
use self::validate::validate_library_routes;

/// Dust plugin for typed Flutter Navigator 2.0 routing.
pub struct RoutePlugin;

impl RoutePlugin {
    /// Creates a new route plugin.
    pub fn new() -> Self {
        Self
    }
}

impl Default for RoutePlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates the route plugin.
pub fn register_plugin() -> RoutePlugin {
    RoutePlugin::new()
}

impl DustPlugin for RoutePlugin {
    fn plugin_name(&self) -> &'static str {
        "Route"
    }

    fn claimed_configs(&self) -> &'static [&'static str] {
        CLAIMED_CONFIG_SYMBOLS
    }

    fn partless_configs(&self) -> &'static [&'static str] {
        CLAIMED_CONFIG_SYMBOLS
    }

    fn supported_annotations(&self) -> &'static [&'static str] {
        SUPPORTED_ANNOTATIONS
    }

    fn collect_workspace_analysis(
        &self,
        context: WorkspaceAnalysisContext<'_>,
        library: &ParsedDartFileSurface,
        analysis: &mut WorkspaceAnalysisBuilder,
    ) {
        collect_route_workspace_analysis(context, library, analysis);
    }

    fn validate(&self, library: &DartFileIr) -> Vec<Diagnostic> {
        validate_library_routes(library)
    }

    fn emit(&self, library: &DartFileIr, plan: &SymbolPlan) -> PluginContribution {
        let spec = match build_router_spec(library, plan) {
            Ok(Some(spec)) => spec,
            Ok(None) => return PluginContribution::default(),
            Err(diagnostics) => {
                return PluginContribution {
                    diagnostics,
                    ..PluginContribution::default()
                };
            }
        };
        PluginContribution {
            primary_source: Some(render_route_generated(library, &spec)),
            ..PluginContribution::default()
        }
    }
}
