import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Binding values, and why the numbers.
///
/// Dust writes parameters as `$1`, `$2` on both drivers. PostgreSQL takes that
/// natively; this driver rewrites it to SQLite's `?` before binding. One
/// spelling means a query validated once runs on either database, and a
/// `@SqlxDao` method reads the same wherever it is pointed.
///
/// Numbering also lets one value be used twice without being bound twice, which
/// `?` cannot express.
///
/// SQLite's own `?` still works if you have a reason to reach for it — the
/// rewrite leaves a statement with no `$n` alone.
///
/// ```shell
/// dart run example/placeholders.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE people (
  id    INTEGER PRIMARY KEY,
  first TEXT NOT NULL,
  last  TEXT NOT NULL
);
''',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO people (first, last) VALUES ($1, $2), ($3, $4)',
      const <Object?>['Ada', 'Lovelace', 'Grace', 'Grace'],
    );

    // $1 appears twice and is bound once.
    final matches = await db.fetchAll<String>(
      r'SELECT first FROM people WHERE first = $1 OR last = $1',
      const <Object?>['Grace'],
      (row) => row.read<String>('first'),
    );
    print('reused: ${matches.unwrapOrElse((_) => const <String>[])}');

    // A `$1` inside a literal or a comment is text, not a parameter, so this
    // statement binds nothing at all.
    final literal = await db.fetchScalar<String>(
      r"SELECT '$1 is not a placeholder here'",
      const [],
    );
    print('literal: ${literal.unwrapOrElse((_) => '')}');
  } finally {
    await db.close();
  }
}
