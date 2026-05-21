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
