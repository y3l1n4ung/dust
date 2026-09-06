import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Reading every row a query selects.
///
/// `fetchAll` decodes the whole result into a `List`, so it wants a query with
/// a bound on how much it can return: an `ORDER BY` with a `LIMIT`, or a filter
/// narrow enough that the answer fits in memory. Every row also crosses a
/// socket here, so an unbounded `SELECT *` costs bandwidth as well as heap.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/fetch_all.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_events', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_events (id BIGSERIAL PRIMARY KEY, kind TEXT NOT NULL)',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_events (kind) VALUES ($1), ($2), ($3), ($4)',
      const <Object?>['created', 'updated', 'created', 'deleted'],
    );

    final page = await db.fetchAll<int>(
      r'SELECT id FROM example_events WHERE kind = $1 ORDER BY id DESC LIMIT $2',
      const <Object?>['created', 10],
      (row) => row.read<int>('id'),
    );
    print('ids: ${page.unwrapOrElse((_) => const <int>[])}');

    // No rows is an empty list, not an error.
    final none = await db.fetchAll<int>(
      r'SELECT id FROM example_events WHERE kind = $1',
      const <Object?>['archived'],
      (row) => row.read<int>('id'),
    );
    print('empty: ${none.unwrapOrElse((_) => const <int>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_events', const []);
    await db.close();
  }
}
