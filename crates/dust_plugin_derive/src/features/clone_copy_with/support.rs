use dust_ir::{ClassIr, TypeIr};

use crate::features::writer::render_type;

pub(super) fn render_copy_with_params(class: &ClassIr) -> String {
    if class.fields.is_empty() {
        return "{}".to_owned();
    }

    let params = class
        .fields
        .iter()
        .map(|field| {
            if uses_undefined_sentinel(&field.ty) {
                format!(
                    "  {} {} = _undefined,",
                    undefined_parameter_type(&field.ty),
                    field.name
                )
            } else {
                format!(
                    "  {} {},",
                    render_copy_with_param_type(&field.ty),
                    field.name
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{{\n{params}\n}}")
}

pub(super) fn render_copy_with_source_expr(field_name: &str, ty: &TypeIr) -> String {
    if uses_undefined_sentinel(ty) {
        let cast = render_type(ty);
        format!(
            "identical({field_name}, _undefined) ? _dustSelf.{field_name} : {field_name} as {cast}"
        )
    } else {
        format!("{field_name} ?? _dustSelf.{field_name}")
    }
}

pub(super) fn uses_undefined_sentinel(ty: &TypeIr) -> bool {
    ty.is_nullable() || matches!(ty, TypeIr::Dynamic | TypeIr::Unknown)
}

pub(super) fn should_keep_source_local(ty: &TypeIr) -> bool {
    ty.is_nullable()
}

pub(super) fn render_setup_blocks(blocks: Vec<String>) -> String {
    blocks
        .into_iter()
        .map(|block| {
            block
                .lines()
                .map(|line| format!("  {line}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n")
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
            nullable_parameter_type(render_type(ty))
        }
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Unknown => "Object?".to_owned(),
    }
}

fn undefined_parameter_type(ty: &TypeIr) -> &'static str {
    if matches!(ty, TypeIr::Dynamic) {
        "dynamic"
    } else {
        "Object?"
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
