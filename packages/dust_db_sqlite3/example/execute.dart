import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Writing, and what the database tells you afterwards.
///
/// `execute` returns an `ExecResult`: `rowsAffected`, and `lastInsertId` when
/// the statement was an insert. `rowsAffected` is worth checking on an update —
/// zero means the `WHERE` matched nothing, which is a different outcome from a
/// failure and one an application usually has to report differently.
///
/// ```shell
/// dart run example/execute.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE tasks (
  id   INTEGER PRIMARY KEY,
  done INTEGER NOT NULL DEFAULT 0
);
''',
    },
  );

  try {
    final inserted = await db.execute(
      'INSERT INTO tasks DEFAULT VALUES',
      const [],
    );
    final id = inserted.unwrapOrElse((error) => throw error).lastInsertId;
    print('inserted id: $id');

    final closed = await db.execute(
      r'UPDATE tasks SET done = 1 WHERE id = $1',
      <Object?>[id],
    );
    print(
        'updated: ${closed.unwrapOrElse((error) => throw error).rowsAffected}');

    // A WHERE that matches nothing is Ok with zero rows, not an error. Only the
    // caller knows whether that is fine.
    final absent = await db.execute(
      r'UPDATE tasks SET done = 1 WHERE id = $1',
      const <Object?>[404],
    );
    print(
        'no match: ${absent.unwrapOrElse((error) => throw error).rowsAffected}');
  } finally {
    await db.close();
  }
}
