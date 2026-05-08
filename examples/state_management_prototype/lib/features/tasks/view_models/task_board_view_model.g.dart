part of 'task_board_view_model.dart';

class TaskBoardViewModelScope extends StatefulWidget {
  const TaskBoardViewModelScope({
    super.key,
    required this.create,
    required this.child,
  });

  final TaskBoardViewModel Function(BuildContext context) create;
  final Widget child;

  static TaskBoardViewModel of(BuildContext context) {
    final scope =
        context.dependOnInheritedWidgetOfExactType<_TaskBoardViewModelInherited>();
    if (scope == null) {
      throw StateError('No TaskBoardViewModelScope found in context.');
    }
    return scope.viewModel;
  }

  @override
  State<TaskBoardViewModelScope> createState() =>
      _TaskBoardViewModelScopeState();
}

class _TaskBoardViewModelScopeState extends State<TaskBoardViewModelScope> {
  TaskBoardViewModel? _viewModel;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _viewModel ??= widget.create(context);
  }

  @override
  void dispose() {
    _viewModel?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return _TaskBoardViewModelInherited(
      viewModel: _viewModel!,
      child: widget.child,
    );
  }
}

class _TaskBoardViewModelInherited extends InheritedNotifier<TaskBoardViewModel> {
  const _TaskBoardViewModelInherited({
    required this.viewModel,
    required super.child,
  }) : super(notifier: viewModel);

  final TaskBoardViewModel viewModel;
}

extension TaskBoardViewModelBuildContext on BuildContext {
  TaskBoardViewModel get taskBoardViewModel =>
      TaskBoardViewModelScope.of(this);
}
