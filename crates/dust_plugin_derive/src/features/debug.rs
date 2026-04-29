use dust_ir::{ClassIr, SymbolId};

use crate::features::eq_hash::has_trait;

pub(crate) fn emit_debug_mixin(class: &ClassIr) -> Option<String> {
    if !has_to_string_trait(class) {
        return None;
    }

    Some(if class.fields.is_empty() {
        format!(
            "@override\nString toString() {{\n  return '{}()';\n}}",
            class.name
        )
    } else {
        let segments = class
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let suffix = if index + 1 == class.fields.len() {
                    ""
                } else {
                    ", "
                };
                format!(
                    "      '{}: ${{_dustSelf.{}}}{}'",
                    field.name, field.name, suffix
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!(
            "@override\nString toString() {{\n  return '{}('\n{}\n      ')';\n}}",
            class.name, segments
        )
    })
}

fn has_to_string_trait(class: &ClassIr) -> bool {
    has_trait(class, &SymbolId::new("derive_annotation::ToString"))
        || has_trait(class, &SymbolId::new("derive_annotation::Debug"))
}
