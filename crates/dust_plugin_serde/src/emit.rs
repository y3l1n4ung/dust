use std::collections::HashMap;

use dust_ir::{ClassIr, DartFileIr, EnumIr};
use dust_plugin_api::PluginContribution;

use crate::{
    emit_class::{emit_from_json_helper, emit_to_json_helper, emit_to_json_mixin},
    emit_enum::{emit_enum_from_json_helper, emit_enum_to_json_helper},
    emit_sealed::{
        emit_sealed_from_json_helper, emit_sealed_to_json_helper, is_tagged_sealed_class,
    },
    emit_variant_class::{emit_generated_variant_class, generated_variant_classes},
};

/// Orchestrates the emission of all SerDe-related code for a library.
///
/// This function identifies which models (classes and enums) have requested
/// serialization or deserialization and generates the corresponding Dart code.
pub(crate) fn emit_library(library: &DartFileIr) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let generated_variants = generated_variant_classes(library);
    let mut serializable_classes = library
        .classes
        .iter()
        .filter(|class| wants_serialize(class))
        .map(|class| class.name.as_str())
        .collect::<Vec<_>>();
    let serializable_enums = library
        .enums
        .iter()
        .filter(|e| wants_serialize_enum(e))
        .map(|e| e.name.as_str())
        .collect::<Vec<_>>();

    serializable_classes.extend(
        generated_variants
            .iter()
            .filter(|variant| variant.serializable)
            .map(|variant| variant.class.name.as_str()),
    );

    let mut deserializable_classes = library
        .classes
        .iter()
        .filter(|class| wants_deserialize(class))
        .map(|class| class.name.as_str())
        .collect::<Vec<_>>();
    let deserializable_enums = library
        .enums
        .iter()
        .filter(|e| wants_deserialize_enum(e))
        .map(|e| e.name.as_str())
        .collect::<Vec<_>>();
    deserializable_classes.extend(
        generated_variants
            .iter()
            .filter(|variant| variant.deserializable)
            .map(|variant| variant.class.name.as_str()),
    );
    let sealed_base_by_variant = sealed_base_by_variant(library);

    // Generate class-specific code.
    for class in &library.classes {
        if wants_serialize(class) {
            let helper_class_name = sealed_base_by_variant
                .get(class.name.as_str())
                .copied()
                .unwrap_or(class.name.as_str());
            contribution.push_mixin_member(&class.name, emit_to_json_mixin(helper_class_name));
            if is_tagged_sealed_class(class) {
                if let Some(helper) = emit_sealed_to_json_helper(class, &serializable_classes) {
                    contribution.top_level_functions.push(helper);
                }
            } else {
                contribution.top_level_functions.push(emit_to_json_helper(
                    class,
                    &serializable_classes,
                    &serializable_enums,
                ));
            }
        }
        if wants_deserialize(class) {
            if is_tagged_sealed_class(class) {
                if let Some(helper) = emit_sealed_from_json_helper(class, &deserializable_classes) {
                    contribution.top_level_functions.push(helper);
                }
            } else if let Some(helper) =
                emit_from_json_helper(class, &deserializable_classes, &deserializable_enums)
            {
                contribution.top_level_functions.push(helper);
            }
        }
    }

    let generated_variant_support = generated_variants
        .iter()
        .map(emit_generated_variant_class)
        .collect::<Vec<_>>()
        .join("\n\n");
    if !generated_variant_support.is_empty() {
        contribution.support_types.push(generated_variant_support);
    }

    for variant in &generated_variants {
        if variant.serializable {
            contribution.top_level_functions.push(emit_to_json_helper(
                &variant.class,
                &serializable_classes,
                &serializable_enums,
            ));
        }
        if variant.deserializable {
            if let Some(helper) = emit_from_json_helper(
                &variant.class,
                &deserializable_classes,
                &deserializable_enums,
            ) {
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

/// Maps sealed variant target class names back to their sealed base class.
fn sealed_base_by_variant(library: &DartFileIr) -> HashMap<&str, &str> {
    let mut base_by_variant = HashMap::new();
    for class in &library.classes {
        let Some(serde) = &class.serde else {
            continue;
        };
        for variant in &serde.variants {
            base_by_variant.insert(variant.target_class_name.as_str(), class.name.as_str());
        }
    }
    base_by_variant
}

/// Returns true when a class requests JSON serialization.
fn wants_serialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "dust_dart::Serialize")
}

/// Returns true when a class requests JSON deserialization.
fn wants_deserialize(class: &ClassIr) -> bool {
    class
        .traits
        .iter()
        .any(|item| item.symbol.0 == "dust_dart::Deserialize")
}

/// Returns true when an enum requests JSON serialization.
fn wants_serialize_enum(e: &EnumIr) -> bool {
    e.traits
        .iter()
        .any(|item| item.symbol.0 == "dust_dart::Serialize")
}

/// Returns true when an enum requests JSON deserialization.
fn wants_deserialize_enum(e: &EnumIr) -> bool {
    e.traits
        .iter()
        .any(|item| item.symbol.0 == "dust_dart::Deserialize")
}
