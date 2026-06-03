use dust_dart_emit::render_template;
use dust_ir::ClassIr;
use serde::Serialize;

use crate::features::{DEBUG_SYMBOL, TO_STRING_SYMBOL, eq_hash::has_trait};

#[derive(Serialize)]
struct EmptyDebugContext<'a> {
    class_name: &'a str,
}

#[derive(Serialize)]
struct FieldDebugContext<'a> {
    class_name: &'a str,
    lines: String,
}

pub(crate) fn emit_debug_mixin(class: &ClassIr) -> Option<String> {
    if !has_to_string_trait(class) {
        return None;
    }

    Some(if class.fields.is_empty() {
        render_template(
            "debug_empty",
            include_str!("templates/debug_empty.jinja"),
            EmptyDebugContext {
                class_name: &class.name,
            },
        )
    } else {
        render_template(
            "debug_fields",
            include_str!("templates/debug_fields.jinja"),
            FieldDebugContext {
                class_name: &class.name,
                lines: render_debug_field_lines(class),
            },
        )
    })
}

fn render_debug_field_lines(class: &ClassIr) -> String {
    class
        .fields
        .iter()
        .enumerate()
        .map(|(index, field)| {
            let suffix = if index + 1 == class.fields.len() {
                ""
            } else {
                ", "
            };
            format!("      '{}: ${{self.{}}}{}'", field.name, field.name, suffix)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn has_to_string_trait(class: &ClassIr) -> bool {
    has_trait(class, TO_STRING_SYMBOL) || has_trait(class, DEBUG_SYMBOL)
}
