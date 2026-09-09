//! The canonical library model, its directives and its annotation values.

use super::*;

#[test]
fn workspace_ir_collects_libraries() {
    let mut workspace = WorkspaceIr::default();
    let mut file = DartFileIr::empty(
        ".",
        "dust_test",
        "lib/user.dart",
        "lib/user.g.dart",
        span(0, 100),
    );
    file.classes.push(ClassIr {
        kind: ClassKindIr::Class,
        name: "User".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(10, 90),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
        serde: None,
    });
    workspace.push_library(file);

    assert_eq!(workspace.libraries.len(), 1);
    assert_eq!(workspace.libraries[0].classes[0].name, "User");
}

#[test]
fn dart_file_ir_is_the_canonical_library_model() {
    let mut file = DartFileIr::empty(
        ".",
        "dust_test",
        "lib/user.dart",
        "lib/user.g.dart",
        span(0, 100),
    );
    file.imports
        .push("package:dust_dart/dust_dart.dart".to_owned());
    let copy: DartFileIr = file.clone();

    assert_eq!(copy.source_path, "lib/user.dart");
    assert_eq!(copy.imports, ["package:dust_dart/dust_dart.dart"]);
}

#[test]
fn dart_file_ir_empty_initializes_additive_surfaces() {
    let file = DartFileIr::empty(
        ".",
        "dust_test",
        "lib/empty.dart",
        "lib/empty.g.dart",
        span(0, 1),
    );

    assert!(file.library.is_none());
    assert!(file.library_annotations.is_empty());
    assert!(file.import_directives.is_empty());
    assert!(file.export_directives.is_empty());
    assert!(file.part_directives.is_empty());
    assert!(file.part_of.is_none());
    assert!(file.mixins.is_empty());
    assert!(file.extensions.is_empty());
    assert!(file.extension_types.is_empty());
    assert!(file.functions.is_empty());
    assert!(file.variables.is_empty());
    assert!(file.typedefs.is_empty());
    assert!(file.classes.is_empty());
    assert!(file.enums.is_empty());
    assert!(file.query_calls.is_empty());
}

#[test]
fn dart_file_ir_preserves_directive_surfaces() {
    let library_name = NameIr {
        source: "models.user".to_owned(),
        short: "user".to_owned(),
        prefix: Some("models".to_owned()),
        span: span(8, 19),
    };
    let mut file = DartFileIr::empty(
        ".",
        "dust_test",
        "lib/user.dart",
        "lib/user.g.dart",
        span(0, 100),
    );
    file.library = Some(LibraryDeclIr {
        name: Some(library_name.clone()),
        span: span(0, 20),
    });
    file.import_directives.push(ImportIr {
        uri: "package:dust_dart/dust_dart.dart".to_owned(),
        prefix: Some("dust".to_owned()),
        show: vec!["Derive".to_owned()],
        hide: vec!["Internal".to_owned()],
        is_deferred: false,
        span: span(21, 67),
    });
    file.export_directives.push(ExportIr {
        uri: "src/user_api.dart".to_owned(),
        span: span(68, 95),
    });
    file.part_directives.push(PartIr {
        uri: "user.g.dart".to_owned(),
        span: span(96, 115),
    });
    file.part_of = Some(PartOfIr {
        library_name: Some(library_name),
        uri: None,
        span: span(116, 137),
    });

    assert_eq!(
        file.library
            .as_ref()
            .and_then(|library| library.name.as_ref())
            .map(|name| name.source.as_str()),
        Some("models.user")
    );
    assert_eq!(file.import_directives[0].prefix.as_deref(), Some("dust"));
    assert_eq!(file.import_directives[0].show, ["Derive"]);
    assert_eq!(file.import_directives[0].hide, ["Internal"]);
    assert!(!file.import_directives[0].is_deferred);
    assert_eq!(file.export_directives[0].uri, "src/user_api.dart");
    assert_eq!(file.part_directives[0].uri, "user.g.dart");
    assert_eq!(
        file.part_of
            .as_ref()
            .and_then(|part_of| part_of.library_name.as_ref())
            .map(|name| name.short.as_str()),
        Some("user")
    );
}

#[test]
fn annotation_ir_preserves_prefix_and_structured_values() {
    let name = NameIr {
        source: "dust.SerDe".to_owned(),
        short: "SerDe".to_owned(),
        prefix: Some("dust".to_owned()),
        span: span(4, 14),
    };
    let annotation = AnnotationIr {
        raw_name: name.source.clone(),
        short_name: name.short.clone(),
        prefix: name.prefix.clone(),
        positional_args: Vec::new(),
        named_args: [(
            "rename".to_owned(),
            AnnotationValueIr::String("user_id".to_owned()),
        )]
        .into(),
        resolved_symbol: None,
        span: span(3, 33),
    };
    let import = ImportIr {
        uri: "package:dust_dart/dust_dart.dart".to_owned(),
        prefix: Some("dust".to_owned()),
        show: Vec::new(),
        hide: Vec::new(),
        is_deferred: false,
        span: span(0, 32),
    };

    assert_eq!(annotation.raw_name, "dust.SerDe");
    assert_eq!(annotation.short_name, "SerDe");
    assert_eq!(annotation.prefix.as_deref(), Some("dust"));
    assert_eq!(annotation.named_args.len(), 1);
    assert_eq!(import.prefix.as_deref(), Some("dust"));
}

#[test]
fn config_application_ir_preserves_structured_arguments() {
    let config = ConfigApplicationIr::with_arguments(
        SymbolId::new("dust_dart::SerDe"),
        None,
        Vec::new(),
        [(
            "renameAll".to_owned(),
            AnnotationValueIr::Expression(ExprSourceIr {
                source: "SerDeRename.snakeCase".to_owned(),
                span: span(12, 36),
            }),
        )]
        .into(),
        span(0, 37),
    );
    let compat = ConfigApplicationIr::new(
        SymbolId::new("dust_dart::SerDe"),
        Some("(rename: 'user_id')".to_owned()),
        span(0, 20),
    );

    let Some(AnnotationValueIr::Expression(value)) = config.named_argument_value("renameAll")
    else {
        panic!("expected structured renameAll expression");
    };
    assert_eq!(value.source, "SerDeRename.snakeCase");
    assert_eq!(
        config.named_argument_source("renameAll"),
        Some("SerDeRename.snakeCase")
    );
    assert_eq!(
        config.named_expression_source("renameAll").as_deref(),
        Some("SerDeRename.snakeCase")
    );
    assert!(config.positional_args.is_empty());
    assert!(config.arguments_source.is_none());
    assert!(compat.named_args.is_empty());
}

#[test]
fn config_accessors_prefer_structured_annotation_values() {
    let mut named_args = BTreeMap::new();
    named_args.insert("enabled".to_owned(), AnnotationValueIr::Bool(true));
    named_args.insert(
        "tags".to_owned(),
        AnnotationValueIr::List(vec![
            AnnotationValueIr::String("one".to_owned()),
            AnnotationValueIr::String("two".to_owned()),
        ]),
    );
    named_args.insert(
        "headers".to_owned(),
        AnnotationValueIr::Map(vec![(
            AnnotationValueIr::String("Accept".to_owned()),
            AnnotationValueIr::String("application/json".to_owned()),
        )]),
    );
    named_args.insert(
        "codec".to_owned(),
        AnnotationValueIr::Member(NameIr {
            source: "UserCodec".to_owned(),
            short: "UserCodec".to_owned(),
            prefix: None,
            span: span(0, 8),
        }),
    );
    named_args.insert(
        "guards".to_owned(),
        AnnotationValueIr::List(vec![AnnotationValueIr::Member(NameIr {
            source: "AuthGuard".to_owned(),
            short: "AuthGuard".to_owned(),
            prefix: None,
            span: span(0, 9),
        })]),
    );

    let config = ConfigApplicationIr::with_arguments(
        SymbolId::new("dust.TestConfig"),
        None,
        vec![AnnotationValueIr::String("query".to_owned())],
        named_args,
        span(0, 1),
    );

    assert_eq!(config.positional_string(0).as_deref(), Some("query"));
    assert_eq!(config.named_bool("enabled"), Some(true));
    assert_eq!(
        config.named_string_list("tags"),
        Some(vec!["one".to_owned(), "two".to_owned()])
    );
    assert_eq!(
        config.named_string_map("headers"),
        Some(vec![("Accept".to_owned(), "application/json".to_owned(),)])
    );
    assert_eq!(config.named_member("codec"), Some("UserCodec".to_owned()));
    assert_eq!(
        config.named_type_list("guards"),
        Some(vec!["AuthGuard".to_owned()])
    );
}
