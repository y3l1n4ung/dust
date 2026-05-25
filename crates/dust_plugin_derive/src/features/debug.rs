use dust_ir::ClassIr;
use std::fmt::Write;

use crate::features::{DEBUG_SYMBOL, TO_STRING_SYMBOL, eq_hash::has_trait};

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
        let mut out = String::with_capacity(class.name.len() * 2 + class.fields.len() * 32 + 96);
        write!(
            &mut out,
            "@override\nString toString() {{\n  final self = this as {};\n  return '{}('\n",
            class.name, class.name,
        )
        .ok()?;
        for (index, field) in class.fields.iter().enumerate() {
            let suffix = if index + 1 == class.fields.len() {
                ""
            } else {
                ", "
            };
            writeln!(
                &mut out,
                "      '{}: ${{self.{}}}{}'",
                field.name, field.name, suffix
            )
            .ok()?;
        }
        out.push_str("      ')';\n}");
        out
    })
}

fn has_to_string_trait(class: &ClassIr) -> bool {
    has_trait(class, TO_STRING_SYMBOL) || has_trait(class, DEBUG_SYMBOL)
}
