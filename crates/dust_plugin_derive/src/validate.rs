use dust_diagnostics::Diagnostic;
use dust_ir::{ClassKindIr, LibraryIr};

use crate::features::{clone_copy_with::validate_clone_copy_with, eq_hash::validate_eq_hash};

pub(crate) fn validate_library(library: &LibraryIr) -> Vec<Diagnostic> {
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
        diagnostics.extend(validate_clone_copy_with(class));
    }
    diagnostics
}
