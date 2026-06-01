use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use dust_db_plugin::{register_plugin, register_row_plugin};
use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr, FieldIr,
    LibraryIr, MethodIr, MethodParamIr, ParamKind, SpanIr, SymbolId, TraitApplicationIr, TypeIr,
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

fn trait_app(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
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

fn method_param(name: &str, ty: TypeIr) -> MethodParamIr {
    MethodParamIr {
        name: name.to_owned(),
        ty,
        span: span(),
        kind: ParamKind::Positional,
        has_default: false,
        default_value_source: None,
        traits: Vec::new(),
        configs: Vec::new(),
    }
}

fn result_type(ok: TypeIr) -> TypeIr {
    TypeIr::generic(
        "Future",
        vec![TypeIr::generic(
            "Result",
            vec![ok, TypeIr::named("SqlxError")],
        )],
    )
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
                vec![config(
                    "dust_db_annotation::Sqlx",
                    "(rename: 'display_name')",
                )],
            ),
            field(
                "bio",
                TypeIr::string(),
                vec![config("dust_db_annotation::Sqlx", "(defaultValue: '')")],
            ),
            field(
                "sessionActive",
                TypeIr::bool(),
                vec![config(
                    "dust_db_annotation::Sqlx",
                    "(skip: true, defaultValue: false)",
                )],
            ),
            field(
                "preferences",
                TypeIr::named("UserPreferences"),
                vec![config("dust_db_annotation::Sqlx", "(json: true)")],
            ),
            field(
                "status",
                TypeIr::named("UserStatus"),
                vec![config(
                    "dust_db_annotation::Sqlx",
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
        traits: vec![trait_app("dust_db_annotation::FromRow")],
        configs: vec![config(
            "dust_db_annotation::Sqlx",
            "(renameAll: SqlxRename.snakeCase)",
        )],
        serde: None,
    }
}

fn dao_class() -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "UserDao".to_owned(),
        is_abstract: true,
        is_interface: false,
        superclass_name: None,
        span: span(),
        fields: Vec::new(),
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: true,
            redirected_target_source: Some("_$UserDao".to_owned()),
            redirected_target_name: Some("_$UserDao".to_owned()),
            span: span(),
            params: vec![ConstructorParamIr {
                name: "db".to_owned(),
                ty: TypeIr::named("SqlxDriver"),
                span: span(),
                kind: ParamKind::Positional,
                has_default: false,
                default_value_source: None,
            }],
        }],
        methods: vec![
            MethodIr {
                name: "findById".to_owned(),
                is_static: false,
                is_external: false,
                return_type: result_type(TypeIr::named("UserProfile").nullable()),
                has_body: false,
                body_source: None,
                params: vec![method_param("id", TypeIr::int())],
                span: span(),
                traits: Vec::new(),
                configs: vec![config(
                    "dust_db_annotation::Query",
                    "(r'SELECT id, display_name, bio FROM users WHERE id = $1')",
                )],
            },
            MethodIr {
                name: "count".to_owned(),
                is_static: false,
                is_external: false,
                return_type: result_type(TypeIr::int()),
                has_body: false,
                body_source: None,
                params: Vec::new(),
                span: span(),
                traits: Vec::new(),
                configs: vec![config(
                    "dust_db_annotation::Query",
                    "(r'SELECT COUNT(*) FROM users')",
                )],
            },
            MethodIr {
                name: "rename".to_owned(),
                is_static: false,
                is_external: false,
                return_type: result_type(TypeIr::named("ExecResult")),
                has_body: false,
                body_source: None,
                params: vec![
                    method_param("name", TypeIr::string()),
                    method_param("id", TypeIr::int()),
                ],
                span: span(),
                traits: Vec::new(),
                configs: vec![config(
                    "dust_db_annotation::Query",
                    "(r'UPDATE users SET display_name = $1 WHERE id = $2')",
                )],
            },
        ],
        traits: Vec::new(),
        configs: vec![config("dust_db_annotation::SqlxDao", "()")],
        serde: None,
    }
}

fn database_class() -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: "AppDatabase".to_owned(),
        is_abstract: true,
        is_interface: false,
        superclass_name: None,
        span: span(),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![config(
            "dust_db_annotation::SqlxDatabase",
            "(type: SqlxDatabaseType.sqlite, migrations: './migrations')",
        )],
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

fn library_with_imports(
    root: &std::path::Path,
    source_path: &str,
    imports: Vec<&str>,
    classes: Vec<ClassIr>,
) -> LibraryIr {
    LibraryIr {
        package_root: root.display().to_string(),
        package_name: "example".to_owned(),
        source_path: source_path.to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        imports: imports.into_iter().map(str::to_owned).collect(),
        span: span(),
        classes,
        enums: Vec::new(),
    }
}

#[test]
fn emits_sqlx_style_from_row_mapper() {
    let plugin = register_row_plugin();
    let contribution = plugin.emit(&library(vec![row_class()]), &SymbolPlan::default());

    assert_eq!(
        contribution.support_types[0],
        r#"extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Row row) {
    return UserProfile(
      id: row.read<int>('id'),
      name: row.read<String>('display_name'),
      bio: row.readOrNull<Object?>('bio') == null ? '' : row.read<String>('bio'),
      sessionActive: false,
      preferences: UserPreferences.fromJson(decodeJsonObject(row.read<String>('preferences'))),
      status: const UserStatusFromInt().decode(row.read<int>('status')),
    );
  }
}

final bool _$userProfileFromRowRegistered = registerRowMapper<UserProfile>(UserProfileFromRow.fromRow);"#
    );
}

#[test]
fn emits_sqlx_style_dao_redirecting_factory_impl() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![row_class(), dao_class()]),
        &SymbolPlan::default(),
    );

    assert_eq!(
        contribution.support_types[0],
        r#"extension UserProfileFromRow on UserProfile {
  static UserProfile fromRow(Row row) {
    return UserProfile(
      id: row.read<int>('id'),
      name: row.read<String>('display_name'),
      bio: row.readOrNull<Object?>('bio') == null ? '' : row.read<String>('bio'),
      sessionActive: false,
      preferences: UserPreferences.fromJson(decodeJsonObject(row.read<String>('preferences'))),
      status: const UserStatusFromInt().decode(row.read<int>('status')),
    );
  }
}

final bool _$userProfileFromRowRegistered = registerRowMapper<UserProfile>(UserProfileFromRow.fromRow);

final class _$UserDao implements UserDao {
  const _$UserDao(this._db);

  final SqlxDriver _db;

  @override
  Future<Result<UserProfile?, SqlxError>> findById(int id) {
    return _db.fetchOptional<UserProfile>(
      r'''SELECT id, display_name, bio FROM users WHERE id = $1''',
      [id],
      UserProfileFromRow.fromRow,
    );
  }

  @override
  Future<Result<int, SqlxError>> count() {
    return _db.fetchScalar<int>(
      r'''SELECT COUNT(*) FROM users''',
      [],
    );
  }

  @override
  Future<Result<ExecResult, SqlxError>> rename(String name, int id) {
    return _db.execute(
      r'''UPDATE users SET display_name = $1 WHERE id = $2''',
      [name, id],
    );
  }
}"#
    );
}

#[test]
fn emits_dao_mapper_for_imported_from_row_return_type() {
    let root = temp_root("imported_dao_rows");
    fs::create_dir_all(root.join("lib/models")).unwrap();
    fs::write(
        root.join("lib/models/user_profile.dart"),
        "@Derive([FromRow()])\nfinal class UserProfile {}\n",
    )
    .unwrap();

    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_imports(
            &root,
            "lib/dao/user_dao.dart",
            vec!["../models/user_profile.dart"],
            vec![dao_class()],
        ),
        &SymbolPlan::default(),
    );

    assert!(contribution.support_types[0].contains("UserProfileFromRow.fromRow"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_unknown_custom_dao_result_type() {
    let mut dao = dao_class();
    dao.methods[0].return_type = result_type(TypeIr::named("NotARow"));

    let diagnostics = register_plugin().validate(&library(vec![database_class(), dao]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unsupported SqlxDao result type `NotARow`")),
        "{diagnostics:?}"
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

#[test]
fn validates_skipped_field_requires_default() {
    let mut class = row_class();
    class.fields = vec![field(
        "sessionActive",
        TypeIr::bool(),
        vec![config("dust_db_annotation::Sqlx", "(skip: true)")],
    )];
    class.constructors[0].params = vec![named_param("sessionActive", TypeIr::bool(), false)];
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("uses `Sqlx(skip: true)` without a default")),
        "{diagnostics:?}"
    );
}

#[test]
fn validates_flatten_target_must_be_from_row() {
    let mut class = row_class();
    class.fields = vec![field(
        "address",
        TypeIr::named("Address"),
        vec![config("dust_db_annotation::Sqlx", "(flatten: true)")],
    )];
    class.constructors[0].params = vec![named_param("address", TypeIr::named("Address"), false)];
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("must reference an @FromRow class")),
        "{diagnostics:?}"
    );
}

#[test]
fn validates_conflicting_sqlx_field_options() {
    let mut json_flatten = row_class();
    json_flatten.fields = vec![field(
        "address",
        TypeIr::named("Address"),
        vec![config(
            "dust_db_annotation::Sqlx",
            "(flatten: true, json: true)",
        )],
    )];
    json_flatten.constructors[0].params =
        vec![named_param("address", TypeIr::named("Address"), false)];

    let mut try_from_flatten = row_class();
    try_from_flatten.fields = vec![field(
        "status",
        TypeIr::named("UserStatus"),
        vec![config(
            "dust_db_annotation::Sqlx",
            "(flatten: true, tryFrom: const UserStatusFromInt())",
        )],
    )];
    try_from_flatten.constructors[0].params =
        vec![named_param("status", TypeIr::named("UserStatus"), false)];

    let diagnostics = register_plugin().validate(&library(vec![json_flatten, try_from_flatten]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("cannot use both `json` and `flatten`")),
        "{diagnostics:?}"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("cannot use both `tryFrom` and `flatten`")),
        "{diagnostics:?}"
    );
}

#[test]
fn validates_unsupported_plain_row_field_type() {
    let mut class = row_class();
    class.fields = vec![field("tags", TypeIr::named("List<String>"), Vec::new())];
    class.constructors[0].params = vec![named_param("tags", TypeIr::named("List<String>"), false)];
    let diagnostics = register_plugin().validate(&library(vec![class]));

    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unsupported SQLx row field type")),
        "{diagnostics:?}"
    );
}

fn temp_root(name: &str) -> std::path::PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("dust_db_plugin_{name}_{stamp}"))
}
