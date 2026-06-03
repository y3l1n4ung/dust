use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, LibraryIr, MethodIr, ParamKind,
    QueryCallIr, TypeIr,
};

use super::ir::{config, field, method_param, named_param, result_type, span, trait_app};

pub(crate) fn row_class() -> ClassIr {
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
                vec![config("dust_dart::Sqlx", "(rename: 'display_name')")],
            ),
            field(
                "bio",
                TypeIr::string(),
                vec![config("dust_dart::Sqlx", "(defaultValue: '')")],
            ),
            field(
                "sessionActive",
                TypeIr::bool(),
                vec![config(
                    "dust_dart::Sqlx",
                    "(skip: true, defaultValue: false)",
                )],
            ),
            field(
                "preferences",
                TypeIr::named("UserPreferences"),
                vec![config("dust_dart::Sqlx", "(json: true)")],
            ),
            field(
                "status",
                TypeIr::named("UserStatus"),
                vec![config(
                    "dust_dart::Sqlx",
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
        traits: vec![trait_app("dust_dart::FromRow")],
        configs: vec![config(
            "dust_dart::Sqlx",
            "(renameAll: SqlxRename.snakeCase)",
        )],
        serde: None,
    }
}

pub(crate) fn simple_user_row_class() -> ClassIr {
    ClassIr {
        fields: vec![
            field("id", TypeIr::int(), Vec::new()),
            field(
                "name",
                TypeIr::string(),
                vec![config("dust_dart::Sqlx", "(rename: 'display_name')")],
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
            ],
        }],
        configs: Vec::new(),
        ..row_class()
    }
}

pub(crate) fn dao_class() -> ClassIr {
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
        methods: vec![find_by_id_method(), count_method(), rename_method()],
        traits: Vec::new(),
        configs: vec![config("dust_dart::SqlxDao", "()")],
        serde: None,
    }
}

pub(crate) fn database_class() -> ClassIr {
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
            "dust_dart::SqlxDatabase",
            "(type: SqlxDatabaseType.sqlite, migrations: './migrations')",
        )],
        serde: None,
    }
}

pub(crate) fn library(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        package_root: String::new(),
        package_name: "example".to_owned(),
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        imports: Vec::new(),
        span: span(),
        classes,
        enums: Vec::new(),
        query_calls: Vec::new(),
    }
}

pub(crate) fn library_with_imports(
    root: &std::path::Path,
    source_path: &str,
    imports: Vec<&str>,
    classes: Vec<ClassIr>,
) -> LibraryIr {
    LibraryIr {
        package_root: root.display().to_string(),
        source_path: source_path.to_owned(),
        imports: imports.into_iter().map(str::to_owned).collect(),
        ..library(classes)
    }
}

pub(crate) fn library_with_queries(
    root: &std::path::Path,
    classes: Vec<ClassIr>,
    query_calls: Vec<QueryCallIr>,
) -> LibraryIr {
    LibraryIr {
        package_root: root.display().to_string(),
        source_path: "lib/db.dart".to_owned(),
        query_calls,
        ..library(classes)
    }
}

fn find_by_id_method() -> MethodIr {
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
            "dust_dart::Query",
            "(r'SELECT id, display_name, bio FROM users WHERE id = $1')",
        )],
    }
}

fn count_method() -> MethodIr {
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
            "dust_dart::Query",
            "(r'SELECT COUNT(*) FROM users')",
        )],
    }
}

fn rename_method() -> MethodIr {
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
            "dust_dart::Query",
            "(r'UPDATE users SET display_name = $1 WHERE id = $2')",
        )],
    }
}
