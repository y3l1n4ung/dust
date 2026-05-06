import 'package:product_showcase/product_showcase.dart';
import 'package:test/test.dart';

void main() {
  test('todo serde roundtrip stays intact', () {
    final todo = Todo(
      id: 'todo-1',
      title: 'Ship HttpClient support',
      isCompleted: false,
    );
    final decoded = Todo.fromJson(todo.toJson());

    expect(decoded.id, todo.id);
    expect(decoded.title, todo.title);
    expect(decoded.isCompleted, todo.isCompleted);
    expect(TodoCreate(title: 'Write docs', isCompleted: true).toJson(), {
      'title': 'Write docs',
      'isCompleted': true,
    });
    expect(
      TodoUpdate(
        title: 'Refine render reuse',
      ).copyWith(isCompleted: true).toJson(),
      {'title': 'Refine render reuse', 'isCompleted': true},
    );
  });
}
