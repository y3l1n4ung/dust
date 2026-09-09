import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Getting the written row back in one statement.
///
/// `RETURNING` makes a write a row query: the generated id, a `DEFAULT`, a
/// trigger's work, all read back in the same round trip and without a race
/// against another writer. On PostgreSQL it is not a convenience but the only
/// way to learn a generated id, since there is no `lastInsertId`.
///
/// Use a row terminal rather than `execute`, since the statement now returns
/// rows.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/returning.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_invoices', const []);
    await db.unsafe.execute(
      '''
CREATE TABLE example_invoices (
  id       BIGSERIAL PRIMARY KEY,
  amount   DOUBLE PRECISION NOT NULL,
  currency TEXT NOT NULL DEFAULT 'EUR'
)''',
      const [],
    );

    final created = await db.fetchOne<String>(
      r'''
INSERT INTO example_invoices (amount) VALUES ($1)
RETURNING id, currency
''',
      const <Object?>[99.5],
      (row) => '#${row.read<int>('id')} in ${row.read<String>('currency')}',
    );
    print('created: ${created.unwrapOrElse((error) => throw error)}');

    // Updates and deletes take it too, so a caller can act on exactly what was
    // changed rather than re-reading and hoping nothing moved in between.
    final deleted = await db.fetchAll<double>(
      'DELETE FROM example_invoices RETURNING amount',
      const [],
      (row) => row.read<double>('amount'),
    );
    print('deleted: ${deleted.unwrapOrElse((_) => const <double>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_invoices', const []);
    await db.close();
  }
}
