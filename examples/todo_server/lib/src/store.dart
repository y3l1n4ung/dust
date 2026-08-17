import '../models/create_todo.dart';
import '../models/todo.dart';

/// What a handler needs from storage, whatever is underneath.
///
/// Every method is asynchronous because one implementation talks to a
/// database. Making the in-memory one async too keeps the handlers identical
/// across both, which is the point: swapping the store is a change at the
/// composition site and nowhere else.
abstract interface class TodoStore {
  /// Every todo, narrowed to [owner] when it is given.
  ///
  /// Passing `null` reads across owners, which only an admin handler does.
  Future<List<Todo>> all({String? owner, bool? done});

  /// The todo with [id], or `null`.
  Future<Todo?> find(int id);

  /// Stores a new todo, owned by whoever it was assigned to.
  Future<Todo> add(CreateTodo input);

  /// Replaces the done flag on [id], or returns `null` when it is missing.
  Future<Todo?> complete(int id, {required bool done});

  /// Removes [id], reporting whether it was there.
  Future<bool> remove(int id);

  /// Releases whatever the store holds.
  Future<void> close();
}
