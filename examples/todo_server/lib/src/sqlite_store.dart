import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:dust_server/server.dart';

import '../models/create_todo.dart';
import '../db/database.dart';
import '../db/todo_queries.dart';
import 'store.dart';

/// A [TodoStore] over the generated [TodoDao].
///
/// Nothing here writes a row mapper, reads a column by name, or converts
/// between a row and a resource: `FromRow()` on [Todo] generates the mapper and
/// `@Query` generates the call. What is left is the part that is genuinely this
/// application's — deciding what a database failure means to a client.
final class SqliteTodoStore implements TodoStore {
  /// Wraps an open [database].
  SqliteTodoStore(this.database) : _todos = TodoDao(database.connection);

  /// Opens the database at [path], applying any unapplied migrations.
  ///
  /// The migration folder is named on `@SqlxDatabase`, so opening applies
  /// whatever is unapplied and records it. `:memory:` gives each test its own.
  factory SqliteTodoStore.open([
    String path = ':memory:',
    SqliteConnectOptions? options,
  ]) =>
      SqliteTodoStore(TodoDatabase.open(path, options: options));

  /// The settings a file-backed database wants when more than one process or
  /// isolate has it open.
  ///
  /// WAL lets readers run while a writer holds the file, which is the whole
  /// difference between a shared SQLite database that works and one that
  /// returns `SQLITE_BUSY`. The timeout gives a blocked writer somewhere to
  /// wait instead of failing immediately.
  static SqliteConnectOptions get sharedFile => const SqliteConnectOptions(
        journalMode: SqliteJournalMode.wal,
        busyTimeout: Duration(seconds: 5),
        foreignKeys: true,
      );

  /// The open database.
  final TodoDatabase database;

  final TodoDao _todos;

  @override
  Future<List<Todo>> all({String? owner, bool? done}) async {
    // Four queries rather than a concatenated `WHERE`: each one is checked
    // against the schema at build time, and none of them can be built out of
    // anything a client sent.
    final rows = switch ((owner, done)) {
      (final String owner, final bool done) =>
        await _todos.listByOwnerAndDone(owner, done ? 1 : 0),
      (final String owner, null) => await _todos.listByOwner(owner),
      (null, final bool done) => await _todos.listByDone(done ? 1 : 0),
      (null, null) => await _todos.listAll(),
    };

    return _unwrap(rows);
  }

  @override
  Future<Todo?> find(int id) async => _unwrap(await _todos.findById(id));

  @override
  Future<Todo> add(CreateTodo input) async {
    final inserted = _unwrap(
      await _todos.insertTodo(input.title, input.assignTo, input.done ? 1 : 0),
    );

    return Todo(
      id: inserted.lastInsertId!,
      title: input.title,
      owner: input.assignTo,
      done: input.done,
    );
  }

  @override
  Future<Todo?> complete(int id, {required bool done}) async {
    // An update followed by a read is two statements, and between them another
    // request could change the row. Running both against the same transaction
    // is what the DAO being separate from the database buys: `TodoDao(tx)` is
    // the same queries bound to a different executor.
    final outcome = await database.transaction<Todo?>((tx) async {
      final todos = TodoDao(tx);

      switch (await todos.setDone(done ? 1 : 0, id)) {
        case Err(:final error):
          return Err(error);
        case Ok(value: final updated):
          if (updated.rowsAffected == 0) return const Ok(null);
      }

      return switch (await todos.findById(id)) {
        Err(:final error) => Err(error),
        Ok(:final value) => Ok(value),
      };
    });

    return _unwrap(outcome);
  }

  @override
  Future<bool> remove(int id) async =>
      _unwrap(await _todos.deleteById(id)).rowsAffected > 0;

  @override
  Future<void> close() async {
    await database.close();
  }

  /// Takes the value, or turns the database failure into a response.
  ///
  /// A driver message names columns and constraints, so it goes to the error
  /// sink rather than to the client, and the client gets an opaque 500. The one
  /// failure worth distinguishing is a query that had to return a row and found
  /// none, which is a 404 rather than a fault.
  ///
  /// This mapping lives here rather than in the runtime because it is a product
  /// decision: another application would answer 409 for a constraint violation,
  /// or retry a busy database instead of failing.
  static T _unwrap<T>(Result<T, SqlxError> outcome) {
    switch (outcome) {
      case Ok(:final value):
        return value;
      case Err(:final error):
        if (error case SqlxCardinalityError(actual: 0)) {
          throw const Rejection.notFound('no such todo');
        }
        ServerErrors.report(error, StackTrace.current);
        throw const Rejection.internal();
    }
  }
}
