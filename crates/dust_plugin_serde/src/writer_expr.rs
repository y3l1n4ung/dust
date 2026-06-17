use dust_dart_emit::{
    DART_BIG_INT, DART_BOOL, DART_DATE_TIME, DART_INT, DART_LIST, DART_MAP, DART_NUM, DART_OBJECT,
    DART_SET, DART_STRING, DART_URI, OBJECT_NULLABLE_TYPES,
};
use dust_ir::{BuiltinType, FieldIr, TypeIr};

use crate::writer_type::{access_receiver, non_null_encode_expr, non_nullable};

pub(crate) fn encode_field_expr(
    expr: &str,
    field: &FieldIr,
    serializable_classes: &[&str],
    serializable_enums: &[&str],
) -> String {
    match field
        .serde
        .as_ref()
        .and_then(|serde| serde.codec_source.as_deref())
    {
        Some(codec) => encode_with_codec(expr, &field.ty, codec),
        None => encode_expr(expr, &field.ty, serializable_classes, serializable_enums),
    }
}

pub(crate) fn encode_expr(
    expr: &str,
    ty: &TypeIr,
    serializable_classes: &[&str],
    serializable_enums: &[&str],
) -> String {
    if nullable_identity_encode(ty) {
        return expr.to_owned();
    }

    if ty.is_nullable() {
        let inner = non_nullable(ty);
        let encoded = encode_non_nullable_expr(
            &non_null_encode_expr(expr),
            &inner,
            serializable_classes,
            serializable_enums,
        );
        return format!("{expr} == null\n    ? null\n    : {encoded}");
    }

    encode_non_nullable_expr(expr, ty, serializable_classes, serializable_enums)
}

pub(crate) fn decode_field_expr(
    raw: &str,
    key: &str,
    field: &FieldIr,
    deserializable_classes: &[&str],
    deserializable_enums: &[&str],
) -> String {
    match field
        .serde
        .as_ref()
        .and_then(|serde| serde.codec_source.as_deref())
    {
        Some(codec) => decode_with_codec(raw, key, &field.ty, codec),
        None => decode_expr(
            raw,
            key,
            &field.ty,
            deserializable_classes,
            deserializable_enums,
        ),
    }
}

pub(crate) fn decode_expr(
    raw: &str,
    key: &str,
    ty: &TypeIr,
    deserializable_classes: &[&str],
    deserializable_enums: &[&str],
) -> String {
    if ty.is_nullable() {
        let inner = non_nullable(ty);
        let decoded = decode_non_nullable_expr(
            raw,
            key,
            &inner,
            deserializable_classes,
            deserializable_enums,
        );
        return format!("{raw} == null\n    ? null\n    : {decoded}");
    }

    decode_non_nullable_expr(raw, key, ty, deserializable_classes, deserializable_enums)
}

fn nullable_identity_encode(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Builtin { nullable: true, .. } | TypeIr::Dynamic)
}

fn encode_non_nullable_expr(
    expr: &str,
    ty: &TypeIr,
    serializable_classes: &[&str],
    serializable_enums: &[&str],
) -> String {
    match ty {
        TypeIr::Builtin { .. } | TypeIr::Dynamic | TypeIr::Unknown => expr.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => expr.to_owned(),
        TypeIr::Named { name, .. } if name.as_ref() == DART_DATE_TIME => {
            format!("{expr}.toIso8601String()")
        }
        TypeIr::Named { name, .. }
            if name.as_ref() == DART_URI || name.as_ref() == DART_BIG_INT =>
        {
            format!("{expr}.toString()")
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_LIST => format!(
            "{expr}\n    .map((item) => {})\n    .toList()",
            encode_expr("item", &args[0], serializable_classes, serializable_enums)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_SET => format!(
            "{expr}\n    .map((item) => {})\n    .toList()",
            encode_expr("item", &args[0], serializable_classes, serializable_enums)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_MAP => {
            let value_expr =
                encode_expr("value", &args[1], serializable_classes, serializable_enums);
            if value_expr.contains('\n') {
                format!(
                    "{expr}\n    .map(\n      (key, value) => MapEntry(\n        key,\n{},\n      ),\n    )",
                    indent_expr(&value_expr, 8)
                )
            } else {
                format!("{expr}\n    .map((key, value) => MapEntry(key, {value_expr}))")
            }
        }
        TypeIr::Named { name, .. } => {
            if contains_symbol(serializable_classes, name.as_ref())
                || contains_symbol(serializable_enums, name.as_ref())
            {
                format!("_${name}ToJson({expr})")
            } else {
                format!("{expr}.toJson()")
            }
        }
    }
}

fn decode_non_nullable_expr(
    raw: &str,
    key: &str,
    ty: &TypeIr,
    deserializable_classes: &[&str],
    deserializable_enums: &[&str],
) -> String {
    match ty {
        TypeIr::Builtin { kind, .. } => match kind {
            BuiltinType::String => {
                format!("JsonHelper.as<{DART_STRING}>({raw}, {key}, '{DART_STRING}')")
            }
            BuiltinType::Int => format!("JsonHelper.as<{DART_INT}>({raw}, {key}, '{DART_INT}')"),
            BuiltinType::Bool => {
                format!("JsonHelper.as<{DART_BOOL}>({raw}, {key}, '{DART_BOOL}')")
            }
            BuiltinType::Double => {
                format!("JsonHelper.as<{DART_NUM}>({raw}, {key}, '{DART_NUM}').toDouble()")
            }
            BuiltinType::Num => {
                format!("JsonHelper.as<{DART_NUM}>({raw}, {key}, '{DART_NUM}')")
            }
            BuiltinType::Object => {
                format!("JsonHelper.as<{DART_OBJECT}>({raw}, {key}, '{DART_OBJECT}')")
            }
        },
        TypeIr::Dynamic | TypeIr::Unknown => raw.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => raw.to_owned(),
        TypeIr::Named { name, .. } if name.as_ref() == DART_DATE_TIME => {
            format!("JsonHelper.asDateTime({raw}, {key})")
        }
        TypeIr::Named { name, .. } if name.as_ref() == DART_URI => {
            format!("JsonHelper.asUri({raw}, {key})")
        }
        TypeIr::Named { name, .. } if name.as_ref() == DART_BIG_INT => {
            format!("JsonHelper.asBigInt({raw}, {key})")
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_LIST => format!(
            "JsonHelper.asList({raw}, {key})\n    .map((item) => {})\n    .toList()",
            decode_expr(
                "item",
                key,
                &args[0],
                deserializable_classes,
                deserializable_enums
            )
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_SET => format!(
            "JsonHelper.asList({raw}, {key})\n    .map((item) => {})\n    .toSet()",
            decode_expr(
                "item",
                key,
                &args[0],
                deserializable_classes,
                deserializable_enums
            )
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_MAP => {
            let value_expr = decode_expr(
                "value",
                key,
                &args[1],
                deserializable_classes,
                deserializable_enums,
            );
            if value_expr.contains('\n') {
                format!(
                    "JsonHelper.asMap({raw}, {key})\n    .map(\n      (mapKey, value) => MapEntry(\n        mapKey,\n{},\n      ),\n    )",
                    indent_expr(&value_expr, 8)
                )
            } else {
                format!(
                    "JsonHelper.asMap({raw}, {key})\n    .map((mapKey, value) => MapEntry(mapKey, {value_expr}))"
                )
            }
        }
        TypeIr::Named { name, .. } => {
            if contains_symbol(deserializable_classes, name.as_ref()) {
                format!("_${name}FromJson(JsonHelper.asMap({raw}, {key}))")
            } else if contains_symbol(deserializable_enums, name.as_ref()) {
                format!("_${name}FromJson({raw})")
            } else {
                format!("{name}.fromJson(JsonHelper.asMap({raw}, {key}))")
            }
        }
    }
}

fn contains_symbol(symbols: &[&str], name: &str) -> bool {
    symbols.contains(&name)
}

fn indent_expr(expr: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    expr.lines()
        .map(|line| format!("{pad}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn encode_with_codec(expr: &str, ty: &TypeIr, codec: &str) -> String {
    let codec = access_receiver(codec);
    if ty.is_nullable() {
        let encoded = format!("{codec}.serialize({expr}!)");
        return format!("{expr} == null\n    ? null\n    : {encoded}");
    }

    format!("{codec}.serialize({expr})")
}

fn decode_with_codec(raw: &str, key: &str, ty: &TypeIr, codec: &str) -> String {
    let codec = access_receiver(codec);
    let value_ty = OBJECT_NULLABLE_TYPES.render(&non_nullable(ty));
    if ty.is_nullable() {
        let decoded = format!("JsonHelper.decodeWithCodec<{value_ty}>({codec}, {raw}, {key})");
        return format!("{raw} == null\n    ? null\n    : {decoded}");
    }

    format!("JsonHelper.decodeWithCodec<{value_ty}>({codec}, {raw}, {key})")
}
