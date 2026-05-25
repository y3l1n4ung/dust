use dust_db_plugin::register_plugin;
use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr, FieldIr,
    LibraryIr, ParamKind, SpanIr, SymbolId, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_text::{FileId, TextRange};

fn span() -> SpanIr {
    SpanIr::new(FileId::new(7), TextRange::new(0_u32, 1_u32))
}

fn config(symbol: &str, args: &str) -> ConfigApplicationIr {
    ConfigApplicationIr {
        symbol: SymbolId::new(symbol),
        arguments_source: Some(args.to_owned()),
        span: span(),
    }
}

fn field(name: &str, ty: TypeIr, configs: Vec<ConfigApplicationIr>) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(),
        has_default: false,
        serde: None,
        configs,
    }
}

fn named_param(name: &str, ty: TypeIr, has_default: bool) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(),
        kind: ParamKind::Named,
        has_default,
        default_value_source: None,
    }
}

fn row_class() -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "UserProfile".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(),
        fields: vec![
            field("id", TypeIr::int(), Vec::new()),
            field(
                "name",
                TypeIr::string(),
                vec![config("dust_db::Sqlx", "(rename: 'display_name')")],
            ),
            field(
                "bio",
                TypeIr::string(),
                vec![config("dust_db::Sqlx", "(defaultValue: '')")],
            ),
            field(
                "sessionActive",
                TypeIr::bool(),
                vec![config("dust_db::Sqlx", "(skip: true, defaultValue: false)")],
            ),
            field(
                "preferences",
                TypeIr::named("UserPreferences"),
                vec![config("dust_db::Sqlx", "(json: true)")],
            ),
            field(
                "status",
                TypeIr::named("UserStatus"),
                vec![config(
                    "dust_db::Sqlx",
                    "(tryFrom: const UserStatusFromInt())",
                )],
            ),
        ],
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            span: span(),
            params: vec![
                named_param("id", TypeIr::int(), false),
                named_param("name", TypeIr::string(), false),
                named_param("bio", TypeIr::string(), false),
                named_param("sessionActive", TypeIr::bool(), false),
                named_param("preferences", TypeIr::named("UserPreferences"), false),
                named_param("status", TypeIr::named("UserStatus"), false),
            ],
        }],
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![
            config("dust_db::FromRow", "()"),
            config("dust_db::Sqlx", "(renameAll: SqlxRename.snakeCase)"),
        ],
        serde: None,
    }
}

fn library(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        package_root: String::new(),
        package_name: "example".to_owned(),
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        imports: Vec::new(),
        span: span(),
        classes,
        enums: Vec::new(),
    }
}

#[test]
fn emits_sqlx_style_from_row_mapper() {
    let plugin = register_plugin();
    let contribution = plugin.emit(&library(vec![row_class()]), &SymbolPlan::default());

    assert_eq!(
        contribution.support_types[0],
        r#"extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Map<String, Object?> row) => UserProfile(
    id: row['id'] as int,
    name: row['display_name'] as String,
    bio: row.containsKey('bio') ? row['bio'] as String : '',
    sessionActive: false,
    preferences: UserPreferences.fromJson(
      jsonDecode(row['preferences'] as String) as Map<String, Object?>,
    ),
    status: const UserStatusFromInt().decode(row['status'] as int),
  );
}"#
    );
}

#[test]
fn validates_duplicate_effective_columns() {
    let mut class = row_class();
    class.fields[1].configs = Vec::new();
    class.fields[1].name = "id".to_owned();
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("duplicate SQL column `id`")),
        "{diagnostics:?}"
    );
}
