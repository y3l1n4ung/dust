use dust_diagnostics::Diagnostic;
use dust_ir::{ClassKindIr, DartFileIr};
use dust_plugin_api::SymbolPlan;

use crate::features::{
    clone_copy_with::validate_copy_with,
    eq_hash::validate_eq_hash,
    validate::{emit_flutter_form_helpers, validate_validate},
};

/// Validates derive requests for every class in a library.
pub(crate) fn validate_library(library: &DartFileIr) -> Vec<Diagnostic> {
    validate_library_inner(library, false)
}

/// Validates derive requests for every class with generation-plan context.
pub(crate) fn validate_library_with_plan(
    library: &DartFileIr,
    plan: &SymbolPlan,
) -> Vec<Diagnostic> {
    validate_library_inner(library, emit_flutter_form_helpers(plan))
}

/// Validates derive requests for every class in a library.
fn validate_library_inner(library: &DartFileIr, emit_form_helpers: bool) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for class in &library.classes {
        if matches!(class.kind, ClassKindIr::MixinClass) {
            diagnostics.push(Diagnostic::error(format!(
                "Dust derive generation does not support `mixin class` targets like `{}`",
                class.name
            )));
            continue;
        }
        diagnostics.extend(validate_eq_hash(class));
        diagnostics.extend(validate_copy_with(class));
        diagnostics.extend(validate_validate(library, class, emit_form_helpers));
    }
    diagnostics
}
