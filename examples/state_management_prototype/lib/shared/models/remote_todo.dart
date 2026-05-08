import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'remote_todo.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class RemoteTodo with _$RemoteTodoDust {
  const RemoteTodo({
    required this.userId,
    required this.id,
    required this.title,
    required this.completed,
  });

  final int userId;
  final int id;
  final String title;
  final bool completed;

  factory RemoteTodo.fromJson(Map<String, Object?> json) =>
      _$RemoteTodoFromJson(json);

  String get lane => switch (id % 3) {
        0 => 'Ops',
        1 => 'Design',
        _ => 'Platform',
      };

  String get priority => switch (id % 3) {
        0 => 'High',
        1 => 'Medium',
        _ => 'Low',
      };
}
