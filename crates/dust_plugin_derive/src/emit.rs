use std::collections::HashSet;

use dust_ir::LibraryIr;
use dust_plugin_api::{PluginContribution, SymbolPlan};

use crate::{
    analysis::COPYABLE_TYPES_ANALYSIS_KEY,
    features::{
        clone_copy_with::emit_copy_with,
        debug::emit_debug_mixin,
        eq_hash::{emit_eq, emit_hash_code, emit_shared_helpers},
    },
};

pub(crate) fn emit_library(library: &LibraryIr, _plan: &SymbolPlan) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let copyable_types = library
        .classes
        .iter()
        .filter(|class| {
            class
                .traits
                .iter()
                .any(|trait_app| trait_app.symbol.0 == "derive_annotation::CopyWith")
        })
        .map(|class| class.name.clone())
        .chain(
            _plan
                .workspace_string_set(COPYABLE_TYPES_ANALYSIS_KEY)
                .into_iter()
                .flatten()
                .cloned(),
        )
        .collect::<HashSet<_>>();
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
        if let Some(copy_with) = emit_copy_with(class, &copyable_types) {
            contribution.push_mixin_member(&class.name, copy_with);
        }
    }

    contribution
}
