use std::sync::Arc;

use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_state_plugin::register_plugin;

use super::support::extract_class;
use crate::support::{args_class, library_with_classes, view_model_class};

#[test]
fn ignores_state_fields_from_workspace_analysis() {
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
    assert!(!source.contains("_TaskBoardViewModelAspect"));
    assert!(!source.contains("_taskBoardViewModelCountAspect"));
    assert!(!source.contains("_taskBoardViewModelMessageAspect"));
    assert!(!source.contains("get count => state.count"));
    assert!(!source.contains("get message => state.message"));
    assert!(!source.contains("get repository => args.repository"));
    assert!(!source.contains("get observer"));
    assert_eq!(
        extract_class(source, "class _$TaskBoardViewModelProxy"),
        r#"class _$TaskBoardViewModelProxy {
  _$TaskBoardViewModelProxy(this._context);

  final BuildContext _context;

  TaskBoardState get value {
    return TaskBoardViewModelScope.of(_context).value;
  }
}"#
    );
    assert_eq!(
        extract_class(source, "class _TaskBoardViewModelInherited"),
        r#"class _TaskBoardViewModelInherited extends InheritedWidget {
  const _TaskBoardViewModelInherited({required this.viewModel, required this.state, required super.child});

  final TaskBoardViewModel viewModel;
  final TaskBoardState state;

  @override
  bool updateShouldNotify(_TaskBoardViewModelInherited oldWidget) {
    return !identical(viewModel, oldWidget.viewModel) || state != oldWidget.state;
  }
}"#
    );
}

#[test]
fn ignores_many_state_fields_without_import_leaks() {
    let plugin = register_plugin();
    let fields = (0..120)
        .map(|index| format!(r#"{{"name":"field{index}","type_source":"int"}}"#))
        .collect::<Vec<_>>()
        .join(",");
    let mut builder = WorkspaceAnalysisBuilder::default();
    builder.add_string_set_value(
        "dust_state.states.v1",
        format!(r#"{{"class_name":"TaskBoardState","fields":[{fields}]}}"#),
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

    assert!(!source.contains("_taskBoardViewModelField119Aspect"));
    assert!(!source.contains("int get field119 {"));
    assert!(!source.contains("R select<R>"));
    assert!(!source.contains("int get field119 => state.field119;"));
    assert!(!source.contains("get repository => args.repository"));
}
