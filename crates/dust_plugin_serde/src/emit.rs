use std::collections::HashSet;

use dust_ir::{ClassIr, EnumIr, LibraryIr};
use dust_plugin_api::PluginContribution;

use crate::{
    emit_class::{emit_from_json_helper, emit_to_json_helper, emit_to_json_mixin},
    emit_enum::{emit_enum_from_json_helper, emit_enum_to_json_helper},
    emit_support::render_deserialize_helpers,
};

/// Orchestrates the emission of all SerDe-related code for a library.
///
/// This function identifies which models (classes and enums) have requested
/// serialization or deserialization and generates the corresponding Dart code.
pub(crate) fn emit_library(library: &LibraryIr) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let serializable_classes = library
        .classes
        .iter()
        .filter(|class| wants_serialize(class))
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();
    let serializable_enums = library
        .enums
        .iter()
        .filter(|e| wants_serialize_enum(e))
        .map(|e| e.name.clone())
        .collect::<HashSet<_>>();

    let deserializable_classes = library
        .classes
        .iter()
        .filter(|class| wants_deserialize(class))
        .map(|class| class.name.clone())
        .collect::<HashSet<_>>();
    let deserializable_enums = library
        .enums
        .iter()
        .filter(|e| wants_deserialize_enum(e))
        .map(|e| e.name.clone())
        .collect::<HashSet<_>>();

    // If any model needs deserialization, include the standard shared JSON helpers.
    if !deserializable_classes.is_empty() || !deserializable_enums.is_empty() {
        contribution
            .shared_helpers
            .push(render_deserialize_helpers().to_owned());
    }

    // Generate class-specific code.
    for class in &library.classes {
        if wants_serialize(class) {
            contribution.push_mixin_member(&class.name, emit_to_json_mixin(class));
            contribution.top_level_functions.push(emit_to_json_helper(
                class,
                &serializable_classes,
                &serializable_enums,
            ));
        }
        if wants_deserialize(class) {
            if let Some(helper) =
                emit_from_json_helper(class, &deserializable_classes, &deserializable_enums)
            {
                contribution.top_level_functions.push(helper);
            }
        }
    }

    // Generate enum-specific code.
    for e in &library.enums {
        if wants_serialize_enum(e) {
            contribution
                .top_level_functions
                .push(emit_enum_to_json_helper(e));
        }
        if wants_deserialize_enum(e) {
            contribution
                .top_level_functions
                .push(emit_enum_from_json_helper(e));
        }
    }
    contribution
}

fn wants_serialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Serialize")
}

fn wants_deserialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Deserialize")
}

fn wants_serialize_enum(e: &EnumIr) -> bool {
    e.traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Serialize")
}

fn wants_deserialize_enum(e: &EnumIr) -> bool {
    e.traits
        .iter()
        .any(|item| item.symbol.0 == "derive_serde_annotation::Deserialize")
}
