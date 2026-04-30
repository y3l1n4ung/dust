use std::collections::HashSet;

use heck::{AsKebabCase, AsLowerCamelCase, AsPascalCase, AsSnakeCase, AsTrainCase};

use dust_ir::{
    BuiltinType, ClassIr, ConstructorIr, FieldIr, ParamKind, SerdeFieldConfigIr, SerdeRenameRuleIr,
    TypeIr,
};

/// Finds a constructor that can be used for deserialization.
///
/// A suitable constructor must have parameters that correspond to all fields
/// of the class, accounting for positional and named parameters.
pub(crate) fn find_deserialize_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.can_construct_all_fields(&class.fields))
}

/// Computes the JSON key for a field, respecting custom renames or class-wide rules.
pub(crate) fn json_key(
    class: &ClassIr,
    field_name: &str,
    field_serde: Option<&SerdeFieldConfigIr>,
) -> String {
    if let Some(rename) = field_serde.and_then(|serde| serde.rename.as_deref()) {
        return rename.to_owned();
    }

    match class.serde.as_ref().and_then(|serde| serde.rename_all) {
        Some(rule) => apply_rename_rule(field_name, rule),
        None => field_name.to_owned(),
    }
}

/// Returns all possible keys that are allowed for a class during deserialization.
///
/// This includes the primary keys (after renaming) and any explicitly defined aliases.
pub(crate) fn all_allowed_keys(class: &ClassIr) -> Vec<String> {
    let mut keys = Vec::new();
    let mut seen = HashSet::new();
    for field in &class.fields {
        let Some(serde) = &field.serde else {
            let key = json_key(class, &field.name, None);
            if seen.insert(key.clone()) {
                keys.push(key);
            }
            continue;
        };

        let key = json_key(class, &field.name, Some(serde));
        if seen.insert(key.clone()) {
            keys.push(key);
        }
        for alias in &serde.aliases {
            if seen.insert(alias.clone()) {
                keys.push(alias.clone());
            }
        }
    }
    keys
}

/// Renders a Dart constructor call for deserialization.
pub(crate) fn render_constructor_call(
    class: &ClassIr,
    constructor: &ConstructorIr,
    values: &[(&str, String)],
) -> Option<String> {
    let mut positional = Vec::new();
    let mut named = Vec::new();

    for param in &constructor.params {
        let value = values
            .iter()
            .find(|(name, _)| *name == param.name)
            .map(|(_, value)| value.clone());

        let Some(value) = value else {
            if param.has_default {
                continue;
            }
            return None;
        };

        match param.kind {
            ParamKind::Positional => positional.push(value),
            ParamKind::Named => named.push(format!("{}: {}", param.name, value)),
        }
    }

    let ctor = match &constructor.name {
        Some(name) => format!("{}.{}", class.name, name),
        None => class.name.clone(),
    };

    let mut args = positional;
    args.extend(named);
    if args.is_empty() {
        return Some(format!("{ctor}()"));
    }

    let rendered_args = args
        .into_iter()
        .map(|arg| format!("  {arg},"))
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!("{ctor}(\n{rendered_args}\n)"))
}

/// Generates a Dart expression to encode a specific field value to JSON.
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

/// Generates a Dart expression to encode a generic type value to JSON.
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

fn nullable_identity_encode(ty: &TypeIr) -> bool {
    matches!(ty, TypeIr::Builtin { nullable: true, .. } | TypeIr::Dynamic)
}

/// Generates a Dart expression to decode a JSON value into a specific field.
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
        None => decode_expr(raw, key, &field.ty, deserializable_classes, deserializable_enums),
    }
}

/// Generates a Dart expression to decode a JSON value into a specific type.
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

fn encode_non_nullable_expr(
    expr: &str,
    ty: &TypeIr,
    serializable_classes: &HashSet<String>,
    serializable_enums: &HashSet<String>,
) -> String {
    match ty {
        // Built-ins and dynamics map directly to JSON-compatible types.
        TypeIr::Builtin { .. } | TypeIr::Dynamic | TypeIr::Unknown => expr.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => expr.to_owned(),
        
        // Special standard types with built-in conversions.
        TypeIr::Named { name, .. } if name.as_ref() == "DateTime" => {
            format!("{expr}.toIso8601String()")
        }
        TypeIr::Named { name, .. } if name.as_ref() == "Uri" || name.as_ref() == "BigInt" => {
            format!("{expr}.toString()")
        }
        
        // Collections are recursively encoded.
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
        
        // Named types use generated helpers if available, or fall back to `toJson()`.
        TypeIr::Named { name, .. } => {
            if serializable_classes.contains(name.as_ref()) || serializable_enums.contains(name.as_ref()) {
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
        // Scalar types use shared helper functions for type safety.
        TypeIr::Builtin { kind, .. } => match kind {
            BuiltinType::String => {
                format!("_dustJsonAs<String>({raw}, {key}, 'String')")
            }
            BuiltinType::Int => format!("_dustJsonAs<int>({raw}, {key}, 'int')"),
            BuiltinType::Bool => format!("_dustJsonAs<bool>({raw}, {key}, 'bool')"),
            BuiltinType::Double => format!("_dustJsonAs<num>({raw}, {key}, 'num').toDouble()"),
            BuiltinType::Num => format!("_dustJsonAs<num>({raw}, {key}, 'num')"),
            BuiltinType::Object => format!("_dustJsonAs<Object>({raw}, {key}, 'Object')"),
        },
        TypeIr::Dynamic | TypeIr::Unknown => raw.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => raw.to_owned(),
        
        // Conversions for standard non-JSON types.
        TypeIr::Named { name, .. } if name.as_ref() == "DateTime" => {
            format!("_dustJsonAsDateTime({raw}, {key})")
        }
        TypeIr::Named { name, .. } if name.as_ref() == "Uri" => {
            format!("_dustJsonAsUri({raw}, {key})")
        }
        TypeIr::Named { name, .. } if name.as_ref() == "BigInt" => {
            format!("_dustJsonAsBigInt({raw}, {key})")
        }
        
        // Collections are recursively decoded and re-cast.
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" => format!(
            "_dustJsonAsList({raw}, {key}).map((item) => {}).toList()",
            decode_expr("item", key, &args[0], deserializable_classes, deserializable_enums)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Set" => format!(
            "_dustJsonAsList({raw}, {key}).map((item) => {}).toSet()",
            decode_expr("item", key, &args[0], deserializable_classes, deserializable_enums)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Map" => format!(
            "_dustJsonAsMap({raw}, {key}).map((mapKey, value) => MapEntry(mapKey, {}))",
            decode_expr("value", key, &args[1], deserializable_classes, deserializable_enums)
        ),
        
        // Named types use generated helpers if available, or fall back to factory `fromJson()`.
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

/// Applies a custom codec for encoding.
fn encode_with_codec(expr: &str, ty: &TypeIr, codec: &str) -> String {
    let codec = access_receiver(codec);
    if ty.is_nullable() {
        let encoded = format!("{codec}.serialize({expr}!)");
        return format!("{expr} == null\n? null\n: {encoded}");
    }

    format!("{codec}.serialize({expr})")
}

/// Applies a custom codec for decoding.
fn decode_with_codec(raw: &str, key: &str, ty: &TypeIr, codec: &str) -> String {
    let codec = access_receiver(codec);
    let value_ty = render_type(&non_nullable(ty));
    if ty.is_nullable() {
        let decoded = format!("_dustJsonDecodeWithCodec<{value_ty}>({codec}, {raw}, {key})");
        return format!("{raw} == null\n? null\n: {decoded}");
    }

    format!("_dustJsonDecodeWithCodec<{value_ty}>({codec}, {raw}, {key})")
}

/// Helper to get the non-nullable version of a type.
fn non_nullable(ty: &TypeIr) -> TypeIr {
    match ty {
        TypeIr::Builtin { kind, .. } => TypeIr::builtin(*kind),
        TypeIr::Named { name, args, .. } => TypeIr::generic(name.as_ref(), args.to_vec()),
        TypeIr::Function { signature, .. } => TypeIr::function(signature.as_ref()),
        TypeIr::Record { shape, .. } => TypeIr::record(shape.as_ref()),
        TypeIr::Dynamic => TypeIr::dynamic(),
        TypeIr::Unknown => TypeIr::unknown(),
    }
}

fn non_null_encode_expr(expr: &str) -> String {
    format!("({expr}!)")
}

/// Renders a semantic `TypeIr` into a Dart type string.
pub(crate) fn render_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => {
            let nullable = if *nullable { "?" } else { "" };
            format!("{}{}", kind.as_str(), nullable)
        }
        TypeIr::Named {
            name,
            args,
            nullable,
        } => {
            let args = if args.is_empty() {
                String::new()
            } else {
                format!(
                    "<{}>",
                    args.iter().map(render_type).collect::<Vec<_>>().join(", ")
                )
            };
            let nullable = if *nullable { "?" } else { "" };
            format!("{name}{args}{nullable}")
        }
        TypeIr::Function {
            signature,
            nullable,
        } => {
            let nullable = if *nullable { "?" } else { "" };
            format!("{signature}{nullable}")
        }
        TypeIr::Record { shape, nullable } => {
            let nullable = if *nullable { "?" } else { "" };
            format!("{shape}{nullable}")
        }
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Unknown => "Object?".to_owned(),
    }
}

/// Normalizes access to a Dart receiver (adding parentheses if needed).
fn access_receiver(source: &str) -> String {
    if is_simple_receiver(source) {
        source.to_owned()
    } else {
        format!("({source})")
    }
}

fn is_simple_receiver(source: &str) -> bool {
    !source.is_empty()
        && source
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '.'))
}

/// Translates an IR rename rule into a `heck` case conversion call.
pub(crate) fn apply_rename_rule(source: &str, rule: SerdeRenameRuleIr) -> String {
    match rule {
        SerdeRenameRuleIr::LowerCase => source.to_lowercase(),
        SerdeRenameRuleIr::UpperCase => source.to_uppercase(),
        SerdeRenameRuleIr::PascalCase => AsPascalCase(source).to_string(),
        SerdeRenameRuleIr::CamelCase => AsLowerCamelCase(source).to_string(),
        SerdeRenameRuleIr::SnakeCase => AsSnakeCase(source).to_string(),
        SerdeRenameRuleIr::ScreamingSnakeCase => AsSnakeCase(source).to_string().to_uppercase(),
        SerdeRenameRuleIr::KebabCase => AsKebabCase(source).to_string(),
        SerdeRenameRuleIr::ScreamingKebabCase => AsTrainCase(source).to_string().to_uppercase(),
    }
}
