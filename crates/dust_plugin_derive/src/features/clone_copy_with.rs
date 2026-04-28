use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{BuiltinType, ClassIr, SymbolId, TypeIr};

use crate::features::{
    eq_hash::has_trait,
    writer::{
        build_constructor_call_multiline, find_clone_constructor, render_return_statement,
        render_type,
    },
};

pub(crate) fn copy_with_requires_undefined(class: &ClassIr) -> bool {
    class
        .fields
        .iter()
        .any(|field| uses_undefined_sentinel(&field.ty))
}

pub(crate) fn emit_copy_with(class: &ClassIr, copyable_types: &HashSet<String>) -> Option<String> {
    let copy_with = SymbolId::new("derive_annotation::CopyWith");

    if !has_trait(class, &copy_with) {
        return None;
    }

    let constructor = find_clone_constructor(class)?;
    let params = render_copy_with_params(class);
    let mut setup = Vec::new();
    let mut values = Vec::with_capacity(class.fields.len());

    for (depth, field) in class.fields.iter().enumerate() {
        let source_var = temp_name("next", &field.name, "Source");
        let source_expr = render_copy_with_source_expr(field.name.as_str(), &field.ty);
        let copied_expr =
            render_copied_value(&field.ty, source_var.as_str(), depth, copyable_types);

        if copied_expr == source_var {
            values.push((field.name.as_str(), source_expr));
        } else {
            let next_var = temp_name("next", &field.name, "");
            setup.push(format!("final {source_var} = {source_expr};"));
            setup.push(format!("final {next_var} = {copied_expr};"));
            values.push((field.name.as_str(), next_var));
        }
    }

    let call = build_constructor_call_multiline(class, constructor, &values)?;
    let setup = if setup.is_empty() {
        String::new()
    } else {
        setup
            .into_iter()
            .map(|line| format!("  {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let return_call = render_return_statement(&call, "  ");

    let body = if setup.is_empty() {
        return_call
    } else {
        format!("{setup}\n\n{return_call}")
    };

    Some(format!(
        "{name} copyWith({params}) {{\n{body}\n}}",
        name = class.name,
        params = params,
        body = body
    ))
}

pub(crate) fn validate_copy_with(class: &ClassIr) -> Vec<Diagnostic> {
    let copy_with = SymbolId::new("derive_annotation::CopyWith");

    if !has_trait(class, &copy_with) {
        return Vec::new();
    }

    if class.is_abstract {
        return vec![Diagnostic::error(format!(
            "`CopyWith` cannot target abstract class `{}` because Dust cannot instantiate it",
            class.name
        ))];
    }

    if find_clone_constructor(class).is_some() {
        Vec::new()
    } else {
        vec![Diagnostic::error(format!(
            "`CopyWith` requires a constructor that accepts every field on class `{}`",
            class.name
        ))]
    }
}

fn render_copied_value(
    ty: &TypeIr,
    value: &str,
    depth: usize,
    copyable_types: &HashSet<String>,
) -> String {
    match ty {
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "List" => {
            render_sequence_copy("List", args, *nullable, value, depth, copyable_types)
        }
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "Set" => {
            render_sequence_copy("Set", args, *nullable, value, depth, copyable_types)
        }
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "Map" => {
            render_map_copy(args, *nullable, value, depth, copyable_types)
        }
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "Iterable" => {
            render_sequence_copy("List", args, *nullable, value, depth, copyable_types)
        }
        TypeIr::Named { name, nullable, .. } if copyable_types.contains(name.as_ref()) => {
            render_named_copy(*nullable, value)
        }
        TypeIr::Builtin {
            kind: BuiltinType::Object,
            nullable: true,
        }
        | TypeIr::Dynamic
        | TypeIr::Unknown
        | TypeIr::Builtin { .. }
        | TypeIr::Function { .. }
        | TypeIr::Record { .. }
        | TypeIr::Named { .. } => value.to_owned(),
    }
}

fn render_sequence_copy(
    container: &str,
    args: &[TypeIr],
    nullable: bool,
    value: &str,
    depth: usize,
    copyable_types: &HashSet<String>,
) -> String {
    let item_ty = args.first();
    let item_rendered = item_ty
        .map(render_type)
        .unwrap_or_else(|| "Object?".to_owned());
    let source_value = if nullable {
        non_null_value_expr(value)
    } else {
        value.to_owned()
    };
    let item_binding = format!("item_{depth}");
    let mapped_value = if let Some(item_ty) = item_ty {
        let copied_item = render_copied_value(item_ty, &item_binding, depth + 1, copyable_types);
        if copied_item == item_binding {
            source_value.clone()
        } else {
            format!("{source_value}.map(({item_binding}) => {copied_item})")
        }
    } else {
        source_value.clone()
    };

    wrap_nullable_copy(
        nullable,
        value,
        format!("{container}<{item_rendered}>.of({mapped_value})"),
    )
}

fn render_map_copy(
    args: &[TypeIr],
    nullable: bool,
    value: &str,
    depth: usize,
    copyable_types: &HashSet<String>,
) -> String {
    let key_ty = args.first();
    let value_ty = args.get(1);
    let key_rendered = key_ty
        .map(render_type)
        .unwrap_or_else(|| "Object?".to_owned());
    let value_rendered = value_ty
        .map(render_type)
        .unwrap_or_else(|| "Object?".to_owned());
    let source_value = if nullable {
        non_null_value_expr(value)
    } else {
        value.to_owned()
    };
    let entry_binding = format!("entry_{depth}");
    let key_expr = key_ty
        .map(|ty| {
            render_copied_value(
                ty,
                &format!("{entry_binding}.key"),
                depth + 1,
                copyable_types,
            )
        })
        .unwrap_or_else(|| format!("{entry_binding}.key"));
    let value_expr = value_ty
        .map(|ty| {
            render_copied_value(
                ty,
                &format!("{entry_binding}.value"),
                depth + 1,
                copyable_types,
            )
        })
        .unwrap_or_else(|| format!("{entry_binding}.value"));

    let body = if key_expr == format!("{entry_binding}.key")
        && value_expr == format!("{entry_binding}.value")
    {
        format!("Map<{key_rendered}, {value_rendered}>.of({source_value})")
    } else {
        format!(
            "Map<{key_rendered}, {value_rendered}>.fromEntries({source_value}.entries.map(({entry_binding}) => MapEntry({key_expr}, {value_expr})))"
        )
    };

    wrap_nullable_copy(nullable, value, body)
}

fn render_named_copy(nullable: bool, value: &str) -> String {
    let source_value = if nullable {
        non_null_value_expr(value)
    } else {
        value.to_owned()
    };

    wrap_nullable_copy(nullable, value, format!("{source_value}.copyWith()"))
}

fn wrap_nullable_copy(nullable: bool, original: &str, copied: String) -> String {
    if nullable {
        format!("{original} == null ? null : {copied}")
    } else {
        copied
    }
}

fn render_copy_with_params(class: &ClassIr) -> String {
    if class.fields.is_empty() {
        return "{}".to_owned();
    }

    let params = class
        .fields
        .iter()
        .map(|field| {
            if uses_undefined_sentinel(&field.ty) {
                format!("  Object? {} = _undefined,", field.name)
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

fn render_copy_with_source_expr(field_name: &str, ty: &TypeIr) -> String {
    if uses_undefined_sentinel(ty) {
        let cast = render_type(ty);
        format!(
            "identical({field_name}, _undefined) ? _dustSelf.{field_name} : {field_name} as {cast}"
        )
    } else {
        format!("{field_name} ?? _dustSelf.{field_name}")
    }
}

fn render_copy_with_param_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, .. } => nullable_parameter_type(kind.as_str().to_owned()),
        TypeIr::Named { .. } | TypeIr::Function { .. } | TypeIr::Record { .. } => {
            nullable_parameter_type(render_type(ty))
        }
        TypeIr::Dynamic | TypeIr::Unknown => "Object?".to_owned(),
    }
}

fn uses_undefined_sentinel(ty: &TypeIr) -> bool {
    ty.is_nullable() || matches!(ty, TypeIr::Dynamic | TypeIr::Unknown)
}

fn nullable_parameter_type(rendered: String) -> String {
    if rendered.ends_with('?') {
        rendered
    } else {
        format!("{rendered}?")
    }
}

fn non_null_value_expr(value: &str) -> String {
    if is_simple_identifier(value) {
        value.to_owned()
    } else {
        format!("{value}!")
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

fn temp_name(prefix: &str, field_name: &str, suffix: &str) -> String {
    format!("{prefix}{}{suffix}", upper_camel_suffix(field_name))
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
