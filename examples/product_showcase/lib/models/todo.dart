import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'todo.g.dart';

@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class Todo with _$TodoDust {
  final String id;
  final String title;
  final bool isCompleted;

  Todo({required this.id, required this.title, required this.isCompleted});

  factory Todo.fromJson(Map<String, Object?> json) => _$TodoFromJson(json);
}

@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class TodoCreate with _$TodoCreateDust {
  final String title;
  final bool isCompleted;

  TodoCreate({required this.title, required this.isCompleted});

  factory TodoCreate.fromJson(Map<String, Object?> json) =>
      _$TodoCreateFromJson(json);
}

@Derive([ToString(), CopyWith(), Serialize(), Deserialize()])
class TodoUpdate with _$TodoUpdateDust {
  final String? title;
  final bool? isCompleted;

  TodoUpdate({this.title, this.isCompleted});

  factory TodoUpdate.fromJson(Map<String, Object?> json) =>
      _$TodoUpdateFromJson(json);
}
