import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'remote_todo.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RemoteTodo with _$RemoteTodo {
  const RemoteTodo({
    required this.userId,
    required this.id,
    required this.title,
    required this.completed,
    this.lane = 'Backlog',
    this.priority = 'Medium',
  });

  final int userId;
  final int id;
  final String title;
  final bool completed;
  final String lane;
  final String priority;

  factory RemoteTodo.fromJson(Map<String, Object?> json) {
    final todo = _$RemoteTodoFromJson(json);
    // If lane or priority are the default 'Backlog'/'Medium' but this was from JSON,
    // they might have been missing. We can keep them as is or enrich them.
    return todo;
  }
}
