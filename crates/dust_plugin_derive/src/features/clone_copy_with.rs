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

pub(crate) fn emit_copy_with(class: &ClassIr) -> Option<String> {
    let copy_with = SymbolId::new("derive_annotation::CopyWith");

    if !has_trait(class, &copy_with) {
        return None;
    }

    let constructor = find_clone_constructor(class)?;
    let params = render_copy_with_params(class);
    let setup = class
        .fields
        .iter()
        .enumerate()
        .flat_map(|(depth, field)| {
            let source_var = temp_name("next", &field.name, "Source");
            let source_expr = render_copy_with_source_expr(field.name.as_str(), &field.ty);
            let next_var = temp_name("next", &field.name, "");
            let copied_expr = render_clone_value(&field.ty, source_var.as_str(), depth);

            let mut lines = vec![format!("final {source_var} = {source_expr};")];
            if copied_expr == source_var {
                lines.push(format!("final {next_var} = {source_var};"));
            } else {
                lines.push(format!("final {next_var} = {copied_expr};"));
            }
            lines
        })
        .collect::<Vec<_>>();
    let call = build_constructor_call_multiline(
        class,
        constructor,
        &class
            .fields
            .iter()
            .map(|field| (field.name.as_str(), temp_name("next", &field.name, "")))
            .collect::<Vec<_>>(),
    )?;
    let setup = setup
        .into_iter()
        .map(|line| format!("  {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let return_call = render_return_statement(&call, "  ");

    Some(format!(
        "{name} copyWith({params}) {{\n{setup}\n\n{return_call}\n}}",
        name = class.name,
        params = params,
        setup = setup,
        return_call = return_call
    ))
}

pub(crate) fn emit_clone(class: &ClassIr) -> Option<String> {
    let clone = SymbolId::new("derive_annotation::Clone");
    if !has_trait(class, &clone) {
        return None;
    }

    let constructor = find_clone_constructor(class)?;
    let setup = class
        .fields
        .iter()
        .enumerate()
        .map(|(depth, field)| {
            let value = format!("_dustSelf.{}", field.name);
            format!(
                "  final {} = {};",
                temp_name("cloned", &field.name, ""),
                render_clone_value(&field.ty, value.as_str(), depth)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let call = build_constructor_call_multiline(
        class,
        constructor,
        &class
            .fields
            .iter()
            .map(|field| (field.name.as_str(), temp_name("cloned", &field.name, "")))
            .collect::<Vec<_>>(),
    )?;
    let return_call = render_return_statement(&call, "  ");

    Some(format!(
        "{name} clone() {{\n{setup}\n\n{return_call}\n}}",
        name = class.name,
        setup = setup,
        return_call = return_call
    ))
}

pub(crate) fn validate_clone_copy_with(class: &ClassIr) -> Vec<Diagnostic> {
    let clone = SymbolId::new("derive_annotation::Clone");
    let copy_with = SymbolId::new("derive_annotation::CopyWith");

    if !has_trait(class, &clone) && !has_trait(class, &copy_with) {
        return Vec::new();
    }

    if class.is_abstract {
        return vec![Diagnostic::error(format!(
            "`Clone`/`CopyWith` cannot target abstract class `{}` because Dust cannot instantiate it",
            class.name
        ))];
    }

    if find_clone_constructor(class).is_some() {
        Vec::new()
    } else {
        vec![Diagnostic::error(format!(
            "`Clone`/`CopyWith` requires a constructor that accepts every field on class `{}`",
            class.name
        ))]
    }
}

fn render_clone_value(ty: &TypeIr, value: &str, depth: usize) -> String {
    match ty {
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "List" => {
            render_sequence_clone("List", args, *nullable, value, depth)
        }
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "Set" => render_sequence_clone("Set", args, *nullable, value, depth),
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "Map" => render_map_clone(args, *nullable, value, depth),
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == "Iterable" => {
            render_sequence_clone("List", args, *nullable, value, depth)
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

fn render_sequence_clone(
    container: &str,
    args: &[TypeIr],
    nullable: bool,
    value: &str,
    depth: usize,
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
        let cloned_item = render_clone_value(item_ty, &item_binding, depth + 1);
        if cloned_item == item_binding {
            source_value.clone()
        } else {
            format!("{source_value}.map(({item_binding}) => {cloned_item})")
        }
    } else {
        source_value.clone()
    };

    wrap_nullable_clone(
        nullable,
        value,
        format!("{container}<{item_rendered}>.of({mapped_value})"),
    )
}

fn render_map_clone(args: &[TypeIr], nullable: bool, value: &str, depth: usize) -> String {
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
        .map(|ty| render_clone_value(ty, &format!("{entry_binding}.key"), depth + 1))
        .unwrap_or_else(|| format!("{entry_binding}.key"));
    let value_expr = value_ty
        .map(|ty| render_clone_value(ty, &format!("{entry_binding}.value"), depth + 1))
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

    wrap_nullable_clone(nullable, value, body)
}

fn wrap_nullable_clone(nullable: bool, original: &str, cloned: String) -> String {
    if nullable {
        format!("{original} == null ? null : {cloned}")
    } else {
        cloned
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
