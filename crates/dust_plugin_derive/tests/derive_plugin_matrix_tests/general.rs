use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{class, constructor, constructor_param, field, library, members_for_class};
use dust_ir::{ParamKind, TypeIr};

#[test]
fn emits_debug_eq_and_hash_for_zero_field_class() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Unit",
            Vec::new(),
            vec![constructor(None, Vec::new())],
            &["derive_annotation::ToString", "derive_annotation::Eq"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Unit");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members.len(), 3);
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("String toString() {\n  return 'Unit()';\n}"))
    );
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("other is Unit &&"))
    );
    assert!(members.iter().any(|fragment| {
        fragment.contains("int get hashCode => Object.hashAll([\n  runtimeType,\n]);")
    }));
}

#[test]
fn validation_accumulates_multiple_class_errors() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library(vec![class(
        "BrokenCopyWith",
        vec![field("id", TypeIr::string()), field("age", TypeIr::int())],
        vec![constructor(
            None,
            vec![constructor_param(
                "id",
                TypeIr::string(),
                ParamKind::Positional,
            )],
        )],
        &["derive_annotation::CopyWith"],
    )]));

    assert_eq!(diagnostics.len(), 1);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "`CopyWith` requires a constructor that accepts every field on class `BrokenCopyWith`",
        )
    }));
}

#[test]
fn requested_symbols_are_deduped_for_multiple_copywith_classes() {
    let plugin = register_plugin();
    let requested = plugin.requested_symbols(&library(vec![
        class(
            "User",
            vec![field("id", TypeIr::string())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "id",
                    TypeIr::string(),
                    ParamKind::Positional,
                )],
            )],
            &["derive_annotation::CopyWith"],
        ),
        class(
            "Team",
            vec![field("name", TypeIr::string())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "name",
                    TypeIr::string(),
                    ParamKind::Positional,
                )],
            )],
            &["derive_annotation::CopyWith"],
        ),
    ]));

    assert!(requested.is_empty());
}

#[test]
fn emits_fragments_for_multiple_classes_in_stable_feature_order() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![
            class(
                "User",
                vec![field("id", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "id",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["derive_annotation::ToString", "derive_annotation::Eq"],
            ),
            class(
                "Team",
                vec![field("name", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "name",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["derive_annotation::CopyWith"],
            ),
        ]),
        &SymbolPlan::default(),
    );
    let user_members = members_for_class(&contribution, "User");
    let team_members = members_for_class(&contribution, "Team");

    assert_eq!(contribution.mixin_members.len(), 2);
    assert_eq!(user_members.len(), 3);
    assert_eq!(team_members.len(), 1);
    assert!(user_members[0].contains("String toString() {"));
    assert!(user_members[0].contains("return 'User('"));
    assert!(user_members[0].contains("'id: ${_dustSelf.id}'"));
    assert!(user_members[1].contains("bool operator ==(Object other) =>"));
    assert!(user_members[2].contains("int get hashCode => Object.hashAll(["));
    assert!(team_members[0].contains("Team copyWith({"));
    assert!(team_members[0].contains("String? name,"));
    assert!(team_members[0].contains("name ?? _dustSelf.name,"));
}
