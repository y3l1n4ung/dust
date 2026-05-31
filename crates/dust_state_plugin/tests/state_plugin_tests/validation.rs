use dust_ir::{ConstructorIr, ConstructorParamIr, FieldIr, ParamKind, TypeIr};
use dust_plugin_api::DustPlugin;
use dust_state_plugin::register_plugin;

use super::support::{
    args_class, enum_type, library_with_classes, library_with_classes_and_enums, state_class,
    view_model_class,
};

#[test]
fn accepts_valid_view_model() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_with_classes(vec![
        state_class(),
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_missing_state() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_with_classes(vec![view_model_class(
        "TaskBoardViewModel",
        "(args: TaskBoardArgs)",
    )]));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.message.contains("requires `state: SomeState`") })
    );
}

#[test]
fn rejects_bad_generated_superclass() {
    let plugin = register_plugin();
    let mut view_model = view_model_class(
        "TaskBoardViewModel",
        "(state: TaskBoardState, args: TaskBoardArgs)",
    );
    view_model.superclass_name = Some("Object".to_owned());

    let diagnostics = plugin.validate(&library_with_classes(vec![
        state_class(),
        args_class(),
        view_model,
    ]));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("must extend `$TaskBoardViewModel`")
    }));
}

#[test]
fn rejects_args_that_do_not_extend_view_model_args() {
    let plugin = register_plugin();
    let mut args = args_class();
    args.superclass_name = Some("Object".to_owned());

    let diagnostics = plugin.validate(&library_with_classes(vec![
        state_class(),
        args,
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("must extend `ViewModelArgs`"))
    );
}

#[test]
fn rejects_enum_state_without_initial() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_with_classes_and_enums(
        vec![
            args_class(),
            view_model_class("ShellViewModel", "(state: ShellTab, args: TaskBoardArgs)"),
        ],
        vec![enum_type("ShellTab", &["dashboard", "tasks"])],
    ));

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("requires `initial:"))
    );
}

#[test]
fn accepts_enum_state_with_initial() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_with_classes_and_enums(
        vec![
            args_class(),
            view_model_class(
                "ShellViewModel",
                "(state: ShellTab, args: TaskBoardArgs, initial: ShellTab.dashboard)",
            ),
        ],
        vec![enum_type("ShellTab", &["dashboard", "tasks"])],
    ));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn rejects_state_without_default_constructor_or_initial() {
    let plugin = register_plugin();
    let mut state = state_class();
    state.fields = vec![FieldIr {
        name: "count".to_owned(),
        ty: TypeIr::named("int"),
        span: super::support::span(20, 30),
        has_default: false,
        serde: None,
        configs: Vec::new(),
    }];

    let diagnostics = plugin.validate(&library_with_classes(vec![
        state,
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("needs `initial:` because Dust cannot prove")
    }));
}

#[test]
fn rejects_state_constructor_with_required_params() {
    let plugin = register_plugin();
    let mut state = state_class();
    state.constructors = vec![ConstructorIr {
        name: None,
        is_factory: false,
        redirected_target_source: None,
        redirected_target_name: None,
        span: super::support::span(20, 30),
        params: vec![ConstructorParamIr {
            name: "count".to_owned(),
            ty: TypeIr::named("int"),
            span: super::support::span(31, 40),
            kind: ParamKind::Named,
            has_default: false,
            default_value_source: None,
        }],
    }];

    let diagnostics = plugin.validate(&library_with_classes(vec![
        state,
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("default constructor has required params")
    }));
}

#[test]
fn accepts_imported_state_and_args_when_library_has_imports() {
    let plugin = register_plugin();
    let mut library = library_with_classes(vec![view_model_class(
        "TaskBoardViewModel",
        "(state: ImportedState, args: ImportedArgs, initial: ImportedState.empty)",
    )]);
    library
        .imports
        .push("package:example/imported_state.dart".to_owned());

    let diagnostics = plugin.validate(&library);

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
