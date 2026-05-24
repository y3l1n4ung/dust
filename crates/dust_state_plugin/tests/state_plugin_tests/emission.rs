use std::sync::Arc;

use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_state_plugin::register_plugin;

use super::support::{args_class, library_with_classes, state_class, view_model_class};

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
    assert!(source.contains("PrototypeRepository get repository => args.repository;"));
    assert!(source.contains("class TaskBoardViewModelScope extends StatefulWidget"));
    assert!(source.contains("debugName: 'TaskBoardViewModelScope'"));
    assert!(source.contains("debugName: 'TaskBoardViewModelScope.value'"));
    assert!(source.contains("class TaskBoardViewModelListener extends StatefulWidget"));
    assert_eq!(
        extract_extension(source, "extension TaskBoardViewModelBuildContext"),
        r#"extension TaskBoardViewModelBuildContext on BuildContext {
  TaskBoardViewModel get taskBoardViewModel => TaskBoardViewModelScope.of(this);

  _$TaskBoardViewModelProxy watchTaskBoardViewModel() {
    return _$TaskBoardViewModelProxy(this, TaskBoardViewModelScope.read(this));
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
fn emits_state_fields_from_workspace_analysis() {
    let plugin = register_plugin();
    let mut builder = WorkspaceAnalysisBuilder::default();
    builder.add_string_set_value(
        "dust_state.states.v1",
        r#"{"class_name":"TaskBoardState","fields":[{"name":"count","type_source":"int"},{"name":"message","type_source":"String?"}]}"#,
    );
    builder.add_string_set_value(
        "dust_state.states.v1",
        r#"{"class_name":"TaskBoardArgs","fields":[{"name":"repository","type_source":"PrototypeRepository"},{"name":"observer","type_source":"StateObserver?"}]}"#,
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(builder.build()));

    let contribution = plugin.emit(
        &library_with_classes(vec![
            args_class(),
            view_model_class(
                "TaskBoardViewModel",
                "(state: TaskBoardState, args: TaskBoardArgs)",
            ),
        ]),
        &plan,
    );

    let source = &contribution.support_types[0];
    assert!(source.contains("enum _TaskBoardViewModelAspect { count, message }"));
    assert!(source.contains("int get count => state.count;"));
    assert!(source.contains("String? get message => state.message;"));
    assert!(source.contains("PrototypeRepository get repository => args.repository;"));
    assert!(!source.contains("StateObserver? get observer"));
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
  TaskBoardViewModel get taskBoardViewModel => TaskBoardViewModelScope.of(this);

  _$TaskBoardViewModelProxy watchTaskBoardViewModel() {
    return _$TaskBoardViewModelProxy(this, TaskBoardViewModelScope.read(this));
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

fn extract_extension<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    &source[start..]
}
