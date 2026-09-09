import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Binding values, and why the numbers.
///
/// `$1`, `$2` is PostgreSQL's own syntax, and Dust writes parameters that way
/// on both drivers — the SQLite driver rewrites it to `?` at bind time. One
/// spelling means a query validated once runs on either database, and a
/// `@SqlxDao` method reads the same wherever it is pointed.
///
/// Numbering also lets one value be used twice while being bound once, which
/// `?` cannot express.
///
/// In Dart, write these as raw strings — `r'... $1'` — or `$1` becomes string
/// interpolation and the compiler rejects it. That error is the good case; the
/// bad one is a `$name` that happens to be in scope.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/placeholders.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_people', const []);
    await db.unsafe.execute(
      '''
CREATE TABLE example_people (
  id    BIGSERIAL PRIMARY KEY,
  first TEXT NOT NULL,
  last  TEXT NOT NULL
)''',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_people (first, last) VALUES ($1, $2), ($3, $4)',
      const <Object?>['Ada', 'Lovelace', 'Grace', 'Grace'],
    );

    // $1 appears twice and is bound once.
    final matches = await db.fetchAll<String>(
      r'SELECT first FROM example_people WHERE first = $1 OR last = $1',
      const <Object?>['Grace'],
      (row) => row.read<String>('first'),
    );
    print('reused: ${matches.unwrapOrElse((_) => const <String>[])}');

    // A parameter is a value, never an identifier: no dialect binds a table or
    // column name. A sort column is a `switch` over an enum, so the SQL stays
    // constant and stays checkable.
    final sorted = await db.fetchAll<String>(
      'SELECT first FROM example_people ORDER BY last DESC',
      const [],
      (row) => row.read<String>('first'),
    );
    print('sorted: ${sorted.unwrapOrElse((_) => const <String>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_people', const []);
    await db.close();
  }
}
