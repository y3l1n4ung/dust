use std::collections::HashMap;

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{DartFileIr, TypeIr};
use dust_plugin_api::SymbolPlan;

use crate::plugin::{constants::STATES_ANALYSIS_KEY, model::StateFact};

use super::StateFieldSpec;

/// Loads state field facts produced by workspace analysis.
pub(super) fn state_facts(plan: &SymbolPlan) -> HashMap<String, Vec<StateFieldSpec>> {
    plan.workspace_string_set(STATES_ANALYSIS_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| serde_json::from_str::<StateFact>(value).ok())
        .map(|fact| {
            let fields = fact
                .fields
                .into_iter()
                .map(|field| StateFieldSpec {
                    name: field.name,
                    type_source: field.type_source,
                })
                .collect::<Vec<_>>();
            (fact.class_name, fields)
        })
        .collect()
}

/// Returns state fields from the current library or workspace fallback facts.
pub(super) fn class_fields(
    library: &DartFileIr,
    state_facts: &HashMap<String, Vec<StateFieldSpec>>,
    class_name: &str,
) -> Vec<StateFieldSpec> {
    if let Some(class) = library
        .classes
        .iter()
        .find(|candidate| candidate.name == class_name)
    {
        let fields = class
            .fields
            .iter()
            .map(|field| StateFieldSpec {
                name: field.name.clone(),
                type_source: render_type(&field.ty),
            })
            .collect::<Vec<_>>();
        if !fields.is_empty() {
            return fields;
        }
    }
    state_facts
        .get(class_name)
        .filter(|fields| !fields.is_empty())
        .cloned()
        .unwrap_or_default()
}

/// Renders a state field type through the shared dynamic-safe type renderer.
fn render_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render(ty)
}
