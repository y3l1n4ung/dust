use std::fmt::Write;

use dust_dart_emit::{DART_DYNAMIC, DART_OBJECT_NULLABLE, OBJECT_NULLABLE_TYPES};
use dust_ir::{ClassIr, TypeIr};

pub(super) fn render_copy_with_params(class: &ClassIr) -> String {
    if class.fields.is_empty() {
        return "{}".to_owned();
    }

    let mut params = String::with_capacity(class.fields.len() * 32);
    params.push_str("{\n");
    for field in &class.fields {
        if uses_undefined_sentinel(&field.ty) {
            writeln!(
                params,
                "  {} {} = _undefined,",
                undefined_parameter_type(&field.ty),
                field.name
            )
            .expect("writing to String cannot fail");
        } else {
            writeln!(
                params,
                "  {} {},",
                render_copy_with_param_type(&field.ty),
                field.name
            )
            .expect("writing to String cannot fail");
        }
    }
    params.push('}');
    params
}

pub(super) fn render_copy_with_source_expr(field_name: &str, ty: &TypeIr) -> String {
    if uses_undefined_sentinel(ty) {
        let cast = OBJECT_NULLABLE_TYPES.render(ty);
        format!(
            "identical({field_name}, _undefined)\n    ? self.{field_name}\n    : {field_name} as {cast}"
        )
    } else {
        format!("{field_name} ?? self.{field_name}")
    }
}

pub(super) fn uses_undefined_sentinel(ty: &TypeIr) -> bool {
    ty.is_nullable() || matches!(ty, TypeIr::Dynamic | TypeIr::Unknown)
}

pub(super) fn should_keep_source_local(ty: &TypeIr) -> bool {
    ty.is_nullable()
}

pub(super) fn render_setup_blocks(blocks: Vec<String>) -> String {
    let mut out = String::new();
    for (block_index, block) in blocks.into_iter().enumerate() {
        if block_index > 0 {
            out.push('\n');
        }
        for (line_index, line) in block.lines().enumerate() {
            if line_index > 0 {
                out.push('\n');
            }
            out.push_str("  ");
            out.push_str(line);
        }
    }
    out
}

pub(super) fn non_null_value_expr(value: &str) -> String {
    if is_simple_identifier(value) {
        value.to_owned()
    } else {
        format!("{value}!")
    }
}

pub(super) fn member_access_expr(value: &str) -> String {
    if is_simple_identifier(value) {
        value.to_owned()
    } else {
        format!("({value})")
    }
}

pub(super) fn temp_name(prefix: &str, field_name: &str, suffix: &str) -> String {
    format!("{prefix}{}{suffix}", upper_camel_suffix(field_name))
}

fn render_copy_with_param_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, .. } => nullable_parameter_type(kind.as_str().to_owned()),
        TypeIr::Named { .. } | TypeIr::Function { .. } | TypeIr::Record { .. } => {
            nullable_parameter_type(OBJECT_NULLABLE_TYPES.render(ty))
        }
        TypeIr::Dynamic => DART_DYNAMIC.to_owned(),
        TypeIr::Unknown => DART_OBJECT_NULLABLE.to_owned(),
    }
}

fn undefined_parameter_type(ty: &TypeIr) -> &'static str {
    if matches!(ty, TypeIr::Dynamic) {
        DART_DYNAMIC
    } else {
        DART_OBJECT_NULLABLE
    }
}

fn nullable_parameter_type(rendered: String) -> String {
    if rendered.ends_with('?') {
        rendered
    } else {
        format!("{rendered}?")
    }
}

fn is_simple_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    chars.all(|ch| ch == '_' || ch.is_ascii_alphanumeric())
}

fn upper_camel_suffix(field_name: &str) -> String {
    let mut chars = field_name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut rendered = first.to_uppercase().collect::<String>();
    rendered.push_str(chars.as_str());
    rendered
}
