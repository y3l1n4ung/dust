use std::collections::HashSet;

use dust_ir::{BuiltinType, TypeIr};

use crate::features::writer::render_type;

use super::support::{member_access_expr, non_null_value_expr};

pub(super) fn render_copied_value(
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
            format!(
                "{}.map(({item_binding}) => {copied_item})",
                member_access_expr(&source_value)
            )
        }
    } else {
        source_value.clone()
    };

    let body = if mapped_value == source_value {
        format!("{container}<{item_rendered}>.of({mapped_value})")
    } else {
        format!("{container}<{item_rendered}>.of(\n  {mapped_value},\n)")
    };

    wrap_nullable_copy(nullable, value, body)
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
            "Map<{key_rendered}, {value_rendered}>.fromEntries(\n  {}.entries.map(\n    ({entry_binding}) => MapEntry({key_expr}, {value_expr}),\n  ),\n)",
            member_access_expr(&source_value)
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

    wrap_nullable_copy(
        nullable,
        value,
        format!("{}.copyWith()", member_access_expr(&source_value)),
    )
}

fn wrap_nullable_copy(nullable: bool, original: &str, copied: String) -> String {
    if nullable {
        format!("{original} == null ? null : {copied}")
    } else {
        copied
    }
}
