//! Checking that a default value and its type are ones the codec supports.

use super::*;

/// Ensures a typed serde default is compatible with the field type root.
pub(super) fn validate_default_value(
    ty: &TypeIr,
    value: &AnnotationValueIr,
    source: &str,
    class_name: &str,
    field_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if default_value_is_compatible(ty, value) {
        return;
    }

    if matches!(value, AnnotationValueIr::Null) {
        diagnostics.push(Diagnostic::error(format!(
            "field `{field_name}` on class `{class_name}` uses `SerDe(defaultValue: {source})` on non-nullable field"
        )));
        return;
    }

    diagnostics.push(Diagnostic::error(format!(
        "field `{field_name}` on class `{class_name}` uses `SerDe(defaultValue: {source})` that is not compatible with `{}`",
        ty.name().unwrap_or("field type")
    )));
}

/// Returns whether one parser-owned default value root can initialize a type.
pub(super) fn default_value_is_compatible(ty: &TypeIr, value: &AnnotationValueIr) -> bool {
    match value {
        AnnotationValueIr::Null => ty.is_nullable(),
        AnnotationValueIr::Bool(_) => ty.is_builtin(BuiltinType::Bool),
        AnnotationValueIr::String(_) => ty.is_builtin(BuiltinType::String),
        AnnotationValueIr::Number {
            kind: AnnotationNumberKindIr::Int,
            ..
        } => {
            ty.is_builtin(BuiltinType::Int)
                || ty.is_builtin(BuiltinType::Double)
                || ty.is_builtin(BuiltinType::Num)
        }
        AnnotationValueIr::Number {
            kind: AnnotationNumberKindIr::Double,
            ..
        } => ty.is_builtin(BuiltinType::Double) || ty.is_builtin(BuiltinType::Num),
        AnnotationValueIr::List(_) => ty.is_named(DART_LIST),
        AnnotationValueIr::Set(_) => ty.is_named(DART_SET),
        AnnotationValueIr::Map(_) => ty.is_named(DART_MAP),
        AnnotationValueIr::Record(_)
        | AnnotationValueIr::Constructor { .. }
        | AnnotationValueIr::Member(_)
        | AnnotationValueIr::Expression(_) => true,
    }
}

/// Ensures a type can be automatically mapped from JSON.
///
/// We currently support built-ins, specific named types (DateTime, Uri, etc.),
/// collections (List, Set, Map), and other models within the same library.
pub(super) fn validate_type_supported(
    ty: &TypeIr,
    context: &JsonModelContext<'_>,
    class_name: &str,
    field_name: &str,
    direction: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match ty {
        TypeIr::Builtin { .. } | TypeIr::Dynamic => {}
        TypeIr::Unknown => diagnostics.push(Diagnostic::error(format!(
            "`{direction}` does not support unresolved type on `{class_name}.{field_name}`"
        ))),
        TypeIr::Function { .. } => diagnostics.push(Diagnostic::error(format!(
            "`{direction}` does not support function types on `{class_name}.{field_name}`"
        ))),
        TypeIr::Record { .. } => diagnostics.push(Diagnostic::error(format!(
            "`{direction}` does not support record types on `{class_name}.{field_name}`"
        ))),
        TypeIr::Named { name, args, .. }
            if name.as_ref() == DART_LIST || name.as_ref() == DART_SET =>
        {
            if let Some(item) = args.first() {
                validate_type_supported(
                    item,
                    context,
                    class_name,
                    field_name,
                    direction,
                    diagnostics,
                );
            } else {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` requires one type argument for `{name}` on `{class_name}.{field_name}`"
                )));
            }
        }
        TypeIr::Named { name, args, .. } if name.as_ref() == DART_MAP => {
            if args.len() != 2 {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` requires two type arguments for `Map` on `{class_name}.{field_name}`"
                )));
                return;
            }
            if !args[0].is_builtin(BuiltinType::String) {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` only supports `Map<String, T>` on `{class_name}.{field_name}`"
                )));
            }
            validate_type_supported(
                &args[1],
                context,
                class_name,
                field_name,
                direction,
                diagnostics,
            );
        }
        TypeIr::Named { name, args, .. } => {
            if !args.is_empty() {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` does not yet support generic named type `{name}` on `{class_name}.{field_name}`"
                )));
            } else if is_supported_named_scalar(name) {
                // Handled by built-in SerDe conversions.
            } else if !has_verified_json_conversion(context, name, direction) {
                diagnostics.push(Diagnostic::error(format!(
                    "`{direction}` requires `{}` or deriving `{direction}`/using `SerDe(codec: ...)` for `{class_name}.{field_name}`",
                    required_json_member(name, direction)
                )));
            }
        }
    }
}

/// Returns true when a class requests JSON serialization.
pub(super) fn wants_serialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "dust_dart::Serialize")
}

/// Returns true when a class requests JSON deserialization.
pub(super) fn wants_deserialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "dust_dart::Deserialize")
}
