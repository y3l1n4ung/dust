import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Reading a column that can be NULL.
///
/// `read<T>` fails on NULL and `readNullable<T>` returns null for it. Which one
/// a mapper calls is the schema's nullability restated in Dart, so a `NOT NULL`
/// column read with `readNullable` costs you a needless `?`, and a nullable one
/// read with `read` fails on the first row that exercises it.
///
/// PostgreSQL reports nullability at build time, so `dust db build` checks the
/// mapper against the query and says which of the two a column wants. The
/// `LEFT JOIN` below is the case it catches most often: `avatars.url` is
/// `NOT NULL`, yet the join can still produce no row on that side.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/nullable_columns.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_avatars', const []);
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_users (id BIGINT PRIMARY KEY, name TEXT NOT NULL)',
      const [],
    );
    await db.unsafe.execute(
      'CREATE TABLE example_avatars (user_id BIGINT PRIMARY KEY, url TEXT NOT NULL)',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_users (id, name) VALUES ($1, $2), ($3, $4)',
      const <Object?>[1, 'Ada', 2, 'Grace'],
    );
    await db.execute(
      r'INSERT INTO example_avatars (user_id, url) VALUES ($1, $2)',
      const <Object?>[1, 'https://example.com/ada.png'],
    );

    final rows = await db.fetchAll<String>(
      '''
SELECT u.name, a.url
FROM example_users u
LEFT JOIN example_avatars a ON a.user_id = u.id
ORDER BY u.id
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
    await db.unsafe.execute('DROP TABLE IF EXISTS example_avatars', const []);
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.close();
  }
}
