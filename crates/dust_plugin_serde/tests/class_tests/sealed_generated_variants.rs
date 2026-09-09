use dust_ir::{
    ClassIr, ClassKindIr, ConstructorParamIr, ParamKind, SerdeClassConfigIr, SerdeVariantConfigIr,
    TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;

use super::support::{class, constructor, constructor_param, field, function_for, library, span};

/// Class fixtures the sealed-variant tests are built from.
#[path = "sealed_generated_variants/fixtures.rs"]
mod fixtures;
use self::fixtures::*;

#[test]
fn generates_missing_concrete_variant_classes_from_factories() {
    let plugin = register_plugin();
    let library = library(vec![payment_event_base()], vec![]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(support_types_contain(
        &contribution.support_types,
        &payment_event_variant_support()
    ));
    assert!(
        support_type_for(&contribution.support_types, "$JsonPaymentEventSerializer")
            .contains("implements Serializer<JsonPaymentEvent, Map<String, Object?>>")
    );
    assert!(
        support_type_for(
            &contribution.support_types,
            "$JsonPaymentSuccessDeserializer"
        )
        .contains("implements Deserializer<JsonPaymentSuccess, Map<String, Object?>>")
    );
    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentEventSerialize",
        ),
        payment_event_to_json()
    );
    assert_eq!(
        function_for(
            &contribution.top_level_functions,
            "_$JsonPaymentSuccessSerialize",
        ),
        success_to_json()
    );
}

#[test]
fn does_not_generate_source_defined_variant_classes() {
    let plugin = register_plugin();
    let library = library(
        vec![payment_event_base(), success_variant(), failed_variant()],
        vec![],
    );

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(!support_types_contain(
        &contribution.support_types,
        &payment_event_variant_support()
    ));
    assert!(
        support_type_for(&contribution.support_types, "$JsonPaymentSuccessSerializer")
            .contains("implements Serializer<JsonPaymentSuccess, Map<String, Object?>>")
    );
}

#[test]
fn renders_empty_positional_nullable_and_defaulted_variant_params() {
    let plugin = register_plugin();
    let library = library(vec![shape_event_base()], vec![]);

    let contribution = plugin
        .generate(
            &library,
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert!(support_types_contain(
        &contribution.support_types,
        &shape_variant_support()
    ));
    assert!(
        support_type_for(&contribution.support_types, "$EmptyVariantSerializer")
            .contains("implements Serializer<EmptyVariant, Map<String, Object?>>")
    );
}

fn support_types_contain(support_types: &[String], expected: &str) -> bool {
    support_types.iter().any(|support| support == expected)
}

fn support_type_for<'a>(support_types: &'a [String], name: &str) -> &'a str {
    support_types
        .iter()
        .find(|support| support.contains(&format!("final class {name} ")))
        .map(String::as_str)
        .unwrap_or("")
}
