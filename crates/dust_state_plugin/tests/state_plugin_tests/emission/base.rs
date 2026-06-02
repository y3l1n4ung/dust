use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_state_plugin::register_plugin;

use super::support::extract_extension;
use crate::support::{args_class, library_with_classes, state_class, view_model_class};

#[test]
fn emits_generated_base_with_args_getters() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            state_class(),
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    assert_eq!(contribution.support_types.len(), 1);
    let source = &contribution.support_types[0];
    assert!(source.contains("abstract class $TaskBoardViewModel"));
    assert!(source.contains("extends ViewModelBase<TaskBoardState, TaskBoardArgs>"));
    assert!(source.contains(
        "$TaskBoardViewModel(super.args) : super(initialState: const TaskBoardState());"
    ));
    assert!(!source.contains("get repository => args.repository"));
    assert!(source.contains("class TaskBoardViewModelScope extends StatefulWidget"));
    assert!(source.contains("void didUpdateWidget(TaskBoardViewModelScope oldWidget)"));
    assert!(source.contains("scheduleMicrotask(() {"));
    assert!(!source.contains("ViewModelOwner<"));
    assert!(!source.contains("ListenableBuilder("));
    assert!(source.contains("class TaskBoardViewModelListener extends StatefulWidget"));
    assert_eq!(
        extract_extension(source, "extension TaskBoardViewModelBuildContext"),
        r#"extension TaskBoardViewModelBuildContext on BuildContext {
  _$TaskBoardViewModelProxy watchTaskBoardViewModel() {
    return _$TaskBoardViewModelProxy(this);
  }

  TaskBoardViewModel readTaskBoardViewModel() => TaskBoardViewModelScope.read(this);
}"#
    );
}

#[test]
fn emits_explicit_initial_expression() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            args_class(),
            view_model_class(
                "ShellViewModel",
                "(state: ShellTab, args: TaskBoardArgs, initial: ShellTab.dashboard)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    let source = &contribution.support_types[0];
    assert!(
        source.contains("$ShellViewModel(super.args) : super(initialState: ShellTab.dashboard);")
    );
}

#[test]
fn emits_value_only_proxy_for_fieldless_state() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            state_class(),
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    let source = &contribution.support_types[0];
    assert!(!source.contains("enum _TaskBoardViewModelAspect"));
    assert!(source.contains("TaskBoardState get value"));
    assert_eq!(
        extract_extension(source, "extension TaskBoardViewModelBuildContext"),
        r#"extension TaskBoardViewModelBuildContext on BuildContext {
  _$TaskBoardViewModelProxy watchTaskBoardViewModel() {
    return _$TaskBoardViewModelProxy(this);
  }

  TaskBoardViewModel readTaskBoardViewModel() => TaskBoardViewModelScope.read(this);
}"#
    );
}

#[test]
fn emits_single_output_per_annotated_view_model() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library_with_classes(vec![
            state_class(),
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
            view_model_class(
                "SecondaryViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &SymbolPlan::default(),
    );

    assert_eq!(contribution.support_types.len(), 2);
    assert!(contribution.support_types[0].contains("$TaskBoardViewModel"));
    assert!(contribution.support_types[1].contains("$SecondaryViewModel"));
}
