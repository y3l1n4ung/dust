use std::collections::HashSet;

use dust_ir::{
    BuiltinType, ClassIr, ConstructorIr, ParamKind, SerdeFieldConfigIr, SerdeRenameRuleIr, TypeIr,
};

pub(crate) fn find_deserialize_constructor<'a>(class: &'a ClassIr) -> Option<&'a ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.can_construct_all_fields(&class.fields))
}

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

pub(crate) fn encode_expr(
    expr: &str,
    ty: &TypeIr,
    serializable_models: &HashSet<String>,
) -> String {
    if nullable_identity_encode(ty) {
        return expr.to_owned();
    }

    if ty.is_nullable() {
        let inner = non_nullable(ty);
        let encoded = encode_non_nullable_expr(expr, &inner, serializable_models);
        return format!("{expr} == null ? null : {encoded}");
    }

    encode_non_nullable_expr(expr, ty, serializable_models)
}

fn nullable_identity_encode(ty: &TypeIr) -> bool {
    matches!(
        ty,
        TypeIr::Builtin { nullable: true, .. } | TypeIr::Dynamic
    )
}

pub(crate) fn decode_expr(
    raw: &str,
    ty: &TypeIr,
    deserializable_models: &HashSet<String>,
) -> String {
    if let Some(builtin) = nullable_builtin_decode(raw, ty) {
        return builtin;
    }

    if ty.is_nullable() {
        let inner = non_nullable(ty);
        let decoded = decode_non_nullable_expr(raw, &inner, deserializable_models);
        return format!("{raw} == null ? null : {decoded}");
    }

    decode_non_nullable_expr(raw, ty, deserializable_models)
}

fn nullable_builtin_decode(raw: &str, ty: &TypeIr) -> Option<String> {
    match ty {
        TypeIr::Builtin {
            kind: BuiltinType::String,
            nullable: true,
        } => Some(format!("{raw} as String?")),
        TypeIr::Builtin {
            kind: BuiltinType::Int,
            nullable: true,
        } => Some(format!("{raw} as int?")),
        TypeIr::Builtin {
            kind: BuiltinType::Bool,
            nullable: true,
        } => Some(format!("{raw} as bool?")),
        TypeIr::Builtin {
            kind: BuiltinType::Double,
            nullable: true,
        } => Some(format!("({raw} as num?)?.toDouble()")),
        TypeIr::Builtin {
            kind: BuiltinType::Num,
            nullable: true,
        } => Some(format!("{raw} as num?")),
        TypeIr::Builtin {
            kind: BuiltinType::Object,
            nullable: true,
        } => Some(format!("{raw} as Object?")),
        _ => None,
    }
}

fn encode_non_nullable_expr(
    expr: &str,
    ty: &TypeIr,
    serializable_models: &HashSet<String>,
) -> String {
    match ty {
        TypeIr::Builtin { .. } | TypeIr::Dynamic | TypeIr::Unknown => expr.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => expr.to_owned(),
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" => format!(
            "{expr}.map((item) => {}).toList()",
            encode_expr("item", &args[0], serializable_models)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Set" => format!(
            "{expr}.map((item) => {}).toList()",
            encode_expr("item", &args[0], serializable_models)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Map" => format!(
            "{expr}.map((key, value) => MapEntry(key, {}))",
            encode_expr("value", &args[1], serializable_models)
        ),
        TypeIr::Named { name, .. } => {
            if serializable_models.contains(name.as_ref()) {
                format!("_${name}ToJson({expr})")
            } else {
                format!("{expr}.toJson()")
            }
        }
    }
}

fn decode_non_nullable_expr(
    raw: &str,
    ty: &TypeIr,
    deserializable_models: &HashSet<String>,
) -> String {
    match ty {
        TypeIr::Builtin { kind, .. } => match kind {
            BuiltinType::String => format!("{raw} as String"),
            BuiltinType::Int => format!("{raw} as int"),
            BuiltinType::Bool => format!("{raw} as bool"),
            BuiltinType::Double => format!("({raw} as num).toDouble()"),
            BuiltinType::Num => format!("{raw} as num"),
            BuiltinType::Object => format!("{raw} as Object"),
        },
        TypeIr::Dynamic | TypeIr::Unknown => raw.to_owned(),
        TypeIr::Function { .. } | TypeIr::Record { .. } => raw.to_owned(),
        TypeIr::Named { name, args, .. } if name.as_ref() == "List" => format!(
            "({raw} as List<Object?>).map((item) => {}).toList()",
            decode_expr("item", &args[0], deserializable_models)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Set" => format!(
            "({raw} as List<Object?>).map((item) => {}).toSet()",
            decode_expr("item", &args[0], deserializable_models)
        ),
        TypeIr::Named { name, args, .. } if name.as_ref() == "Map" => format!(
            "Map<String, Object?>.from({raw} as Map).map((key, value) => MapEntry(key, {}))",
            decode_expr("value", &args[1], deserializable_models)
        ),
        TypeIr::Named { name, .. } => {
            if deserializable_models.contains(name.as_ref()) {
                format!("_${name}FromJson(Map<String, Object?>.from({raw} as Map))")
            } else {
                format!("{name}.fromJson(Map<String, Object?>.from({raw} as Map))")
            }
        }
    }
}

fn non_nullable(ty: &TypeIr) -> TypeIr {
    match ty {
        TypeIr::Builtin { kind, .. } => TypeIr::builtin(*kind),
        TypeIr::Named { name, args, .. } => {
            TypeIr::generic(name.as_ref(), args.iter().cloned().collect::<Vec<_>>())
        }
        TypeIr::Function { signature, .. } => TypeIr::function(signature.as_ref()),
        TypeIr::Record { shape, .. } => TypeIr::record(shape.as_ref()),
        TypeIr::Dynamic => TypeIr::dynamic(),
        TypeIr::Unknown => TypeIr::unknown(),
    }
}

fn apply_rename_rule(source: &str, rule: SerdeRenameRuleIr) -> String {
    let words = split_words(source);
    match rule {
        SerdeRenameRuleIr::LowerCase => words.join(""),
        SerdeRenameRuleIr::UpperCase => words.join("").to_ascii_uppercase(),
        SerdeRenameRuleIr::PascalCase => words
            .iter()
            .map(|word| capitalize(word))
            .collect::<Vec<_>>()
            .join(""),
        SerdeRenameRuleIr::CamelCase => {
            let mut iter = words.into_iter();
            let first = iter.next().unwrap_or_default();
            let tail = iter.map(|word| capitalize(&word)).collect::<String>();
            format!("{first}{tail}")
        }
        SerdeRenameRuleIr::SnakeCase => words.join("_"),
        SerdeRenameRuleIr::ScreamingSnakeCase => words.join("_").to_ascii_uppercase(),
        SerdeRenameRuleIr::KebabCase => words.join("-"),
        SerdeRenameRuleIr::ScreamingKebabCase => words.join("-").to_ascii_uppercase(),
    }
}

fn split_words(source: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();

    for ch in source.chars() {
        if ch == '_' || ch == '-' {
            if !current.is_empty() {
                words.push(current.to_ascii_lowercase());
                current.clear();
            }
            continue;
        }

        let is_boundary = ch.is_ascii_uppercase() && !current.is_empty();
        if is_boundary {
            words.push(current.to_ascii_lowercase());
            current.clear();
        }
        current.push(ch);
    }

    if !current.is_empty() {
        words.push(current.to_ascii_lowercase());
    }

    words
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
}
