use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind, SpanIr,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

fn class_with_traits(name: &str, traits: &[&str]) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(10, 80),
        fields: vec![
            FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::named("String"),
                span: span(20, 30),
                has_default: false,
                serde: None,
            },
            FieldIr {
                name: "age".to_owned(),
                ty: TypeIr::named("int").nullable(),
                span: span(31, 40),
                has_default: false,
                serde: None,
            },
        ],
        constructors: vec![ConstructorIr {
            name: None,
            span: span(40, 60),
            params: vec![
                ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::named("String"),
                    span: span(45, 47),
                    kind: ParamKind::Positional,
                    has_default: false,
                },
                ConstructorParamIr {
                    name: "age".to_owned(),
                    ty: TypeIr::named("int").nullable(),
                    span: span(49, 52),
                    kind: ParamKind::Positional,
                    has_default: false,
                },
            ],
        }],
        traits: traits
            .iter()
            .map(|symbol| TraitApplicationIr {
                symbol: SymbolId::new(*symbol),
                span: span(5, 9),
            })
            .collect(),
        serde: None,
    }
}

fn sample_library(traits: &[&str]) -> LibraryIr {
    LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![class_with_traits("User", traits)],
    }
}

fn members_for_class<'a>(
    contribution: &'a dust_plugin_api::PluginContribution,
    class_name: &str,
) -> &'a [String] {
    contribution
        .mixin_members
        .iter()
        .find(|entry| entry.class_name == class_name)
        .map(|entry| entry.members.as_slice())
        .unwrap_or(&[])
}

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
            "derive_annotation::Debug",
            "derive_annotation::PartialEq",
            "derive_annotation::Eq",
            "derive_annotation::Hash",
            "derive_annotation::Clone",
            "derive_annotation::CopyWith",
        ]
    );
}

#[test]
fn hash_requires_eq() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&sample_library(&["derive_annotation::Hash"]));

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("`Hash` requires `Eq` or `PartialEq` on class `User`")
    );
}

#[test]
fn hash_accepts_partial_eq() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&sample_library(&[
        "derive_annotation::Hash",
        "derive_annotation::PartialEq",
    ]));

    assert!(diagnostics.is_empty());
}

#[test]
fn requests_undefined_when_clone_or_copywith_is_present() {
    let plugin = register_plugin();

    let clone_requested = plugin.requested_symbols(&sample_library(&["derive_annotation::Clone"]));
    let copywith_requested =
        plugin.requested_symbols(&sample_library(&["derive_annotation::CopyWith"]));
    let no_requested = plugin.requested_symbols(&sample_library(&["derive_annotation::Debug"]));

    assert!(clone_requested.is_empty());
    assert_eq!(copywith_requested, vec!["_undefined".to_owned()]);
    assert!(no_requested.is_empty());
}

#[test]
fn emits_full_fragments_for_matching_traits() {
    let plugin = register_plugin();
    let library = sample_library(&[
        "derive_annotation::Debug",
        "derive_annotation::Eq",
        "derive_annotation::Hash",
        "derive_annotation::Clone",
        "derive_annotation::CopyWith",
    ]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let members = members_for_class(&contribution, "User");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members.len(), 5);
    assert!(members.iter().any(|fragment| {
        fragment
            .contains("String toString() => 'User(id: ${_dustSelf.id}, age: ${_dustSelf.age})';")
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
    assert!(members.iter().any(|fragment| {
        fragment.contains("User clone() {")
            && fragment.contains("final clonedId = _dustSelf.id;")
            && fragment.contains("return User(")
    }));
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("User copyWith({"))
    );
    assert!(members.iter().any(|fragment| {
        fragment.contains("String? id,")
            && fragment.contains("Object? age = _undefined,")
            && fragment.contains("final nextIdSource = id ?? _dustSelf.id;")
            && fragment.contains(
                "final nextAgeSource = identical(age, _undefined) ? _dustSelf.age : age as int?;",
            )
    }));
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("return User("))
    );
}

#[test]
fn emits_only_eq_fragment_when_only_eq_is_present() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &sample_library(&["derive_annotation::Eq"]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "User");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(members.len(), 1);
    assert!(
        members
            .iter()
            .any(|fragment| fragment.contains("bool operator ==(Object other) =>"))
    );
}

#[test]
fn clone_copywith_requires_reconstructible_constructor() {
    let plugin = register_plugin();
    let broken = LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(10, 80),
            fields: vec![
                FieldIr {
                    name: "id".to_owned(),
                    ty: TypeIr::named("String"),
                    span: span(20, 30),
                    has_default: false,
                    serde: None,
                },
                FieldIr {
                    name: "age".to_owned(),
                    ty: TypeIr::named("int").nullable(),
                    span: span(31, 40),
                    has_default: false,
                    serde: None,
                },
            ],
            constructors: vec![ConstructorIr {
                name: None,
                span: span(40, 50),
                params: vec![ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::named("String"),
                    span: span(42, 44),
                    kind: ParamKind::Positional,
                    has_default: false,
                }],
            }],
            traits: vec![TraitApplicationIr {
                symbol: SymbolId::new("derive_annotation::CopyWith"),
                span: span(5, 9),
            }],
            serde: None,
        }],
    };

    let diagnostics = plugin.validate(&broken);

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("`Clone`/`CopyWith` requires a constructor that accepts every field")
    );
}

#[test]
fn clone_copywith_rejects_abstract_classes() {
    let plugin = register_plugin();
    let abstract_library = LibraryIr {
        source_path: "lib/entity.dart".to_owned(),
        output_path: "lib/entity.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "Entity".to_owned(),
            is_abstract: true,
            superclass_name: None,
            span: span(10, 80),
            fields: vec![FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::string(),
                span: span(20, 30),
                has_default: false,
                serde: None,
            }],
            constructors: vec![ConstructorIr {
                name: None,
                span: span(40, 50),
                params: vec![ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::string(),
                    span: span(42, 44),
                    kind: ParamKind::Positional,
                    has_default: false,
                }],
            }],
            traits: vec![TraitApplicationIr {
                symbol: SymbolId::new("derive_annotation::Clone"),
                span: span(5, 9),
            }],
            serde: None,
        }],
    };

    let diagnostics = plugin.validate(&abstract_library);

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("cannot target abstract class `Entity`")
    );
}

#[test]
fn rejects_mixin_class_targets() {
    let plugin = register_plugin();
    let mixin_class_library = LibraryIr {
        source_path: "lib/mixin_target.dart".to_owned(),
        output_path: "lib/mixin_target.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::MixinClass,
            name: "MixinTarget".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(10, 80),
            fields: vec![FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::string(),
                span: span(20, 30),
                has_default: false,
                serde: None,
            }],
            constructors: vec![ConstructorIr {
                name: None,
                span: span(40, 50),
                params: vec![ConstructorParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::string(),
                    span: span(42, 44),
                    kind: ParamKind::Positional,
                    has_default: false,
                }],
            }],
            traits: vec![TraitApplicationIr {
                symbol: SymbolId::new("derive_annotation::Debug"),
                span: span(5, 9),
            }],
            serde: None,
        }],
    };

    let diagnostics = plugin.validate(&mixin_class_library);

    assert_eq!(diagnostics.len(), 1);
    assert!(
        diagnostics[0]
            .message
            .contains("does not support `mixin class` targets like `MixinTarget`")
    );
}

#[test]
fn clone_copies_collection_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                superclass_name: None,
                span: span(10, 80),
                fields: vec![
                    FieldIr {
                        name: "groups".to_owned(),
                        ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                        span: span(18, 19),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "items".to_owned(),
                        ty: TypeIr::list_of(TypeIr::string()),
                        span: span(20, 30),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "tags".to_owned(),
                        ty: TypeIr::generic("Set", vec![TypeIr::string()]).nullable(),
                        span: span(31, 40),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "metrics".to_owned(),
                        ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                        span: span(41, 50),
                        has_default: false,
                        serde: None,
                    },
                ],
                constructors: vec![ConstructorIr {
                    name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "items".to_owned(),
                            ty: TypeIr::list_of(TypeIr::string()),
                            span: span(42, 44),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "tags".to_owned(),
                            ty: TypeIr::generic("Set", vec![TypeIr::string()]).nullable(),
                            span: span(45, 47),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Positional,
                            has_default: false,
                        },
                    ],
                }],
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("derive_annotation::Clone"),
                    span: span(5, 9),
                }],
                serde: None,
            }],
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert!(members[0].contains(
        "List<List<String>>.of(_dustSelf.groups.map((item_0) => List<String>.of(item_0)))"
    ));
    assert!(members[0].contains("List<String>.of(_dustSelf.items)"));
    assert!(members[0].contains("_dustSelf.tags == null ? null : Set<String>.of(_dustSelf.tags!)"));
    assert!(members[0].contains("Map<String, List<int>>.fromEntries("));
    assert!(members[0].contains("List<int>.of(entry_"));
    assert!(members[0].contains(".value)"));
}

#[test]
fn copywith_copies_collection_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                superclass_name: None,
                span: span(10, 80),
                fields: vec![
                    FieldIr {
                        name: "groups".to_owned(),
                        ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                        span: span(18, 19),
                        has_default: false,
                        serde: None,
                    },
                    FieldIr {
                        name: "metrics".to_owned(),
                        ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                        span: span(41, 50),
                        has_default: false,
                        serde: None,
                    },
                ],
                constructors: vec![ConstructorIr {
                    name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Named,
                            has_default: false,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Named,
                            has_default: false,
                        },
                    ],
                }],
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("derive_annotation::CopyWith"),
                    span: span(5, 9),
                }],
                serde: None,
            }],
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert!(members[0].contains("Catalog copyWith({"));
    assert!(members[0].contains("List<List<String>>? groups,"));
    assert!(members[0].contains("final nextGroupsSource = groups ?? _dustSelf.groups;"));
    assert!(members[0].contains("final nextGroups = List<List<String>>.of(nextGroupsSource.map((item_0) => List<String>.of(item_0)));"));
    assert!(members[0].contains("final nextMetrics = Map<String, List<int>>.fromEntries("));
    assert!(members[0].contains("return Catalog("));
}
