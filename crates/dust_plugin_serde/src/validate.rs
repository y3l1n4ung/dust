//! Validation for SerDe generation inputs.

use std::collections::BTreeMap;

use dust_dart_emit::{DART_LIST, DART_MAP, DART_SET};
use dust_diagnostics::Diagnostic;
use dust_ir::{
    AnnotationNumberKindIr, AnnotationValueIr, BuiltinType, ClassIr, ClassKindIr, DartFileIr,
    EnumIr, TypeIr,
};
use dust_plugin_api::WorkspaceAnalysis;

/// Local JSON capability facts used by field-type validation.
mod json_capability;

use json_capability::{
    JsonModelContext, has_verified_json_conversion, is_supported_named_scalar, required_json_member,
};

/// Checking that a default value and its type are ones the codec supports.
mod values;
use self::values::*;

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

    for enum_ in &library.enums {
        validate_enum_wire_names(enum_, &mut diagnostics);
    }

    diagnostics
}

/// Ensures generated enum helpers cannot produce ambiguous wire values.
fn validate_enum_wire_names(enum_: &EnumIr, diagnostics: &mut Vec<Diagnostic>) {
    if !enum_wants_serde(enum_) {
        return;
    }

    let mut seen = BTreeMap::new();
    for variant in &enum_.variants {
        let Some(wire_name) = crate::emit_enum::variant_wire_name(enum_, variant) else {
            continue;
        };
        if let Some(previous) = seen.insert(wire_name.clone(), variant.name.as_str()) {
            diagnostics.push(Diagnostic::error(format!(
                "enum `{}` maps variants `{previous}` and `{}` to duplicate SerDe value `{wire_name}`",
                enum_.name, variant.name
            )));
        }
    }
}

/// Returns true when an enum requests JSON serialization or deserialization.
fn enum_wants_serde(enum_: &EnumIr) -> bool {
    enum_.traits.iter().any(|item| {
        item.symbol.0 == "dust_dart::Serialize" || item.symbol.0 == "dust_dart::Deserialize"
    })
}
