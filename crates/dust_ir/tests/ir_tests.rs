use dust_diagnostics::Diagnostic;
use dust_ir::{
    BuiltinType, ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr,
    LoweringOutcome, ParamKind, SpanIr, TypeIr, WorkspaceIr,
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
