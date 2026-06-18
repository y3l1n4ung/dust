use dust_ir::DartFileIr;
use dust_plugin_api::{PluginContribution, SymbolPlan};

use crate::{
    analysis::COPYABLE_TYPES_ANALYSIS_KEY,
    features::{
        clone_copy_with::{CopyableTypes, emit_copy_with},
        debug::emit_debug_mixin,
        eq_hash::{emit_eq, emit_hash_code, plan_equality},
        validate::emit_validate,
    },
};

pub(crate) fn emit_library(library: &DartFileIr, _plan: &SymbolPlan) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let local_copyable_types = library
        .classes
        .iter()
        .filter(|class| {
            class
                .traits
                .iter()
                .any(|trait_app| trait_app.symbol.0 == crate::features::COPY_WITH_SYMBOL)
        })
        .map(|class| class.name.as_str())
        .collect::<Vec<_>>();
    let copyable_types = CopyableTypes::new(
        &local_copyable_types,
        _plan.workspace_string_set(COPYABLE_TYPES_ANALYSIS_KEY),
    );
    let equality = plan_equality(library);
    contribution
        .shared_helpers
        .extend(equality.declarations().iter().cloned());

    for class in &library.classes {
        if let Some(debug_mixin) = emit_debug_mixin(class) {
            contribution.push_mixin_member(&class.name, debug_mixin);
        }
        if let Some(eq) = emit_eq(class, &equality) {
            contribution.push_mixin_member(&class.name, eq);
        }
        if let Some(hash_code) = emit_hash_code(class, &equality) {
            contribution.push_mixin_member(&class.name, hash_code);
        }
        if let Some(copy_with) = emit_copy_with(class, &copyable_types) {
            contribution.push_mixin_member(&class.name, copy_with);
        }
        if let Some(validate) = emit_validate(library, class) {
            contribution.push_mixin_member(&class.name, validate.mixin_member);
            contribution.support_types.push(validate.support_type);
        }
    }

    contribution
}
