use dust_diagnostics::Diagnostic;
use dust_ir::{LibraryIr, SymbolId};
use dust_plugin_api::{AuxiliaryOutputContribution, DustPlugin, PluginContribution, SymbolPlan};
use dust_workspace::generated_test_output_path;

mod build;
mod constants;
mod emit;
mod model;
mod parse;
mod util;
mod validate;

use self::build::build_client_spec;
use self::constants::{GENERATE_TEST, HTTP_CLIENT, claimed_config_names};
use self::emit::{
    render_client_class, render_isolate_helpers, render_shared_helpers, render_test_file,
};
use self::parse::has_config_named;
use self::validate::validate_client_class;

/// Dust plugin for generating Dio-backed HTTP clients.
pub struct HttpClientPlugin;

impl HttpClientPlugin {
    /// Creates a new instance of the plugin.
    pub fn new() -> Self {
        Self
    }
}

impl Default for HttpClientPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates the HttpClient plugin.
pub fn register_plugin() -> HttpClientPlugin {
    HttpClientPlugin::new()
}

impl DustPlugin for HttpClientPlugin {
    fn plugin_name(&self) -> &'static str {
        "HttpClient"
    }

    fn claimed_configs(&self) -> Vec<SymbolId> {
        claimed_config_names()
            .iter()
            .map(|name| SymbolId::new(format!("{}::{name}", constants::PACKAGE)))
            .collect()
    }

    fn supported_annotations(&self) -> Vec<&'static str> {
        claimed_config_names().to_vec()
    }

    fn validate(&self, library: &LibraryIr) -> Vec<Diagnostic> {
        library
            .classes
            .iter()
            .flat_map(|class| validate_client_class(&library.imports, class))
            .collect()
    }

    fn emit(&self, library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
        let mut contribution = PluginContribution::default();
        let mut generated_any = false;
        let mut generated_test_specs = Vec::new();

        for class in &library.classes {
            if !has_config_named(&class.configs, HTTP_CLIENT) {
                continue;
            }

            let spec = match build_client_spec(&library.imports, class) {
                Ok(spec) => spec,
                Err(_) => continue,
            };

            generated_any = true;
            contribution.support_types.push(render_client_class(&spec));
            contribution
                .top_level_functions
                .extend(render_isolate_helpers(&spec));
            if has_config_named(&class.configs, GENERATE_TEST) {
                generated_test_specs.push(spec);
            }
        }

        if generated_any {
            contribution.shared_helpers.extend(render_shared_helpers());
        }
        if !generated_test_specs.is_empty() {
            if let Some(source) = render_test_file(library, &generated_test_specs) {
                contribution
                    .auxiliary_outputs
                    .push(AuxiliaryOutputContribution {
                        output_path: generated_test_output_path(
                            std::path::Path::new(&library.package_root),
                            std::path::Path::new(&library.source_path),
                        )
                        .expect("http client test outputs must originate from lib/ sources"),
                        source,
                    });
            }
        }

        contribution
    }
}
