import 'package:dust_dart/serde.dart';

part 'todo.g.dart';

/// To-do item model for the product showcase example.
@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class Todo with _$Todo {
  /// Unique identifier.
  final String id;

  /// Title.
  final String title;

  /// Is completed.
  final bool isCompleted;

  /// Creates a [Todo].
  Todo({required this.id, required this.title, required this.isCompleted});

  /// Creates a [Todo] from JSON.
  factory Todo.fromJson(Map<String, Object?> json) => _$TodoFromJson(json);
}

/// To-do create request for the product showcase example.
@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class TodoCreate with _$TodoCreate {
  /// Title.
  final String title;

  /// Is completed.
  final bool isCompleted;

  /// Creates a [TodoCreate].
  TodoCreate({required this.title, required this.isCompleted});

  /// Creates a [TodoCreate] from JSON.
  factory TodoCreate.fromJson(Map<String, Object?> json) =>
      _$TodoCreateFromJson(json);
}

/// To-do update request for the product showcase example.
@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class TodoUpdate with _$TodoUpdate {
  /// Title.
  final String? title;

  /// Is completed.
  final bool? isCompleted;

  /// Creates a [TodoUpdate].
  TodoUpdate({this.title, this.isCompleted});

  /// Creates a [TodoUpdate] from JSON.
  factory TodoUpdate.fromJson(Map<String, Object?> json) =>
      _$TodoUpdateFromJson(json);
}
