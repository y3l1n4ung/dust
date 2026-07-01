use dust_ir::DartFileIr;
use dust_plugin_api::{PluginContribution, SymbolPlan};

use super::parse::{parse_view_model_config, view_model_config};

/// Template rendering for generated view model support classes.
mod render;

use self::render::render_view_model_output;

/// Emits generated support types for all view models in a resolved library.
pub(crate) fn emit_library_state(library: &DartFileIr, _plan: &SymbolPlan) -> PluginContribution {
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

    for (class, annotation) in view_models {
        let args_type = annotation
            .args_type
            .clone()
            .unwrap_or_else(|| "ViewModelArgs".to_owned());

        contribution.support_types.push(render_view_model_output(
            class,
            &annotation.state_type,
            &args_type,
            annotation.initial_source.as_deref(),
            annotation.mode,
        ));
    }
    contribution
}
