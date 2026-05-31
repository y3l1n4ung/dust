use dust_plugin_api::{PluginContribution, PluginRegistry};

use super::support::{FakePlugin, emit_with_registry, generated_output, sample_library};

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

    assert_eq!(
        source,
        generated_output(
            r#"part of 'user.dart';

const Object _undefined = Object();

mixin _$User {
  @override
  String toString() => 'User()';
}

typedef UserMap = Map<String, Object?>;

User _$UserFromJson(Map<String, Object?> json) => User();
"#
        )
    );
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
    let mut contribution = PluginContribution::default();
    contribution.push_mixin_member("User", "@override\nString toString() => 'User()';");
    let mut registry = PluginRegistry::new();
    registry
        .register(Box::new(FakePlugin {
            name: "fake",
            requested: Vec::new(),
            diagnostics: Vec::new(),
            contribution,
        }))
        .unwrap();
    let library = sample_library("lib/user.g.dart".to_owned());

    let first = emit_with_registry(&library, &registry, None);
    let second = emit_with_registry(&library, &registry, Some(&first.source));

    assert!(first.changed);
    assert!(!second.changed);
}
