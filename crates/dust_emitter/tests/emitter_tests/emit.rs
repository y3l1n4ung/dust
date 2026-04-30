use dust_plugin_api::{PluginContribution, PluginRegistry};

use super::support::{FakePlugin, emit_with_registry, sample_library};

#[test]
fn emitter_writes_sections_in_fixed_order_and_wraps_mixins() {
    let mut contribution = PluginContribution::default();
    contribution.push_mixin_member("User", "@override\nString toString() => 'User()';");
    contribution
        .support_types
        .push("typedef UserMap = Map<String, Object?>;".to_owned());
    contribution
        .top_level_functions
        .push("User _$UserFromJson(Map<String, Object?> json) => User();".to_owned());

    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "fake",
            requested: vec!["_undefined"],
            diagnostics: Vec::new(),
            contribution,
        }))
        .unwrap();

    let result = emit_with_registry(
        &sample_library("lib/user.g.dart".to_owned()),
        &registry,
        None,
    );
    let source = result.source;

    let header = source.find("// GENERATED CODE").unwrap();
    let part_of = source.find("part of 'user.dart';").unwrap();
    let helper = source.find("const Object _undefined = Object();").unwrap();
    let mixin = source.find("mixin _$UserDust {").unwrap();
    let support = source.find("typedef UserMap").unwrap();
    let function = source.find("_$UserFromJson").unwrap();

    assert!(
        header < part_of
            && part_of < helper
            && helper < mixin
            && mixin < support
            && support < function
    );
    assert!(source.contains("// coverage:ignore-file"));
    assert!(source.contains("// ignore_for_file: type=lint"));
    assert!(source.contains(
        "// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark"
    ));
    assert!(source.contains("mixin _$UserDust {"));
    assert!(source.contains("  User get _dustSelf => this as User;"));
    assert!(source.contains("  @override\n  String toString() => 'User()';"));
}

#[test]
fn emitter_dedupes_helpers_and_symbol_reservations() {
    let mut registry = PluginRegistry::new();

    for name in ["a", "b"] {
        let mut contribution = PluginContribution::default();
        contribution
            .shared_helpers
            .push("typedef UserMap = Map<String, Object?>;".to_owned());
        registry
            .register(Box::new(FakePlugin {
                name,
                requested: vec!["_undefined"],
                diagnostics: Vec::new(),
                contribution,
            }))
            .unwrap();
    }

    let result = emit_with_registry(
        &sample_library("lib/user.g.dart".to_owned()),
        &registry,
        None,
    );
    assert_eq!(result.symbols.reserved().len(), 1);
    assert_eq!(
        result
            .source
            .matches("const Object _undefined = Object();")
            .count(),
        1
    );
    assert_eq!(
        result
            .source
            .matches("typedef UserMap = Map<String, Object?>;")
            .count(),
        1
    );
}

#[test]
fn emitter_sets_changed_false_when_output_matches_previous() {
    let registry = PluginRegistry::new();
    let library = sample_library("lib/user.g.dart".to_owned());

    let first = emit_with_registry(&library, &registry, None);
    let second = emit_with_registry(&library, &registry, Some(&first.source));

    assert!(first.changed);
    assert!(!second.changed);
}
