import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Getting the written row back in one statement.
///
/// `RETURNING` (SQLite 3.35 and later) makes a write a row query: the generated
/// id, a `DEFAULT`, a trigger's work, all read back without a second round trip
/// and without a race against another writer. Use a row terminal rather than
/// `execute`, since the statement now returns rows.
///
/// It is also the portable spelling — PostgreSQL has had `RETURNING` for
/// years, while `lastInsertId` is SQLite's own.
///
/// ```shell
/// dart run example/returning.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE invoices (
  id       INTEGER PRIMARY KEY,
  amount   REAL NOT NULL,
  currency TEXT NOT NULL DEFAULT 'EUR'
);
''',
    },
  );

  try {
    final created = await db.fetchOne<String>(
      r'''
INSERT INTO invoices (amount) VALUES ($1)
RETURNING id, currency
''',
      const <Object?>[99.5],
      (row) => '#${row.read<int>('id')} in ${row.read<String>('currency')}',
    );
    print('created: ${created.unwrapOrElse((error) => throw error)}');

    // Updates and deletes take it too, so a caller can act on exactly what was
    // changed rather than re-reading and hoping nothing moved in between.
    final deleted = await db.fetchAll<double>(
      'DELETE FROM invoices RETURNING amount',
      const [],
      (row) => row.read<double>('amount'),
    );
    print('deleted: ${deleted.unwrapOrElse((_) => const <double>[])}');
  } finally {
    await db.close();
  }
}
