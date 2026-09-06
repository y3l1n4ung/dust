import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Reading a column that can be NULL.
///
/// `read<T>` fails on NULL and `readNullable<T>` returns null for it. Which one
/// a mapper calls is the schema's nullability restated in Dart, so a
/// `NOT NULL` column read with `readNullable` costs you a needless `?`, and a
/// nullable one read with `read` fails on the first row that exercises it.
///
/// SQLite reports a `LEFT JOIN`'s unmatched side as nullable regardless of the
/// column's declaration, which is the case people usually meet first.
///
/// ```shell
/// dart run example/nullable_columns.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);
CREATE TABLE avatars (user_id INTEGER PRIMARY KEY, url TEXT NOT NULL);
''',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO users (id, name) VALUES ($1, $2), ($3, $4)',
      const <Object?>[1, 'Ada', 2, 'Grace'],
    );
    await db.execute(
      r'INSERT INTO avatars (user_id, url) VALUES ($1, $2)',
      const <Object?>[1, 'https://example.com/ada.png'],
    );

    // `avatars.url` is NOT NULL, but this join can still produce no row on that
    // side — so the read has to admit null even though the column does not.
    final rows = await db.fetchAll<String>(
      '''
SELECT users.name, avatars.url
FROM users
LEFT JOIN avatars ON avatars.user_id = users.id
ORDER BY users.id
''',
      const [],
      (row) {
        final url = row.readNullable<String>('url');
        return '${row.read<String>('name')}: ${url ?? 'no avatar'}';
      },
    );
    for (final line in rows.unwrapOrElse((_) => const <String>[])) {
      print(line);
    }
  } finally {
    await db.close();
  }
}
