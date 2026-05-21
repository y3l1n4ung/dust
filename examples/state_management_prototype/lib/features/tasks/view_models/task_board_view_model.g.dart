part of 'task_board_view_model.dart';

enum _TaskBoardViewModelAspect {
  todos,
  isLoading,
  isRefreshing,
  isInitialized,
  errorMessage,
  query,
  filter,
}

abstract class $TaskBoardViewModel
    extends ViewModelBase<TaskBoardState, TaskBoardViewModelArgs> {
  $TaskBoardViewModel(super.args) : super(initialState: const TaskBoardState());
  List<RemoteTodo> get todos => state.todos;
  String get query => state.query;
  TodoFilter get filter => state.filter;
  bool get isLoading => state.isLoading;
  bool get isRefreshing => state.isRefreshing;
  bool get isInitialized => state.isInitialized;
  String? get errorMessage => state.errorMessage;
  PrototypeRepository get repository => args.repository;
}

class _$TaskBoardViewModelProxy {
  _$TaskBoardViewModelProxy(this._context, this._vm);
  final BuildContext _context;
  final TaskBoardViewModel _vm;
  TaskBoardState get value {
    TaskBoardViewModelScope.of(_context);
    return _vm.value;
  }

  List<RemoteTodo> get todos {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.todos,
    );
    return _vm.state.todos;
  }

  bool get isLoading {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.isLoading,
    );
    return _vm.state.isLoading;
  }

  bool get isRefreshing {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.isRefreshing,
    );
    return _vm.state.isRefreshing;
  }

  bool get isInitialized {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.isInitialized,
    );
    return _vm.state.isInitialized;
  }

  String? get errorMessage {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.errorMessage,
    );
    return _vm.state.errorMessage;
  }

  String get query {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.query,
    );
    return _vm.state.query;
  }

  TodoFilter get filter {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.filter,
    );
    return _vm.state.filter;
  }

  List<RemoteTodo> get visibleTodos {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.todos,
    );
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.query,
    );
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.filter,
    );
    return _vm.state.visibleTodos;
  }

  int get pendingCount {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.todos,
    );
    return _vm.state.pendingCount;
  }

  int get completedCount {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.todos,
    );
    return _vm.state.completedCount;
  }

  String get completionLabel {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.todos,
    );
    return _vm.state.completionLabel;
  }

  List<RemoteTodo> get spotlightTodos {
    TaskBoardViewModelScope.of(
      _context,
      aspect: _TaskBoardViewModelAspect.todos,
    );
    return _vm.state.spotlightTodos;
  }

  Future<void> refresh({bool showLoading = false}) =>
      _vm.refresh(showLoading: showLoading);
  void setQuery(String query) => _vm.setQuery(query);
  void setFilter(TodoFilter filter) => _vm.setFilter(filter);
  void toggleTodo(int todoId) => _vm.toggleTodo(todoId);
  void addTodo(String title, String lane, String priority) =>
      _vm.addTodo(title, lane, priority);
  void deleteTodo(int todoId) => _vm.deleteTodo(todoId);
  void clearCompleted() => _vm.clearCompleted();
  void spotlightTodo(RemoteTodo todo, ShellViewModel shellViewModel) =>
      _vm.spotlightTodo(todo, shellViewModel);
}

class TaskBoardViewModelScope extends StatefulWidget {
  const TaskBoardViewModelScope({
    super.key,
    required this.args,
    required this.create,
    required this.child,
  }) : value = null;
  const TaskBoardViewModelScope.value({
    super.key,
    required TaskBoardViewModel this.value,
    required this.child,
  }) : args = null,
       create = null;
  final TaskBoardViewModelArgs Function(BuildContext context)? args;
  final TaskBoardViewModel Function(
    BuildContext context,
    TaskBoardViewModelArgs args,
  )?
  create;
  final TaskBoardViewModel? value;
  final Widget child;
  static TaskBoardViewModel read(BuildContext context) {
    final scope =
        context
                .getElementForInheritedWidgetOfExactType<
                  _TaskBoardViewModelInherited
                >()
                ?.widget
            as _TaskBoardViewModelInherited?;
    if (scope == null)
      throw StateError('No TaskBoardViewModelScope found in context.');
    return scope.viewModel;
  }

  static TaskBoardViewModel of(BuildContext context, {Object? aspect}) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<_TaskBoardViewModelInherited>(
          aspect: aspect,
        );
    if (scope == null)
      throw StateError('No TaskBoardViewModelScope found in context.');
    return scope.viewModel;
  }

  @override
  State<TaskBoardViewModelScope> createState() =>
      _TaskBoardViewModelScopeState();
}

class _TaskBoardViewModelScopeState extends State<TaskBoardViewModelScope> {
  @override
  Widget build(BuildContext context) {
    final external = widget.value;
    return external == null
        ? ViewModelOwner<TaskBoardViewModel, TaskBoardViewModelArgs>(
            args: widget.args!,
            create: widget.create!,
            builder: _buildInherited,
          )
        : ViewModelOwner<TaskBoardViewModel, TaskBoardViewModelArgs>.value(
            value: external,
            builder: _buildInherited,
          );
  }

  Widget _buildInherited(BuildContext context, TaskBoardViewModel viewModel) {
    return ListenableBuilder(
      listenable: viewModel,
      builder: (context, child) => _TaskBoardViewModelInherited(
        viewModel: viewModel,
        state: viewModel.value,
        child: child!,
      ),
      child: widget.child,
    );
  }
}

class _TaskBoardViewModelInherited extends InheritedModel<Object> {
  const _TaskBoardViewModelInherited({
    required this.viewModel,
    required this.state,
    required super.child,
  });
  final TaskBoardViewModel viewModel;
  final TaskBoardState state;
  @override
  bool updateShouldNotify(_TaskBoardViewModelInherited oldWidget) =>
      state != oldWidget.state;
  @override
  bool updateShouldNotifyDependent(
    _TaskBoardViewModelInherited oldWidget,
    Set<Object> dependencies,
  ) {
    for (final aspect in dependencies) {
      if (aspect == _TaskBoardViewModelAspect.todos &&
          state.todos != oldWidget.state.todos)
        return true;
      if (aspect == _TaskBoardViewModelAspect.isLoading &&
          state.isLoading != oldWidget.state.isLoading)
        return true;
      if (aspect == _TaskBoardViewModelAspect.isRefreshing &&
          state.isRefreshing != oldWidget.state.isRefreshing)
        return true;
      if (aspect == _TaskBoardViewModelAspect.isInitialized &&
          state.isInitialized != oldWidget.state.isInitialized)
        return true;
      if (aspect == _TaskBoardViewModelAspect.errorMessage &&
          state.errorMessage != oldWidget.state.errorMessage)
        return true;
      if (aspect == _TaskBoardViewModelAspect.query &&
          state.query != oldWidget.state.query)
        return true;
      if (aspect == _TaskBoardViewModelAspect.filter &&
          state.filter != oldWidget.state.filter)
        return true;
    }
    return false;
  }
}

class TaskBoardViewModelListener extends StatefulWidget {
  const TaskBoardViewModelListener({
    super.key,
    required this.listener,
    required this.child,
  });
  final void Function(BuildContext context, Object effect) listener;
  final Widget child;
  @override
  State<TaskBoardViewModelListener> createState() =>
      _TaskBoardViewModelListenerState();
}

class _TaskBoardViewModelListenerState
    extends State<TaskBoardViewModelListener> {
  StreamSubscription<Object>? _sub;
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _sub?.cancel();
    _sub = TaskBoardViewModelScope.read(context).effects.listen((effect) {
      if (mounted) widget.listener(context, effect);
    });
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}

extension TaskBoardViewModelBuildContext on BuildContext {
  _$TaskBoardViewModelProxy watchTaskBoardViewModel() =>
      _$TaskBoardViewModelProxy(this, TaskBoardViewModelScope.read(this));
  TaskBoardViewModel readTaskBoardViewModel() =>
      TaskBoardViewModelScope.read(this);
}
