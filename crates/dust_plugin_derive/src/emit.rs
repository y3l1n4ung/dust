use dust_ir::DartFileIr;
use dust_plugin_api::{PluginContribution, SymbolPlan};

use crate::features::{
    clone_copy_with::{emit_copy_with, plan_copy_with},
    debug::emit_debug_mixin,
    eq_hash::{emit_eq, emit_hash_code, plan_equality},
    validate::emit_validate,
};

/// Emits all derive-generated mixin members and support types for a library.
pub(crate) fn emit_library(library: &DartFileIr, _plan: &SymbolPlan) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let equality = plan_equality(library);
    let copy_with_plan = plan_copy_with(library);
    contribution
        .shared_helpers
        .extend(equality.declarations().iter().cloned());
    contribution
        .shared_helpers
        .extend(copy_with_plan.declarations().iter().cloned());
    let mut copy_with_credit_pending = true;

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
        if let Some(copy_with) = emit_copy_with(class, &copy_with_plan, copy_with_credit_pending) {
            contribution.push_mixin_member(&class.name, copy_with.mixin_member);
            contribution.support_types.push(copy_with.support_type);
            copy_with_credit_pending = false;
        }
        if let Some(validate) = emit_validate(library, class) {
            contribution.push_mixin_member(&class.name, validate.mixin_member);
            contribution.support_types.push(validate.support_type);
        }
    }

    contribution
}
