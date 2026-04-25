use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr, SpanIr, SymbolId,
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

fn library(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        span: span(0, 200),
        classes,
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
fn plugin_claims_serde_traits_and_config() {
    let plugin = register_plugin();

    let trait_names = plugin
        .claimed_traits()
        .into_iter()
        .map(|item| item.0)
        .collect::<Vec<_>>();
    let config_names = plugin
        .claimed_configs()
        .into_iter()
        .map(|item| item.0)
        .collect::<Vec<_>>();

    assert_eq!(
        trait_names,
        vec![
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize"
        ]
    );
    assert_eq!(config_names, vec!["derive_serde_annotation::SerDe"]);
}

#[test]
fn validates_abstract_deserialize_and_unsupported_field_types() {
    let plugin = register_plugin();
    let mut target = class(
        "Payload",
        vec![
            field("id", TypeIr::string()),
            field("transform", TypeIr::function("void Function(String)")),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("id", TypeIr::string(), ParamKind::Positional),
                constructor_param(
                    "transform",
                    TypeIr::function("void Function(String)"),
                    ParamKind::Positional,
                ),
            ],
        )],
        &["derive_serde_annotation::Deserialize"],
    );
    target.is_abstract = true;

    let diagnostics = plugin.validate(&library(vec![target]));

    assert!(diagnostics.iter().any(|item| {
        item.message
            .contains("`Deserialize` cannot target abstract class `Payload`")
    }));
    assert!(diagnostics.iter().any(|item| {
        item.message
            .contains("`Deserialize` does not support function types on `Payload.transform`")
    }));
}

#[test]
fn validates_field_skip_deserializing_requires_default() {
    let plugin = register_plugin();
    let mut name = field("name", TypeIr::string());
    name.serde = Some(SerdeFieldConfigIr {
        skip_deserializing: true,
        ..SerdeFieldConfigIr::default()
    });
    let target = class(
        "User",
        vec![name],
        vec![constructor(
            None,
            vec![constructor_param(
                "name",
                TypeIr::string(),
                ParamKind::Positional,
            )],
        )],
        &["derive_serde_annotation::Deserialize"],
    );

    let diagnostics = plugin.validate(&library(vec![target]));

    assert!(diagnostics.iter().any(|item| {
        item.message.contains(
            "field `name` on class `User` uses `skipDeserializing` without a `defaultValue`",
        )
    }));
}

#[test]
fn emits_to_json_and_from_json_with_serde_rules() {
    let plugin = register_plugin();

    let mut display_name = field("displayName", TypeIr::string().nullable());
    display_name.serde = Some(SerdeFieldConfigIr {
        rename: Some("full_name".to_owned()),
        aliases: vec!["fullName".to_owned()],
        ..SerdeFieldConfigIr::default()
    });

    let mut transient = field("transientValue", TypeIr::string());
    transient.serde = Some(SerdeFieldConfigIr {
        skip_serializing: true,
        default_value_source: Some("'internal'".to_owned()),
        ..SerdeFieldConfigIr::default()
    });

    let mut user = class(
        "User",
        vec![
            field("id", TypeIr::string()),
            display_name,
            field("tags", TypeIr::list_of(TypeIr::string())),
            transient,
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("id", TypeIr::string(), ParamKind::Positional),
                constructor_param(
                    "displayName",
                    TypeIr::string().nullable(),
                    ParamKind::Positional,
                ),
                constructor_param(
                    "tags",
                    TypeIr::list_of(TypeIr::string()),
                    ParamKind::Positional,
                ),
                constructor_param("transientValue", TypeIr::string(), ParamKind::Positional),
            ],
        )],
        &[
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize",
        ],
    );
    user.serde = Some(SerdeClassConfigIr {
        rename_all: Some(SerdeRenameRuleIr::SnakeCase),
        disallow_unrecognized_keys: true,
        ..SerdeClassConfigIr::default()
    });

    let contribution = plugin.emit(&library(vec![user]), &SymbolPlan::default());
    let mixin = &members_for_class(&contribution, "User")[0];
    let to_json = contribution
        .top_level_functions
        .iter()
        .find(|item| item.contains("_$UserToJson"))
        .unwrap();
    let from_json = contribution
        .top_level_functions
        .iter()
        .find(|item| item.contains("_$UserFromJson"))
        .unwrap();

    assert!(mixin.contains("Map<String, Object?> toJson() => _$UserToJson(_dustSelf);"));
    assert!(to_json.contains("'id': instance.id"));
    assert!(to_json.contains("'full_name': instance.displayName"));
    assert!(to_json.contains("instance.tags.map((item) => item).toList()"));
    assert!(!to_json.contains("transient_value"));

    assert!(from_json.contains(
        "const allowedKeys = <String>{'id', 'full_name', 'fullName', 'tags', 'transient_value'};"
    ));
    assert!(from_json.contains("final rawDisplayName = json.containsKey('full_name') ? json['full_name'] : json.containsKey('fullName') ? json['fullName'] : null;"));
    assert!(
        from_json.contains("final transientValueValue = json.containsKey('transient_value') ?")
    );
    assert!(from_json.contains(": 'internal';"));
    assert!(from_json.contains("return User("));
}

#[test]
fn emits_nested_model_and_map_support() {
    let plugin = register_plugin();
    let child = class(
        "Profile",
        vec![field("handle", TypeIr::string())],
        vec![constructor(
            None,
            vec![constructor_param(
                "handle",
                TypeIr::string(),
                ParamKind::Positional,
            )],
        )],
        &[
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize",
        ],
    );
    let parent = class(
        "Account",
        vec![
            field("profile", TypeIr::named("Profile")),
            field(
                "metrics",
                TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
            ),
        ],
        vec![constructor(
            None,
            vec![
                constructor_param("profile", TypeIr::named("Profile"), ParamKind::Positional),
                constructor_param(
                    "metrics",
                    TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                    ParamKind::Positional,
                ),
            ],
        )],
        &[
            "derive_serde_annotation::Serialize",
            "derive_serde_annotation::Deserialize",
        ],
    );

    let contribution = plugin.emit(&library(vec![child, parent]), &SymbolPlan::default());
    let account_to_json = contribution
        .top_level_functions
        .iter()
        .find(|item| item.contains("_$AccountToJson"))
        .unwrap();
    let account_from_json = contribution
        .top_level_functions
        .iter()
        .find(|item| item.contains("_$AccountFromJson"))
        .unwrap();

    assert!(account_to_json.contains("'profile': _$ProfileToJson(instance.profile)"));
    assert!(account_to_json.contains(
        "instance.metrics.map((key, value) => MapEntry(key, value.map((item) => item).toList()))"
    ));
    assert!(account_from_json.contains(
        "final profileValue = _$ProfileFromJson(Map<String, Object?>.from(rawProfile as Map));"
    ));
    assert!(account_from_json.contains("Map<String, Object?>.from(rawMetrics as Map).map((key, value) => MapEntry(key, (value as List<Object?>).map((item) => item as int).toList()))"));
}
