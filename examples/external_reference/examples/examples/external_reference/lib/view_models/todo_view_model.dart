import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

import '../models/todo.dart';

part 'todo_view_model.ease.dart';

/// Todo ViewModel - demonstrates list management
@ease
class TodoViewModel extends StateNotifier<List<Todo>> {
  TodoViewModel() : super([]);

  int _nextId = 0;

  void add(String title) {
    state = [
      ...state,
      Todo(id: '${_nextId++}', title: title),
    ];
  }

  void toggle(String id) {
    state = state.map((todo) {
      if (todo.id == id) {
        return todo.copyWith(completed: !todo.completed);
      }
      return todo;
    }).toList();
  }

  void remove(String id) {
    state = state.where((todo) => todo.id != id).toList();
  }

  void clear() => state = [];

  void clearCompleted() {
    state = state.where((todo) => !todo.completed).toList();
  }

  // Computed getters
  int get total => state.length;
  int get completedCount => state.where((t) => t.completed).length;
  int get pendingCount => state.where((t) => !t.completed).length;
}
