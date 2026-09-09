import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// An `IN` list without dynamic SQL.
///
/// SQLite has no array type, so a bound `List` arrives as JSON text and
/// `json_each` unpacks it. That keeps the statement constant — one placeholder,
/// whatever the list length — which is what lets `dust db build` validate it,
/// unlike an `IN ($1, $2, $3)` assembled at the call site. The PostgreSQL
/// spelling is `= ANY($1)`; the bound list is the part that does not change.
///
/// ```shell
/// dart run example/set_membership.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001_create_items.sql': '''
CREATE TABLE items (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
''',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO items (name) VALUES ($1), ($2), ($3)',
      const <Object?>['first', 'second', 'third'],
    );

    final some = await db.fetchAll<String>(
      r'''
SELECT name FROM items
WHERE id IN (SELECT value FROM json_each($1))
ORDER BY id
''',
      <Object?>[
        <int>[1, 3],
      ],
      (row) => row.read<String>('name'),
    );
    print('selected: ${some.unwrapOrElse((_) => const <String>[])}');

    // An empty list selects nothing rather than failing, which `IN ()` cannot
    // even express.
    final none = await db.fetchAll<String>(
      r'''
SELECT name FROM items
WHERE id IN (SELECT value FROM json_each($1))
''',
      <Object?>[const <int>[]],
      (row) => row.read<String>('name'),
    );
    print('empty: ${none.unwrapOrElse((_) => const <String>[])}');
  } finally {
    await db.close();
  }
}
