import '../models/create_todo.dart';
import '../models/todo.dart';
import 'store.dart';

/// A [TodoStore] holding everything in memory.
///
/// What a test wants, and what a first draft wants. `SqliteTodoStore` is the
/// same interface over a real database; handlers cannot tell them apart.
final class TodoRepository implements TodoStore {
  final _todos = <int, Todo>{};
  var _next = 1;

  /// Fills the repository so a fresh process has something to serve.
  void seed() {
    _add(
      const CreateTodo(title: 'write the example', assignTo: 'ada@dust.test'),
    );
  }

  @override
  Future<List<Todo>> all({String? owner, bool? done}) async => [
        for (final todo in _todos.values)
          if ((owner == null || todo.owner == owner) &&
              (done == null || todo.done == done))
            todo,
      ];

  @override
  Future<Todo?> find(int id) async => _todos[id];

  @override
  Future<Todo> add(CreateTodo input) async => _add(input);

  Todo _add(CreateTodo input) {
    final todo = Todo(
      id: _next++,
      title: input.title,
      owner: input.assignTo,
      done: input.done,
    );
    _todos[todo.id] = todo;
    return todo;
  }

  @override
  Future<Todo?> complete(int id, {required bool done}) async {
    final existing = _todos[id];
    if (existing == null) return null;
    return _todos[id] = existing.copyWith(done: done);
  }

  @override
  Future<bool> remove(int id) async => _todos.remove(id) != null;

  @override
  Future<void> close() async {}
}
