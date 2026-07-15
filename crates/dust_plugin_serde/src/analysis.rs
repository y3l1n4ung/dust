//! Workspace JSON capability facts for SerDe validation.

use dust_ir::{BuiltinType, ClassIr, DartFileIr, EnumIr, ParamKind, TypeIr};
use dust_plugin_api::WorkspaceAnalysisBuilder;

/// Workspace types declared in Dart source files.
pub(crate) const JSON_TYPES_KEY: &str = "dust_plugin_serde.json_types.v1";
/// Types that can serialize to JSON.
pub(crate) const JSON_SERIALIZABLE_TYPES_KEY: &str = "dust_plugin_serde.json_serializable_types.v1";
/// Types that can deserialize from JSON.
pub(crate) const JSON_DESERIALIZABLE_TYPES_KEY: &str =
    "dust_plugin_serde.json_deserializable_types.v1";

/// Collects workspace-wide JSON capability facts from canonical IR.
pub(crate) fn collect_workspace_analysis_ir(
    library: &DartFileIr,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    for class in &library.classes {
        analysis.add_string_set_value(JSON_TYPES_KEY, class.name.clone());
        if class_has_trait(class, "dust_dart::Serialize") || has_to_json_method_ir(class) {
            analysis.add_string_set_value(JSON_SERIALIZABLE_TYPES_KEY, class.name.clone());
        }
        if class_has_trait(class, "dust_dart::Deserialize") || has_from_json_factory_ir(class) {
            analysis.add_string_set_value(JSON_DESERIALIZABLE_TYPES_KEY, class.name.clone());
        }
    }
    for enum_ in &library.enums {
        analysis.add_string_set_value(JSON_TYPES_KEY, enum_.name.clone());
        if enum_has_trait(enum_, "dust_dart::Serialize") {
            analysis.add_string_set_value(JSON_SERIALIZABLE_TYPES_KEY, enum_.name.clone());
        }
        if enum_has_trait(enum_, "dust_dart::Deserialize") {
            analysis.add_string_set_value(JSON_DESERIALIZABLE_TYPES_KEY, enum_.name.clone());
        }
    }
}

/// Returns whether a class declares the requested resolved trait symbol.
fn class_has_trait(class: &ClassIr, symbol: &str) -> bool {
    class.traits.iter().any(|trait_| trait_.symbol.0 == symbol)
}

/// Returns whether an enum declares the requested resolved trait symbol.
fn enum_has_trait(enum_: &EnumIr, symbol: &str) -> bool {
    enum_.traits.iter().any(|trait_| trait_.symbol.0 == symbol)
}

/// Returns whether a class exposes a JSON object `toJson()` method in IR.
fn has_to_json_method_ir(class: &ClassIr) -> bool {
    class.methods.iter().any(|method| {
        method.name == "toJson"
            && !method.is_static
            && method.params.is_empty()
            && is_json_map_type_ir(&method.return_type)
    })
}

/// Returns whether a class exposes a JSON object `fromJson` factory in IR.
fn has_from_json_factory_ir(class: &ClassIr) -> bool {
    class.constructors.iter().any(|constructor| {
        constructor.name.as_deref() == Some("fromJson")
            && constructor.is_factory
            && constructor.params.len() == 1
            && constructor.params[0].kind == ParamKind::Positional
            && is_json_map_type_ir(&constructor.params[0].ty)
    })
}

/// Returns whether an IR type is a `Map<String, ...>` JSON object.
fn is_json_map_type_ir(ty: &TypeIr) -> bool {
    ty.is_named("Map") && ty.args().len() == 2 && ty.args()[0].is_builtin(BuiltinType::String)
}
