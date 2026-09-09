import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Writing, and what the database tells you afterwards.
///
/// `execute` returns an `ExecResult` holding `rowsAffected`. Its `lastInsertId`
/// is always null here: PostgreSQL has no counterpart to SQLite's
/// `last_insert_rowid()`, and inventing one over a pool would be a guess about
/// which connection ran what. Ask for the id with `RETURNING` instead — see
/// [returning.dart](returning.dart).
///
/// `rowsAffected` is worth checking on an update: zero means the `WHERE`
/// matched nothing, which is a different outcome from a failure and usually
/// reported differently.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/execute.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_tasks', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_tasks (id BIGSERIAL PRIMARY KEY, done BOOLEAN NOT NULL DEFAULT FALSE)',
      const [],
    );

    final inserted = await db.execute(
      'INSERT INTO example_tasks DEFAULT VALUES',
      const [],
    );
    final result = inserted.unwrapOrElse((error) => throw error);
    print('inserted rows: ${result.rowsAffected}');
    print('lastInsertId: ${result.lastInsertId}');

    final closed = await db.execute(
      'UPDATE example_tasks SET done = TRUE WHERE NOT done',
      const [],
    );
    print('updated: ${closed.unwrapOrElse((e) => throw e).rowsAffected}');

    // A WHERE that matches nothing is Ok with zero rows, not an error. Only the
    // caller knows whether that is fine.
    final absent = await db.execute(
      r'UPDATE example_tasks SET done = TRUE WHERE id = $1',
      const <Object?>[404],
    );
    print('no match: ${absent.unwrapOrElse((e) => throw e).rowsAffected}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_tasks', const []);
    await db.close();
  }
}
