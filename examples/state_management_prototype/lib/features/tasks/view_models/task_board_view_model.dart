import 'dart:async';
import 'package:collection/collection.dart';
import 'package:state_management_prototype/features/shell/view_models/shell_view_model.dart';
import 'package:state_management_prototype/features/tasks/models/task_board_state.dart';
import 'package:dust_state/dust_state.dart';
import 'package:state_management_prototype/shared/data/prototype_repository.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';

part 'task_board_view_model.g.dart';

final class TaskBoardViewModelArgs extends ViewModelArgs {
  const TaskBoardViewModelArgs({required this.repository, super.observer});

  final PrototypeRepository repository;
}

@ViewModel(state: TaskBoardState, args: TaskBoardViewModelArgs)
class TaskBoardViewModel extends $TaskBoardViewModel {
  TaskBoardViewModel(super.args);

  @override
  Future<void> onInit() async {
    await refresh(showLoading: true);
    emit(state.copyWith(isInitialized: true));
  }

  Future<void> refresh({bool showLoading = false}) async {
    emit(
      state.copyWith(
        isLoading: showLoading,
        isRefreshing: !showLoading,
        errorMessage: null,
      ),
    );

    try {
      final todos = await repository.fetchTodos(userId: 1, limit: 12);
      emit(
        state.copyWith(
          todos: todos,
          isLoading: false,
          isRefreshing: false,
          errorMessage: null,
        ),
      );
    } catch (_) {
      emit(
        state.copyWith(
          isLoading: false,
          isRefreshing: false,
          errorMessage: 'Unable to load task board right now.',
        ),
      );
    }
  }

  void setQuery(String query) {
    emit(state.copyWith(query: query));
  }

  void setFilter(TodoFilter filter) {
    emit(state.copyWith(filter: filter));
  }

  void toggleTodo(int todoId) {
    emit(
      state.copyWith(
        todos: [
          for (final todo in state.todos)
            todo.id == todoId
                ? todo.copyWith(completed: !todo.completed)
                : todo,
        ],
      ),
    );
  }

  void addTodo(String title, String lane, String priority) {
    final nextId = (state.todos.map((e) => e.id).maxOrNull ?? 0) + 1;
    final newTodo = RemoteTodo(
      id: nextId,
      userId: 1,
      title: title,
      completed: false,
      lane: lane,
      priority: priority,
    );
    emit(state.copyWith(todos: [newTodo, ...state.todos]));
  }

  void deleteTodo(int todoId) {
    emit(
      state.copyWith(
        todos: state.todos.where((todo) => todo.id != todoId).toList(),
      ),
    );
  }

  void clearCompleted() {
    emit(
      state.copyWith(
        todos: state.todos.where((todo) => !todo.completed).toList(),
      ),
    );
  }

  void spotlightTodo(RemoteTodo todo, ShellViewModel shellViewModel) {
    shellViewModel.selectTab(ShellTab.tasks);
    setQuery(todo.title);
  }
}
