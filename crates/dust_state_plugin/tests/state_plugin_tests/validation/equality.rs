use dust_diagnostics::Severity;
use dust_plugin_api::DustPlugin;
use dust_state_plugin::register_plugin;

use crate::support::{
    args_class, library_with_classes, operator_eq_method, state_class, state_class_with_eq,
    view_model_class,
};

#[test]
fn warns_when_sync_state_lacks_value_equality() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_with_classes(vec![
        state_class(),
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]));

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.severity == Severity::Warning
            && diagnostic
                .message
                .contains("state `TaskBoardState` should implement value equality")
    }));
}

#[test]
fn accepts_sync_state_with_eq_derive() {
    let plugin = register_plugin();
    let diagnostics = plugin.validate(&library_with_classes(vec![
        state_class_with_eq(),
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}

#[test]
fn accepts_sync_state_with_manual_equality_operator() {
    let plugin = register_plugin();
    let mut state = state_class();
    state.methods.push(operator_eq_method());

    let diagnostics = plugin.validate(&library_with_classes(vec![
        state,
        args_class(),
        view_model_class(
            "TaskBoardViewModel",
            "(state: TaskBoardState, args: TaskBoardArgs)",
        ),
    ]));

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
}
