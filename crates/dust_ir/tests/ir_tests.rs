use dust_diagnostics::Diagnostic;
use dust_ir::{
    BuiltinType, ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr,
    LoweringOutcome, ParamKind, SerdeClassConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr, SpanIr,
    TypeIr, WorkspaceIr,
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
    };
    let field_config = SerdeFieldConfigIr {
        rename: Some("display_name".to_owned()),
        aliases: vec!["displayName".to_owned(), "display-name".to_owned()],
        default_value_source: Some("const []".to_owned()),
        skip_serializing: false,
        skip_deserializing: true,
    };

    assert_eq!(class_config.rename.as_deref(), Some("user_profile"));
    assert_eq!(class_config.rename_all, Some(SerdeRenameRuleIr::CamelCase));
    assert!(class_config.disallow_unrecognized_keys);
    assert_eq!(field_config.rename.as_deref(), Some("display_name"));
    assert_eq!(field_config.aliases.len(), 2);
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
        },
        FieldIr {
            name: "age".to_owned(),
            ty: TypeIr::named("int").nullable(),
            span: span(21, 29),
            has_default: true,
            serde: None,
        },
    ];
    let constructor = ConstructorIr {
        name: None,
        span: span(30, 48),
        params: vec![ConstructorParamIr {
            name: "name".to_owned(),
            ty: TypeIr::named("String"),
            span: span(36, 45),
            kind: ParamKind::Positional,
            has_default: false,
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
        },
        FieldIr {
            name: "age".to_owned(),
            ty: TypeIr::int(),
            span: span(21, 29),
            has_default: false,
            serde: None,
        },
    ];
    let constructor = ConstructorIr {
        name: Some("partial".to_owned()),
        span: span(30, 48),
        params: vec![ConstructorParamIr {
            name: "name".to_owned(),
            ty: TypeIr::string(),
            span: span(36, 45),
            kind: ParamKind::Positional,
            has_default: false,
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
    workspace.push_library(LibraryIr {
        source_path: "lib/user.dart".to_owned(),
        output_path: "lib/user.g.dart".to_owned(),
        span: span(0, 100),
        classes: vec![ClassIr {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(10, 90),
            fields: Vec::new(),
            constructors: Vec::new(),
            traits: Vec::new(),
            serde: None,
        }],
    });

    assert_eq!(workspace.libraries.len(), 1);
    assert_eq!(workspace.libraries[0].classes[0].name, "User");
}
