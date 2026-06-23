use dust_diagnostics::Diagnostic;
use dust_ir::{ClassKindIr, DartFileIr};

use crate::features::{
    clone_copy_with::validate_copy_with, eq_hash::validate_eq_hash, validate::validate_validate,
};

/// Validates derive requests for every class in a library.
pub(crate) fn validate_library(library: &DartFileIr) -> Vec<Diagnostic> {
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
        diagnostics.extend(validate_validate(library, class));
    }
    diagnostics
}
