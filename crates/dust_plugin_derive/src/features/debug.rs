use dust_ir::{ClassIr, SymbolId};

use crate::features::eq_hash::has_trait;

pub(crate) fn emit_debug_mixin(class: &ClassIr) -> Option<String> {
    if !has_trait(class, &SymbolId::new("derive_annotation::Debug")) {
        return None;
    }

    let fields = if class.fields.is_empty() {
        String::new()
    } else {
        class
            .fields
            .iter()
            .map(|field| format!("{}: ${{_dustSelf.{}}}", field.name, field.name))
            .collect::<Vec<_>>()
            .join(", ")
    };

    Some(if fields.is_empty() {
        format!("@override\nString toString() => '{}()';", class.name)
    } else {
        format!(
            "@override\nString toString() => '{}({})';",
            class.name, fields
        )
    })
}
