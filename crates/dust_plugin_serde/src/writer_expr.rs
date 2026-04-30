use std::collections::HashSet;

use dust_ir::{BuiltinType, FieldIr, TypeIr};

use crate::writer_type::{access_receiver, non_null_encode_expr, non_nullable, render_type};

pub(crate) fn encode_field_expr(
    expr: &str,
    field: &FieldIr,
    serializable_classes: &HashSet<String>,
    serializable_enums: &HashSet<String>,
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
    serializable_classes: &HashSet<String>,
    serializable_enums: &HashSet<String>,
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
        return format!("{expr} == null ? null : {encoded}");
    }

    encode_non_nullable_expr(expr, ty, serializable_classes, serializable_enums)
}

pub(crate) fn decode_field_expr(
    raw: &str,
    key: &str,
    field: &FieldIr,
    deserializable_classes: &HashSet<String>,
    deserializable_enums: &HashSet<String>,
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
    deserializable_classes: &HashSet<String>,
    deserializable_enums: &HashSet<String>,
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
        return format!("{raw} == null\n? null\n: {decoded}");
    }

    decode_non_nullable_expr(raw, key, ty, deserializable_classes, deserializable_enums)
}

fn nullable_identity_encode(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Builtin { nullable: true, .. } | TypeIr::Dynamic)
}

fn encode_non_nullable_expr(
    expr: &str,
    ty: &TypeIr,
    serializable_classes: &HashSet<String>,
    serializable_enums: &HashSet<String>,
) -> String {
    match ty {
        TypeIr::Builtin { .. } | TypeIr::Dynamic | TypeIr::Unknown => expr.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => expr.to_owned(),
        TypeIr::Named { name, .. } if name.as_ref() == "DateTime" => {
            format!("{expr}.toIso8601String()")
        }
        TypeIr::Named { name, .. } if name.as_ref() == "Uri" || name.as_ref() == "BigInt" => {
            format!("{expr}.toString()")
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" => format!(
            "{expr}.map((item) => {}).toList()",
            encode_expr("item", &args[0], serializable_classes, serializable_enums)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Set" => format!(
            "{expr}.map((item) => {}).toList()",
            encode_expr("item", &args[0], serializable_classes, serializable_enums)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Map" => format!(
            "{expr}.map((key, value) => MapEntry(key, {}))",
            encode_expr("value", &args[1], serializable_classes, serializable_enums)
        ),
        TypeIr::Named { name, .. } => {
            if serializable_classes.contains(name.as_ref())
                || serializable_enums.contains(name.as_ref())
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
    deserializable_classes: &HashSet<String>,
    deserializable_enums: &HashSet<String>,
) -> String {
    match ty {
        TypeIr::Builtin { kind, .. } => match kind {
            BuiltinType::String => format!("_dustJsonAs<String>({raw}, {key}, 'String')"),
            BuiltinType::Int => format!("_dustJsonAs<int>({raw}, {key}, 'int')"),
            BuiltinType::Bool => format!("_dustJsonAs<bool>({raw}, {key}, 'bool')"),
            BuiltinType::Double => format!("_dustJsonAs<num>({raw}, {key}, 'num').toDouble()"),
            BuiltinType::Num => format!("_dustJsonAs<num>({raw}, {key}, 'num')"),
            BuiltinType::Object => format!("_dustJsonAs<Object>({raw}, {key}, 'Object')"),
        },
        TypeIr::Dynamic | TypeIr::Unknown => raw.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => raw.to_owned(),
        TypeIr::Named { name, .. } if name.as_ref() == "DateTime" => {
            format!("_dustJsonAsDateTime({raw}, {key})")
        }
        TypeIr::Named { name, .. } if name.as_ref() == "Uri" => {
            format!("_dustJsonAsUri({raw}, {key})")
        }
        TypeIr::Named { name, .. } if name.as_ref() == "BigInt" => {
            format!("_dustJsonAsBigInt({raw}, {key})")
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" => format!(
            "_dustJsonAsList({raw}, {key}).map((item) => {}).toList()",
            decode_expr(
                "item",
                key,
                &args[0],
                deserializable_classes,
                deserializable_enums
            )
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Set" => format!(
            "_dustJsonAsList({raw}, {key}).map((item) => {}).toSet()",
            decode_expr(
                "item",
                key,
                &args[0],
                deserializable_classes,
                deserializable_enums
            )
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Map" => format!(
            "_dustJsonAsMap({raw}, {key}).map((mapKey, value) => MapEntry(mapKey, {}))",
            decode_expr(
                "value",
                key,
                &args[1],
                deserializable_classes,
                deserializable_enums
            )
        ),
        TypeIr::Named { name, .. } => {
            if deserializable_classes.contains(name.as_ref()) {
                format!("_${name}FromJson(_dustJsonAsMap({raw}, {key}))")
            } else if deserializable_enums.contains(name.as_ref()) {
                format!("_${name}FromJson({raw})")
            } else {
                format!("{name}.fromJson(_dustJsonAsMap({raw}, {key}))")
            }
        }
    }
}

fn encode_with_codec(expr: &str, ty: &TypeIr, codec: &str) -> String {
    let codec = access_receiver(codec);
    if ty.is_nullable() {
        let encoded = format!("{codec}.serialize({expr}!)");
        return format!("{expr} == null\n? null\n: {encoded}");
    }

    format!("{codec}.serialize({expr})")
}

fn decode_with_codec(raw: &str, key: &str, ty: &TypeIr, codec: &str) -> String {
    let codec = access_receiver(codec);
    let value_ty = render_type(&non_nullable(ty));
    if ty.is_nullable() {
        let decoded = format!("_dustJsonDecodeWithCodec<{value_ty}>({codec}, {raw}, {key})");
        return format!("{raw} == null\n? null\n: {decoded}");
    }

    format!("_dustJsonDecodeWithCodec<{value_ty}>({codec}, {raw}, {key})")
}
