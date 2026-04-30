use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{members_for_class, sample_library};

#[test]
fn plugin_claims_core_derive_traits() {
    let plugin = register_plugin();
    let claimed = plugin.claimed_traits();

    let names = claimed
        .iter()
        .map(|symbol| symbol.0.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        names,
        vec![
            "derive_annotation::ToString",
            "derive_annotation::Debug",
            "derive_annotation::Eq",
            "derive_annotation::CopyWith",
        ]
    );
}

#[test]
fn eq_requires_no_companion_trait() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&sample_library(&["derive_annotation::Eq"]));

    assert!(diagnostics.is_empty());
}

#[test]
fn requests_undefined_only_for_copywith_when_needed() {
    let plugin = register_plugin();

    let copywith_requested =
        plugin.requested_symbols(&sample_library(&["derive_annotation::CopyWith"]));
    let no_requested = plugin.requested_symbols(&sample_library(&["derive_annotation::ToString"]));

    assert_eq!(copywith_requested, vec!["_undefined".to_owned()]);
    assert!(no_requested.is_empty());
}

#[test]
fn emits_full_fragments_for_matching_traits() {
    let plugin = register_plugin();
    let library = sample_library(&[
        "derive_annotation::ToString",
        "derive_annotation::Eq",
        "derive_annotation::CopyWith",
    ]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let members = members_for_class(&contribution, "User");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members.len(), 4);
    assert!(members.iter().any(|fragment| {
        fragment.contains("String toString() {")
            && fragment.contains("return 'User('")
            && fragment.contains("'id: ${_dustSelf.id}, '")
            && fragment.contains("'age: ${_dustSelf.age}'")
    }));
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("bool operator ==(Object other) =>"))
    );
    assert!(members.iter().any(|fragment| {
        fragment.contains("int get hashCode => Object.hashAll([")
            && fragment.contains("runtimeType,")
            && fragment.contains("_dustSelf.id,")
            && fragment.contains("_dustSelf.age,")
    }));
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("User copyWith({"))
    );
    assert!(members.iter().any(|fragment| {
        fragment.contains("String? id,")
            && fragment.contains("Object? age = _undefined,")
            && !fragment.contains("final nextIdSource = id ?? _dustSelf.id;")
            && !fragment.contains(
                "final nextAgeSource = identical(age, _undefined) ? _dustSelf.age : age as int?;",
            )
            && fragment.contains("id ?? _dustSelf.id,")
            && fragment.contains("identical(age, _undefined) ? _dustSelf.age : age as int?,")
    }));
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("return User("))
    );
}

#[test]
fn legacy_debug_symbol_still_emits_tostring() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &sample_library(&["derive_annotation::Debug"]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "User");

    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("String toString() {"))
    );
}

#[test]
fn emits_eq_and_hash_fragments_when_eq_is_present() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &sample_library(&["derive_annotation::Eq"]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "User");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members.len(), 2);
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("bool operator ==(Object other) =>"))
    );
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("int get hashCode => Object.hashAll(["))
    );
}
