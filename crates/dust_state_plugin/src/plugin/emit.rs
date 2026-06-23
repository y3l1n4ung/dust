use dust_ir::DartFileIr;
use dust_plugin_api::{PluginContribution, SymbolPlan};

use super::parse::{parse_view_model_config, view_model_config};

/// Template rendering for generated view model support classes.
mod render;
/// State field discovery used by generated proxy selectors.
mod state_fields;

use self::{render::render_view_model_output, state_fields::class_fields};

/// Field metadata passed from state discovery to the view model renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StateFieldSpec {
    /// Dart field name read from the state object.
    pub(super) name: String,
    /// Dart type source emitted for the generated selector getter.
    pub(super) type_source: String,
}

/// Emits generated support types for all view models in a resolved library.
pub(crate) fn emit_library_state(library: &DartFileIr, plan: &SymbolPlan) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let view_models = library
        .classes
        .iter()
        .filter_map(|class| {
            let config = view_model_config(&class.configs)?;
            let annotation = parse_view_model_config(config)?;
            Some((class, annotation))
        })
        .collect::<Vec<_>>();
    if view_models.is_empty() {
        return contribution;
    }

    let state_facts = state_fields::state_facts(plan);
    for (class, annotation) in view_models {
        let args_type = annotation
            .args_type
            .clone()
            .unwrap_or_else(|| "ViewModelArgs".to_owned());
        let state_fields = class_fields(library, &state_facts, &annotation.state_type);

        contribution.support_types.push(render_view_model_output(
            class,
            &annotation.state_type,
            &args_type,
            annotation.initial_source.as_deref(),
            &state_fields,
        ));
    }
    contribution
}
