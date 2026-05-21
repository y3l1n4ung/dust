import 'package:derive_annotation/derive_annotation.dart';
import 'package:flutter/material.dart';
import 'package:state_management_prototype/shared/models/remote_todo.dart';

part 'task_board_state.g.dart';

enum TodoFilter { all, open, done }

@immutable
@Derive([ToString(), Eq(), CopyWith()])
class TaskBoardState with _$TaskBoardStateDust {
  const TaskBoardState({
    this.todos = const [],
    this.query = '',
    this.filter = TodoFilter.all,
    this.isLoading = false,
    this.isRefreshing = false,
    this.isInitialized = false,
    this.errorMessage,
  });

  final List<RemoteTodo> todos;
  final String query;
  final TodoFilter filter;
  final bool isLoading;
  final bool isRefreshing;
  final bool isInitialized;
  final String? errorMessage;

  int get completedCount => todos.where((todo) => todo.completed).length;
  int get pendingCount => todos.length - completedCount;
  String get completionLabel => todos.isEmpty
      ? '0%'
      : '${((completedCount / todos.length) * 100).round()}%';

  List<RemoteTodo> get visibleTodos {
    final normalizedQuery = query.trim().toLowerCase();
    return todos
        .where((todo) {
          final matchesFilter = switch (filter) {
            TodoFilter.all => true,
            TodoFilter.open => !todo.completed,
            TodoFilter.done => todo.completed,
          };
          final matchesQuery =
              normalizedQuery.isEmpty ||
              todo.title.toLowerCase().contains(normalizedQuery) ||
              todo.lane.toLowerCase().contains(normalizedQuery);
          return matchesFilter && matchesQuery;
        })
        .toList(growable: false);
  }

  List<RemoteTodo> get spotlightTodos =>
      todos.where((todo) => !todo.completed).take(3).toList(growable: false);
}
