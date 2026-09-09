import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Reading every row a query selects.
///
/// `fetchAll` decodes the whole result into a `List`, so it wants a query with
/// a bound on how much it can return: an `ORDER BY` with a `LIMIT`, or a
/// filter narrow enough that the answer fits in memory. `SELECT *` over a table
/// that grows is how a service falls over at 3am.
///
/// ```shell
/// dart run example/fetch_all.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': 'CREATE TABLE events (id INTEGER PRIMARY KEY, kind TEXT);',
    },
  );

  try {
    for (final kind in const ['created', 'updated', 'created', 'deleted']) {
      await db.execute(
        r'INSERT INTO events (kind) VALUES ($1)',
        <Object?>[kind],
      );
    }

    final page = await db.fetchAll<int>(
      r'SELECT id FROM events WHERE kind = $1 ORDER BY id DESC LIMIT $2',
      const <Object?>['created', 10],
      (row) => row.read<int>('id'),
    );
    print('ids: ${page.unwrapOrElse((_) => const <int>[])}');

    // No rows is an empty list, not an error.
    final none = await db.fetchAll<int>(
      r'SELECT id FROM events WHERE kind = $1',
      const <Object?>['archived'],
      (row) => row.read<int>('id'),
    );
    print('empty: ${none.unwrapOrElse((_) => const <int>[])}');
  } finally {
    await db.close();
  }
}
