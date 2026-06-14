use dust_parser_dart::{
    ParsedAnnotation, ParsedClassKind, ParsedClassSurface, ParsedEnumSurface, ParsedFieldSurface,
    ParsedLibrarySurface,
};
use dust_plugin_api::{DustPlugin, WorkspaceAnalysisBuilder, WorkspaceAnalysisContext};
use dust_state_plugin::register_plugin;
use dust_text::TextRange;

fn span() -> TextRange {
    TextRange::new(0_u32, 1_u32)
}

fn annotation(args: &str) -> ParsedAnnotation {
    ParsedAnnotation {
        name: "ViewModel".to_owned(),
        arguments_source: Some(args.to_owned()),
        parsed_arguments: None,
        span: span(),
    }
}

fn field(name: &str, type_source: Option<&str>) -> ParsedFieldSurface {
    ParsedFieldSurface {
        name: name.to_owned(),
        annotations: Vec::new(),
        type_source: type_source.map(str::to_owned),
        parsed_type: None,
        has_default: false,
        span: span(),
    }
}

fn class(
    name: &str,
    annotations: Vec<ParsedAnnotation>,
    fields: Vec<ParsedFieldSurface>,
) -> ParsedClassSurface {
    ParsedClassSurface {
        kind: ParsedClassKind::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        annotations,
        fields,
        constructors: Vec::new(),
        methods: Vec::new(),
        span: span(),
    }
}

fn parsed_library(classes: Vec<ParsedClassSurface>) -> ParsedLibrarySurface {
    parsed_library_with_enums(classes, Vec::new())
}

fn parsed_library_with_enums(
    classes: Vec<ParsedClassSurface>,
    enums: Vec<ParsedEnumSurface>,
) -> ParsedLibrarySurface {
    ParsedLibrarySurface {
        span: TextRange::new(0_u32, 100_u32),
        directives: Vec::new(),
        classes,
        enums,
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        query_calls: Vec::new(),
    }
}

fn enum_(name: &str) -> ParsedEnumSurface {
    ParsedEnumSurface {
        name: name.to_owned(),
        annotations: Vec::new(),
        variants: Vec::new(),
        span: span(),
    }
}

#[test]
fn collects_state_and_view_model_workspace_facts() {
    let plugin = register_plugin();
    let library = parsed_library(vec![
        class(
            "TaskBoardState",
            Vec::new(),
            vec![
                field("count", Some("int")),
                field("message", Some("String?")),
            ],
        ),
        class(
            "TaskBoardViewModel",
            vec![annotation(
                "(state: TaskBoardState, args: TaskBoardArgs, initial: const TaskBoardState())",
            )],
            Vec::new(),
        ),
    ]);
    let mut builder = WorkspaceAnalysisBuilder::default();

    plugin.collect_workspace_analysis(
        WorkspaceAnalysisContext {
            package_name: "state_test",
            package_root: std::path::Path::new("/workspace/app"),
            source_path: std::path::Path::new("/workspace/app/lib/features/task_board.dart"),
        },
        &library,
        &mut builder,
    );
    let snapshot = builder.snapshot();

    let states = snapshot.string_set("dust_state.states.v1").unwrap();
    assert_eq!(states.len(), 2);
    assert!(states.iter().any(|state| {
        state.contains(r#""class_name":"TaskBoardState""#)
            && state.contains(r#""name":"count","type_source":"int""#)
            && state.contains(r#""name":"message","type_source":"String?""#)
    }));

    let view_models = snapshot.string_set("dust_state.view_models.v1").unwrap();
    assert_eq!(
        view_models,
        &[r#"{"class_name":"TaskBoardViewModel","state_type":"TaskBoardState","args_type":"TaskBoardArgs","initial_source":"const TaskBoardState()","generated_base_class":"$TaskBoardViewModel","import_uri":"package:state_test/features/task_board.dart"}"#.to_owned()]
    );
}

#[test]
fn state_facts_sanitize_untyped_and_non_visible_imported_types() {
    let plugin = register_plugin();
    let library = parsed_library_with_enums(
        vec![
            class(
                "LooseState",
                Vec::new(),
                vec![
                    field("payload", None),
                    field("items", Some("List<Product>")),
                    field("status", Some("ProductsStatus")),
                ],
            ),
            class(
                "LooseViewModel",
                vec![annotation("(state: LooseState)")],
                Vec::new(),
            ),
        ],
        vec![enum_("ProductsStatus")],
    );
    let mut builder = WorkspaceAnalysisBuilder::default();

    plugin.collect_workspace_analysis(
        WorkspaceAnalysisContext {
            package_name: "state_test",
            package_root: std::path::Path::new("/workspace/app"),
            source_path: std::path::Path::new("/tmp/generated/loose.dart"),
        },
        &library,
        &mut builder,
    );
    let snapshot = builder.snapshot();

    let states = snapshot.string_set("dust_state.states.v1").unwrap();
    assert!(states.iter().any(|state| {
        state.contains(r#""class_name":"LooseState""#)
            && state.contains(r#""name":"payload","type_source":"dynamic""#)
            && state.contains(r#""name":"items","type_source":"List<Object?>""#)
            && state.contains(r#""name":"status","type_source":"ProductsStatus""#)
    }));
    let view_models = snapshot.string_set("dust_state.view_models.v1").unwrap();
    assert!(view_models[0].contains(r#""args_type":null"#));
    assert!(view_models[0].contains(r#""import_uri":"/tmp/generated/loose.dart""#));
}
