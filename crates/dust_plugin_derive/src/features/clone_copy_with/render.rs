use std::borrow::Cow;

use dust_dart_emit::{
    DART_ITERABLE, DART_LIST, DART_MAP, DART_OBJECT_NULLABLE, DART_SET, OBJECT_NULLABLE_TYPES,
};
use dust_ir::{BuiltinType, TypeIr};

use super::support::{member_access_expr, non_null_value_expr};

pub(crate) struct CopyableTypes<'a> {
    local: &'a [&'a str],
    workspace: Option<&'a [String]>,
}

impl<'a> CopyableTypes<'a> {
    pub(crate) fn new(local: &'a [&'a str], workspace: Option<&'a [String]>) -> Self {
        Self { local, workspace }
    }

    fn contains(&self, name: &str) -> bool {
        self.local.contains(&name)
            || self.workspace.is_some_and(|values| {
                values
                    .binary_search_by(|value| value.as_str().cmp(name))
                    .is_ok()
            })
    }
}

pub(super) fn render_copied_value<'a>(
    ty: &TypeIr,
    value: &'a str,
    depth: usize,
    copyable_types: &CopyableTypes<'_>,
) -> Cow<'a, str> {
    match ty {
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == DART_LIST => Cow::Owned(render_sequence_copy(
            DART_LIST,
            args,
            *nullable,
            value,
            depth,
            copyable_types,
        )),
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == DART_SET => Cow::Owned(render_sequence_copy(
            DART_SET,
            args,
            *nullable,
            value,
            depth,
            copyable_types,
        )),
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == DART_MAP => Cow::Owned(render_map_copy(
            args,
            *nullable,
            value,
            depth,
            copyable_types,
        )),
        TypeIr::Named {
            name,
            args,
            nullable,
        } if name.as_ref() == DART_ITERABLE => Cow::Owned(render_sequence_copy(
            DART_LIST,
            args,
            *nullable,
            value,
            depth,
            copyable_types,
        )),
        TypeIr::Named { name, nullable, .. } if copyable_types.contains(name.as_ref()) => {
            Cow::Owned(render_named_copy(*nullable, value))
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
        | TypeIr::Named { .. } => Cow::Borrowed(value),
    }
}

fn render_sequence_copy(
    container: &str,
    args: &[TypeIr],
    nullable: bool,
    value: &str,
    depth: usize,
    copyable_types: &CopyableTypes<'_>,
) -> String {
    let item_ty = args.first();
    let item_rendered = item_ty
        .map(|ty| OBJECT_NULLABLE_TYPES.render(ty))
        .unwrap_or_else(|| DART_OBJECT_NULLABLE.to_owned());
    let source_value = if nullable {
        non_null_value_expr(value)
    } else {
        value.to_owned()
    };
    let item_binding = format!("item_{depth}");
    let mapped_value = if let Some(item_ty) = item_ty {
        let copied_item = render_copied_value(item_ty, &item_binding, depth + 1, copyable_types);
        if copied_item == item_binding {
            Cow::Borrowed(source_value.as_str())
        } else {
            Cow::Owned(format!(
                "{}.map(({item_binding}) => {copied_item})",
                member_access_expr(&source_value)
            ))
        }
    } else {
        Cow::Borrowed(source_value.as_str())
    };

    let body = if mapped_value.as_ref() == source_value {
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
    copyable_types: &CopyableTypes<'_>,
) -> String {
    let key_ty = args.first();
    let value_ty = args.get(1);
    let key_rendered = key_ty
        .map(|ty| OBJECT_NULLABLE_TYPES.render(ty))
        .unwrap_or_else(|| DART_OBJECT_NULLABLE.to_owned());
    let value_rendered = value_ty
        .map(|ty| OBJECT_NULLABLE_TYPES.render(ty))
        .unwrap_or_else(|| DART_OBJECT_NULLABLE.to_owned());
    let source_value = if nullable {
        non_null_value_expr(value)
    } else {
        value.to_owned()
    };
    let entry_binding = format!("entry_{depth}");
    let key_binding = format!("{entry_binding}.key");
    let value_binding = format!("{entry_binding}.value");
    let key_expr = key_ty
        .map(|ty| render_copied_value(ty, &key_binding, depth + 1, copyable_types))
        .unwrap_or_else(|| Cow::Borrowed(key_binding.as_str()));
    let value_expr = value_ty
        .map(|ty| render_copied_value(ty, &value_binding, depth + 1, copyable_types))
        .unwrap_or_else(|| Cow::Borrowed(value_binding.as_str()));

    let body = if key_expr.as_ref() == key_binding && value_expr.as_ref() == value_binding {
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
