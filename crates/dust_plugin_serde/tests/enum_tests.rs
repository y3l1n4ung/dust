use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, EnumIr, EnumVariantIr, FieldIr,
    LibraryIr, ParamKind, SerdeClassConfigIr, SerdeRenameRuleIr, SpanIr, SymbolId,
    TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_serde::register_plugin;
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(11), TextRange::new(start, end))
}

fn trait_application(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: None,
    }
}

fn enum_variant(name: &str) -> EnumVariantIr {
    EnumVariantIr {
        name: name.to_owned(),
        span: span(10, 20),
    }
}

fn enum_ir(name: &str, variants: Vec<EnumVariantIr>, traits: &[&str]) -> EnumIr {
    EnumIr {
        name: name.to_owned(),
        span: span(0, 100),
        variants,
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        serde: None,
    }
}

fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(30, 35),
        kind,
        has_default: false,
    }
}

fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        span: span(25, 60),
        params,
    }
}

fn class(
    name: &str,
    fields: Vec<FieldIr>,
    constructors: Vec<ConstructorIr>,
    traits: &[&str],
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        serde: None,
    }
}

fn library(classes: Vec<ClassIr>, enums: Vec<EnumIr>) -> LibraryIr {
    LibraryIr {
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        span: span(0, 200),
        classes,
        enums,
    }
}

#[test]
fn generates_serde_for_enums() {
    let plugin = register_plugin();
    let library = library(
        vec![],
        vec![enum_ir(
            "Status",
            vec![enum_variant("pending"), enum_variant("active")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    assert_eq!(contribution.top_level_functions.len(), 2);

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("Object? _$StatusToJson(Status instance)"));
    assert!(to_json.contains("Status.pending => 'pending'"));
    assert!(to_json.contains("Status.active => 'active'"));

    assert!(from_json.contains("Status _$StatusFromJson(Object? json)"));
    assert!(from_json.contains("'pending' => Status.pending"));
    assert!(from_json.contains("'active' => Status.active"));
}

#[test]
fn supports_enum_renaming() {
    let plugin = register_plugin();
    let mut e = enum_ir(
        "UserRole",
        vec![enum_variant("superAdmin"), enum_variant("guestUser")],
        &[
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize",
        ],
    );
    e.serde = Some(SerdeClassConfigIr {
        rename_all: Some(SerdeRenameRuleIr::SnakeCase),
        ..Default::default()
    });

    let library = library(vec![], vec![e]);
    let contribution = plugin.emit(&library, &SymbolPlan::default());

    let to_json = &contribution.top_level_functions[0];
    let from_json = &contribution.top_level_functions[1];

    assert!(to_json.contains("UserRole.superAdmin => 'super_admin'"));
    assert!(to_json.contains("UserRole.guestUser => 'guest_user'"));

    assert!(from_json.contains("'super_admin' => UserRole.superAdmin"));
    assert!(from_json.contains("'guest_user' => UserRole.guestUser"));
}

#[test]
fn handles_enum_fields_in_classes() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![field("status", TypeIr::named("Status"))],
            vec![constructor(
                None,
                vec![constructor_param(
                    "status",
                    TypeIr::named("Status"),
                    ParamKind::Named,
                )],
            )],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
        vec![enum_ir(
            "Status",
            vec![enum_variant("active")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserToJson"))
        .unwrap();
    let from_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserFromJson"))
        .unwrap();

    assert!(to_json.contains("'status': _$StatusToJson(instance.status),"));
    assert!(from_json.contains("final statusValue = _$StatusFromJson(json['status']);"));
}

#[test]
fn handles_nullable_enum_fields() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "User",
            vec![field("status", TypeIr::named("Status").nullable())],
            vec![constructor(
                None,
                vec![constructor_param(
                    "status",
                    TypeIr::named("Status").nullable(),
                    ParamKind::Named,
                )],
            )],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
        vec![enum_ir(
            "Status",
            vec![enum_variant("active")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserToJson"))
        .unwrap();
    let from_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$UserFromJson"))
        .unwrap();

    assert!(to_json.contains(
        "'status': instance.status == null ? null : _$StatusToJson((instance.status!)),"
    ));
    assert!(from_json.contains("final statusValue = json['status'] == null\n                      ? null\n                      : _$StatusFromJson(json['status']);"));
}

#[test]
fn handles_enums_in_collections() {
    let plugin = register_plugin();
    let library = library(
        vec![class(
            "Bundle",
            vec![field(
                "roles",
                TypeIr::generic("List", vec![TypeIr::named("Role")]),
            )],
            vec![constructor(
                None,
                vec![constructor_param(
                    "roles",
                    TypeIr::generic("List", vec![TypeIr::named("Role")]),
                    ParamKind::Named,
                )],
            )],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
        vec![enum_ir(
            "Role",
            vec![enum_variant("admin")],
            &[
                "derive_serde_annotation::Serialize",
                "derive_serde_annotation::Deserialize",
            ],
        )],
    );

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let to_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$BundleToJson"))
        .unwrap();
    let from_json = contribution
        .top_level_functions
        .iter()
        .find(|f| f.contains("_$BundleFromJson"))
        .unwrap();

    assert!(
        to_json.contains("'roles': instance.roles.map((item) => _$RoleToJson(item)).toList(),")
    );
    assert!(from_json.contains("final rolesValue = _dustJsonAsList(json['roles'], 'roles').map((item) => _$RoleFromJson(item)).toList();"));
}
