//! Validation for SerDe generation inputs.

use dust_dart_emit::{DART_LIST, DART_MAP, DART_SET};
use dust_diagnostics::Diagnostic;
use dust_ir::{
    AnnotationNumberKindIr, AnnotationValueIr, BuiltinType, ClassIr, ClassKindIr, DartFileIr,
    TypeIr,
};
use dust_plugin_api::WorkspaceAnalysis;

/// Local JSON capability facts used by field-type validation.
mod json_capability;

use json_capability::{
    JsonModelContext, has_verified_json_conversion, is_supported_named_scalar, required_json_member,
};

/// Validates that a library and its models are compatible with SerDe generation.
///
/// This function performs static analysis on the IR to catch potential runtime
/// errors early, such as unsupported field types for deserialization.
pub(crate) fn validate_library(library: &DartFileIr) -> Vec<Diagnostic> {
    validate_library_inner(library, JsonModelContext::new(library))
}

/// Validates a library with workspace-wide JSON conversion facts.
pub(crate) fn validate_library_with_workspace(
    library: &DartFileIr,
    workspace: &WorkspaceAnalysis,
) -> Vec<Diagnostic> {
    validate_library_inner(
        library,
        JsonModelContext::with_workspace(library, Some(workspace)),
    )
}

/// Validates a library using the provided JSON capability context.
fn validate_library_inner(library: &DartFileIr, context: JsonModelContext<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for class in &library.classes {
        let serialize = wants_serialize(class);
        let deserialize = wants_deserialize(class);
        if !serialize && !deserialize {
            continue;
        }

        // Dust SerDe generation doesn't yet support mixin classes.
        if matches!(class.kind, ClassKindIr::MixinClass) {
            diagnostics.push(Diagnostic::error(format!(
                "Dust serde generation does not support `mixin class` targets like `{}`",
                class.name
            )));
            continue;
        }

        // Deserialize requires the class to be instantiable.
        if deserialize && class.is_abstract {
            diagnostics.push(Diagnostic::error(format!(
                "`Deserialize` cannot target abstract class `{}`",
                class.name
            )));
        }

        // Deserialization needs to know how to build the object.
        if deserialize
            && !class
                .constructors
                .iter()
                .any(|ctor| ctor.can_construct_all_fields(&class.fields))
        {
            diagnostics.push(Diagnostic::error(format!(
                "`Deserialize` requires a constructor that can initialize every field on class `{}`",
                class.name
            )));
        }

        // Custom renames are only supported at the field level.
        if class
            .serde
            .as_ref()
            .and_then(|serde| serde.rename.as_ref())
            .is_some()
        {
            diagnostics.push(Diagnostic::error(format!(
                "class `{}` does not support `SerDe(rename: ...)` in Dust serde generation",
                class.name
            )));
        }

        for field in &class.fields {
            if let Some(serde) = &field.serde {
                // Ensure default values are provided for skipped fields.
                if serde.skip_deserializing && serde.default_value_source.is_none() {
                    diagnostics.push(Diagnostic::error(format!(
                        "field `{}` on class `{}` uses `skipDeserializing` without a `defaultValue`",
                        field.name, class.name
                    )));
                }
                if let Some(default_value) = &serde.default_value {
                    validate_default_value(
                        &field.ty,
                        default_value,
                        serde.default_value_source.as_deref().unwrap_or("..."),
                        &class.name,
                        &field.name,
                        &mut diagnostics,
                    );
                }
            }

            let uses_codec = field
                .serde
                .as_ref()
                .and_then(|serde| serde.codec_source.as_ref())
                .is_some();

            // Validate type mapping for non-codec fields.
            if serialize && !uses_codec {
                validate_type_supported(
                    &field.ty,
                    &context,
                    &class.name,
                    &field.name,
                    "Serialize",
                    &mut diagnostics,
                );
            }
            if deserialize && !uses_codec {
                validate_type_supported(
                    &field.ty,
                    &context,
                    &class.name,
                    &field.name,
                    "Deserialize",
                    &mut diagnostics,
                );
            }
        }
    }

    diagnostics
}

/// Ensures a typed serde default is compatible with the field type root.
fn validate_default_value(
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
fn default_value_is_compatible(ty: &TypeIr, value: &AnnotationValueIr) -> bool {
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
fn validate_type_supported(
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
