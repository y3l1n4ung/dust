use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_state_plugin::register_plugin;

use super::support::{extract_doc_before, extract_extension};
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
    assert!(source.contains("this.identity"));
    assert!(source.contains("final Object? Function(BuildContext context)? identity;"));
    assert!(source.contains("class TaskBoardViewModelSelector<R> extends StatefulWidget"));
    assert!(source.contains("final R Function(TaskBoardState state) selector;"));
    assert!(source.contains("final bool Function(R previous, R next)? equals;"));
    assert!(
        source.contains("final nextViewModel = TaskBoardViewModelScope._watchInstance(context);")
    );
    assert!(source.contains("class _TaskBoardViewModelInstance extends InheritedWidget"));
    assert!(source.contains("scheduleMicrotask(() async {"));
    assert!(source.contains("await viewModel.init();"));
    assert!(source.contains("FlutterError.reportError("));
    assert!(source.contains("ErrorDescription('TaskBoardViewModelScope init failed')"));
    assert!(!source.contains("ViewModelOwner<"));
    assert!(!source.contains("ListenableBuilder("));
    assert!(!source.contains("if (ownsViewModel) {"));
    assert!(source.contains("class TaskBoardViewModelListener extends StatefulWidget"));
    assert_eq!(
        extract_doc_before(source, "abstract class $TaskBoardViewModel"),
        r#"/// Generated base class for TaskBoardViewModel.
///
/// Extend this class in the user-authored ViewModel and forward typed args:
///
/// ```dart
/// final class TaskBoardViewModel extends $TaskBoardViewModel {
///   TaskBoardViewModel(super.args);
/// }
/// ```"#
    );
    assert_eq!(
        extract_doc_before(source, "class _$TaskBoardViewModelProxy"),
        r#"/// Typed state reader returned by `context.watchTaskBoardViewModel()`.
///
/// Read `value` to rebuild for the whole state.
///
/// ```dart
/// final state = context.watchTaskBoardViewModel().value;
/// ```"#
    );
    assert_eq!(
        extract_doc_before(source, "class TaskBoardViewModelScope"),
        r#"/// Provides TaskBoardViewModel to descendants and owns it by default.
///
/// Use the default constructor when this scope should create and dispose the
/// ViewModel. Use `.value` only for externally owned ViewModels.
///
/// ```dart
/// TaskBoardViewModelScope(
///   args: (context) => TaskBoardArgs(...),
///   create: (context, args) => TaskBoardViewModel(args),
///   child: const FeaturePage(),
/// )
/// ```"#
    );
    assert_eq!(
        extract_doc_before(source, "const TaskBoardViewModelScope({"),
        r#"  /// Creates an owned TaskBoardViewModel from typed args."#
    );
    assert_eq!(
        extract_doc_before(source, "const TaskBoardViewModelScope.value({"),
        r#"  /// Provides an externally owned TaskBoardViewModel without disposing it."#
    );
    assert_eq!(
        extract_doc_before(source, "static TaskBoardViewModel read"),
        r#"  /// Reads TaskBoardViewModel without subscribing the caller to state changes."#
    );
    assert_eq!(
        extract_doc_before(source, "static TaskBoardViewModel of"),
        r#"  /// Watches TaskBoardViewModel and subscribes to state changes."#
    );
    assert_eq!(
        extract_doc_before(source, "class TaskBoardViewModelListener"),
        r#"/// Listens to one-shot effects from TaskBoardViewModel.
///
/// Effects are delivered without changing state and do not rebuild `child`.
///
/// ```dart
/// TaskBoardViewModelListener(
///   listener: onEffect,
///   child: const FeaturePage(),
/// )
/// ```"#
    );
    assert_eq!(
        extract_doc_before(source, "extension TaskBoardViewModelBuildContext"),
        r#"/// Generated BuildContext helpers for TaskBoardViewModel.
///
/// ```dart
/// final vm = context.readTaskBoardViewModel();
/// final state = context.watchTaskBoardViewModel().value;
/// ```"#
    );
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
    assert!(!source.contains("select<R>"));
    assert!(!source.contains("int get count"));
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
