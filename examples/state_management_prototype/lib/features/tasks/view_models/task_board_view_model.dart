import 'package:flutter/material.dart';
import 'package:state_management_prototype/features/shell/view_models/shell_view_model.dart';
import 'package:state_management_prototype/features/tasks/models/task_board_state.dart';
import 'package:state_management_prototype/shared/annotations.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';

part 'task_board_view_model.g.dart';

@ViewModel()
class TaskBoardViewModel extends ValueNotifier<TaskBoardState> {
  TaskBoardViewModel(this._repository) : super(const TaskBoardState());

  final PrototypeRepository _repository;
  bool _initializing = false;

  TaskBoardState get state => value;
  set state(TaskBoardState nextState) => value = nextState;

  Future<void> initialize() async {
    if (_initializing || state.todos.isNotEmpty) {
      return;
    }
    _initializing = true;
    try {
      await refresh(showLoading: true);
    } finally {
      _initializing = false;
    }
  }

  Future<void> refresh({bool showLoading = false}) async {
    state = state.copyWith(
      isLoading: showLoading,
      isRefreshing: !showLoading,
      errorMessage: null,
    );

    try {
      final todos = await _repository.fetchTodos(userId: 1, limit: 12);
      state = state.copyWith(
        todos: todos,
        isLoading: false,
        isRefreshing: false,
        errorMessage: null,
      );
    } catch (_) {
      state = state.copyWith(
        isLoading: false,
        isRefreshing: false,
        errorMessage: 'Unable to load task board right now.',
      );
    }
  }

  void setQuery(String query) {
    state = state.copyWith(query: query);
  }

  void setFilter(TodoFilter filter) {
    state = state.copyWith(filter: filter);
  }

  void toggleTodo(int todoId) {
    state = state.copyWith(
      todos: [
        for (final todo in state.todos)
          todo.id == todoId
              ? todo.copyWith(completed: !todo.completed)
              : todo,
      ],
    );
  }

  void clearCompleted() {
    state = state.copyWith(
      todos: state.todos.where((todo) => !todo.completed).toList(),
    );
  }

  void spotlightTodo(RemoteTodo todo, ShellViewModel shellViewModel) {
    shellViewModel.selectTab(ShellTab.tasks);
    setQuery(todo.title);
  }
}
