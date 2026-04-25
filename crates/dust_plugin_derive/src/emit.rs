use dust_ir::LibraryIr;
use dust_plugin_api::{PluginContribution, SymbolPlan};

use crate::features::{
    clone_copy_with::{emit_clone, emit_copy_with},
    debug::emit_debug_mixin,
    eq_hash::{emit_eq, emit_hash_code, emit_shared_helpers},
};

pub(crate) fn emit_library(library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    contribution
        .shared_helpers
        .extend(emit_shared_helpers(library));

    for class in &library.classes {
        if let Some(debug_mixin) = emit_debug_mixin(class) {
            contribution.push_mixin_member(&class.name, debug_mixin);
        }
        if let Some(eq) = emit_eq(class) {
            contribution.push_mixin_member(&class.name, eq);
        }
        if let Some(hash_code) = emit_hash_code(class) {
            contribution.push_mixin_member(&class.name, hash_code);
        }
        if let Some(copy_with) = emit_copy_with(class) {
            contribution.push_mixin_member(&class.name, copy_with);
        }
        if let Some(clone) = emit_clone(class) {
            contribution.push_mixin_member(&class.name, clone);
        }
    }

    contribution
}
