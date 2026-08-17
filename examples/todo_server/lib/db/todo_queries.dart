import 'package:dust_dart/db.dart';

import '../models/todo.dart';

export '../models/todo.dart';

part 'todo_queries.g.dart';

/// Every query the API makes, and nothing else.
///
/// Bound to a [DatabaseExecutor], which is a connection *or* a transaction, so
/// the same queries run either way:
///
/// ```dart
/// final todos = TodoDao(database.connection);       // on its own
/// database.transaction((tx) => TodoDao(tx).setDone(1, id));  // atomically
/// ```
///
/// The result types are [Todo] itself, mapped by its `FromRow` derive. Adding
/// `@Sqlx(renameAll: ...)` to a type that also derives `Serialize` currently
/// makes the generator reject it here with "unsupported DAO result type"; the
/// columns in this schema need no renaming, so nothing is lost.
@SqlxDao()
abstract final class TodoDao {
  /// Binds the queries to [db], a connection or a transaction.
  const factory TodoDao(DatabaseExecutor db) = _$TodoDao;

  /// Every todo, newest last.
  @Query(r'SELECT id, title, owner, done FROM todos ORDER BY id')
  Future<Result<List<Todo>, SqlxError>> listAll();

  /// Every todo belonging to one owner.
  @Query(r'''
SELECT id, title, owner, done FROM todos
WHERE owner = $1
ORDER BY id
''')
  Future<Result<List<Todo>, SqlxError>> listByOwner(String owner);

  /// Every todo in one state.
  @Query(r'''
SELECT id, title, owner, done FROM todos
WHERE done = $1
ORDER BY id
''')
  Future<Result<List<Todo>, SqlxError>> listByDone(int done);

  /// Every todo belonging to one owner, in one state.
  @Query(r'''
SELECT id, title, owner, done FROM todos
WHERE owner = $1 AND done = $2
ORDER BY id
''')
  Future<Result<List<Todo>, SqlxError>> listByOwnerAndDone(
    String owner,
    int done,
  );

  /// One todo, or nothing.
  @Query(r'SELECT id, title, owner, done FROM todos WHERE id = $1')
  Future<Result<Todo?, SqlxError>> findById(int id);

  /// Stores a todo.
  @Query(r'INSERT INTO todos (title, owner, done) VALUES ($1, $2, $3)')
  Future<Result<ExecResult, SqlxError>> insertTodo(
    String title,
    String owner,
    int done,
  );

  /// Sets the done flag.
  @Query(r'UPDATE todos SET done = $1 WHERE id = $2')
  Future<Result<ExecResult, SqlxError>> setDone(int done, int id);

  /// Removes a todo.
  @Query(r'DELETE FROM todos WHERE id = $1')
  Future<Result<ExecResult, SqlxError>> deleteById(int id);
}
