//! Workspace JSON capability facts for SerDe validation.

use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedClassSurface, ParsedDartFileSurface, ParsedTypeKind,
    ParsedTypeSurface,
};
use dust_plugin_api::WorkspaceAnalysisBuilder;

/// Workspace types declared in Dart source files.
pub(crate) const JSON_TYPES_KEY: &str = "dust_plugin_serde.json_types.v1";
/// Types that can serialize to JSON.
pub(crate) const JSON_SERIALIZABLE_TYPES_KEY: &str = "dust_plugin_serde.json_serializable_types.v1";
/// Types that can deserialize from JSON.
pub(crate) const JSON_DESERIALIZABLE_TYPES_KEY: &str =
    "dust_plugin_serde.json_deserializable_types.v1";

/// Collects workspace-wide JSON capability facts from parser-owned surfaces.
pub(crate) fn collect_workspace_analysis(
    library: &ParsedDartFileSurface,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    for class in &library.classes {
        analysis.add_string_set_value(JSON_TYPES_KEY, class.name.clone());
        if class_has_serialize(class) || has_to_json_method(class) {
            analysis.add_string_set_value(JSON_SERIALIZABLE_TYPES_KEY, class.name.clone());
        }
        if class_has_deserialize(class) || has_from_json_factory(class) {
            analysis.add_string_set_value(JSON_DESERIALIZABLE_TYPES_KEY, class.name.clone());
        }
    }

    for enum_ in &library.enums {
        analysis.add_string_set_value(JSON_TYPES_KEY, enum_.name.clone());
        if annotations_have_trait(&enum_.annotations, "Serialize") {
            analysis.add_string_set_value(JSON_SERIALIZABLE_TYPES_KEY, enum_.name.clone());
        }
        if annotations_have_trait(&enum_.annotations, "Deserialize") {
            analysis.add_string_set_value(JSON_DESERIALIZABLE_TYPES_KEY, enum_.name.clone());
        }
    }
}

/// Returns whether a class requests generated serialization.
fn class_has_serialize(class: &ParsedClassSurface) -> bool {
    annotations_have_trait(&class.annotations, "Serialize")
}

/// Returns whether a class requests generated deserialization.
fn class_has_deserialize(class: &ParsedClassSurface) -> bool {
    annotations_have_trait(&class.annotations, "Deserialize")
}

/// Returns whether annotations include a direct trait or `@Derive([...])` entry.
fn annotations_have_trait(annotations: &[ParsedAnnotation], trait_name: &str) -> bool {
    annotations.iter().any(|annotation| {
        annotation.is_named(trait_name)
            || (annotation.is_named("Derive")
                && annotation
                    .positional_constructor_names()
                    .iter()
                    .any(|name| name == trait_name))
    })
}

/// Returns whether a parsed class has a JSON object `toJson()` method.
fn has_to_json_method(class: &ParsedClassSurface) -> bool {
    class.methods.iter().any(|method| {
        method.name == "toJson"
            && !method.is_static
            && method.params.is_empty()
            && is_json_map_type(method.parsed_return_type.as_ref())
    })
}

/// Returns whether a parsed class has a JSON object `fromJson` factory.
fn has_from_json_factory(class: &ParsedClassSurface) -> bool {
    class.constructors.iter().any(|constructor| {
        constructor.name.as_deref() == Some("fromJson")
            && constructor.is_factory
            && constructor.params.len() == 1
            && constructor.params[0].kind == ParameterKind::Positional
            && is_json_map_type(constructor.params[0].parsed_type.as_ref())
    })
}

/// Returns whether a parsed type is a `Map<String, ...>` JSON object.
fn is_json_map_type(ty: Option<&ParsedTypeSurface>) -> bool {
    let Some(ty) = ty else {
        return false;
    };
    let ParsedTypeKind::Named { name, args } = &ty.kind else {
        return false;
    };

    name == "Map" && args.len() == 2 && is_string_type(&args[0])
}

/// Returns whether a parsed type is `String`.
fn is_string_type(ty: &ParsedTypeSurface) -> bool {
    matches!(&ty.kind, ParsedTypeKind::Builtin(name) if name == "String")
}
