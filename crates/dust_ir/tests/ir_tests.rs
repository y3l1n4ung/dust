//! Integration tests for Dust IR helpers and compatibility surfaces.

use dust_diagnostics::Diagnostic;
use dust_ir::{
    AnnotationIr, AnnotationValueIr, BuiltinType, ClassIr, ClassKindIr, ConfigApplicationIr,
    ConstructorIr, ConstructorParamIr, DartFileIr, ExportIr, ExprSourceIr, FieldIr, ImportIr,
    LibraryDeclIr, LibraryIr, LoweringOutcome, NameIr, ParamKind, PartIr, PartOfIr,
    SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr, SpanIr, SymbolId, TypeIr,
    WorkspaceIr,
};
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(start, end))
}

#[test]
fn type_ir_helpers_work_for_named_and_nullable_types() {
    let named = TypeIr::string();
    let generic = TypeIr::list_of(TypeIr::named("User"));
    let nullable = generic.clone().nullable();
    let mapped = TypeIr::map_of(TypeIr::string(), TypeIr::int());
    let unknown = TypeIr::unknown();

    assert!(!named.is_nullable());
    assert!(!generic.is_nullable());
    assert!(nullable.is_nullable());
    assert!(named.is_builtin(BuiltinType::String));
    assert!(named.is_named("String"));
    assert_eq!(named.name(), Some("String"));
    assert!(mapped.is_named("Map"));
    assert_eq!(mapped.args().len(), 2);
    assert_eq!(TypeIr::dynamic().name(), None);
    assert_eq!(unknown.name(), None);
}

#[test]
fn type_ir_distinguishes_function_and_record_shapes() {
    let function = TypeIr::function("String Function(int)");
    let nullable_function = function.clone().nullable();
    let record = TypeIr::record("(String, int)");
    let nullable_record = record.clone().nullable();

    assert!(function.is_function());
    assert!(!function.is_record());
    assert!(nullable_function.is_nullable());
    assert_eq!(function.name(), None);
    assert!(record.is_record());
    assert!(!record.is_function());
    assert!(nullable_record.is_nullable());
    assert_eq!(record.args(), &[]);
}

#[test]
fn serde_configs_report_when_they_are_effectively_empty() {
    let mut class_config = SerdeClassConfigIr::default();
    let mut field_config = SerdeFieldConfigIr::default();

    assert!(class_config.is_empty());
    assert!(field_config.is_empty());

    class_config.rename_all = Some(SerdeRenameRuleIr::SnakeCase);
    field_config.aliases.push("legacy_name".to_owned());

    assert!(!class_config.is_empty());
    assert!(!field_config.is_empty());
}

#[test]
fn serde_configs_preserve_normalized_values() {
    let class_config = SerdeClassConfigIr {
        rename: Some("user_profile".to_owned()),
        rename_all: Some(SerdeRenameRuleIr::CamelCase),
        disallow_unrecognized_keys: true,
        ..Default::default()
    };
    let field_config = SerdeFieldConfigIr {
        rename: Some("display_name".to_owned()),
        aliases: vec!["displayName".to_owned(), "display-name".to_owned()],
        codec_source: Some("const UserIdCodec()".to_owned()),
        default_value_source: Some("const []".to_owned()),
        default_value: None,
        skip_serializing: false,
        skip_deserializing: true,
    };

    assert_eq!(class_config.rename.as_deref(), Some("user_profile"));
    assert_eq!(class_config.rename_all, Some(SerdeRenameRuleIr::CamelCase));
    assert!(class_config.disallow_unrecognized_keys);
    assert_eq!(field_config.rename.as_deref(), Some("display_name"));
    assert_eq!(field_config.aliases.len(), 2);
    assert_eq!(
        field_config.codec_source.as_deref(),
        Some("const UserIdCodec()")
    );
    assert_eq!(
        field_config.default_value_source.as_deref(),
        Some("const []")
    );
    assert!(!field_config.skip_serializing);
    assert!(field_config.skip_deserializing);
}

#[test]
fn constructor_knows_when_all_fields_are_constructible() {
    let fields = vec![
        FieldIr {
            name: "name".to_owned(),
            ty: TypeIr::named("String"),
            span: span(10, 20),
            has_default: false,
            serde: None,
            configs: Vec::new(),
        },
        FieldIr {
            name: "age".to_owned(),
            ty: TypeIr::named("int").nullable(),
            span: span(21, 29),
            has_default: true,
            serde: None,
            configs: Vec::new(),
        },
    ];
    let constructor = ConstructorIr {
        name: None,
        is_factory: false,
        redirected_target_source: None,
        redirected_target_name: None,
        span: span(30, 48),
        params: vec![ConstructorParamIr {
            name: "name".to_owned(),
            ty: TypeIr::named("String"),
            span: span(36, 45),
            kind: ParamKind::Positional,
            has_default: false,
            default_value_source: None,
        }],
    };

    assert!(constructor.can_construct_all_fields(&fields));
}

#[test]
fn constructor_detects_missing_required_fields() {
    let fields = vec![
        FieldIr {
            name: "name".to_owned(),
            ty: TypeIr::string(),
            span: span(10, 20),
            has_default: false,
            serde: None,
            configs: Vec::new(),
        },
        FieldIr {
            name: "age".to_owned(),
            ty: TypeIr::int(),
            span: span(21, 29),
            has_default: false,
            serde: None,
            configs: Vec::new(),
        },
    ];
    let constructor = ConstructorIr {
        name: Some("partial".to_owned()),
        is_factory: false,
        redirected_target_source: None,
        redirected_target_name: None,
        span: span(30, 48),
        params: vec![ConstructorParamIr {
            name: "name".to_owned(),
            ty: TypeIr::string(),
            span: span(36, 45),
            kind: ParamKind::Positional,
            has_default: false,
            default_value_source: None,
        }],
    };

    assert!(!constructor.can_construct_all_fields(&fields));
}

#[test]
fn lowering_outcome_preserves_diagnostics_when_mapping() {
    let outcome = LoweringOutcome::new(3_u32)
        .with_diagnostic(Diagnostic::warning("field uses fallback type"));
    let mapped = outcome.map(|value| value + 1);

    assert_eq!(mapped.value, 4);
    assert_eq!(mapped.diagnostics.len(), 1);
}

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
    let legacy: LibraryIr = file.clone();

    assert_eq!(legacy.source_path, "lib/user.dart");
    assert_eq!(legacy.imports, ["package:dust_dart/dust_dart.dart"]);
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
