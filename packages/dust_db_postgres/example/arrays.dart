import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// A column that holds a list.
///
/// PostgreSQL has real array types, so a Dart `List` binds to one and reads
/// back as one — no join table, no comma-separated text to parse. SQLite has no
/// counterpart, which is why this file has no SQLite twin: there the same shape
/// is a child table or JSON.
///
/// Reach for it for a short, unordered set a row owns — tags, roles, feature
/// flags. Anything you would want to constrain, index by itself, or join
/// against is still a table.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/arrays.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_articles', const []);
    await db.unsafe.execute(
      '''
CREATE TABLE example_articles (
  id    BIGSERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  tags  TEXT[] NOT NULL DEFAULT '{}'
)''',
      const [],
    );

    await db.execute(
      r'INSERT INTO example_articles (title, tags) VALUES ($1, $2), ($3, $4)',
      const <Object?>[
        'Postgres arrays',
        <String>['sql', 'postgres'],
        'SQLite in practice',
        <String>['sql', 'sqlite'],
      ],
    );

    final tags = await db.fetchOne<List<String>>(
      r'SELECT tags FROM example_articles WHERE title = $1',
      const <Object?>['Postgres arrays'],
      (row) => row.read<List<String>>('tags'),
    );
    print('tags: ${tags.unwrapOrElse((error) => throw error)}');

    // `@>` asks whether the column contains every element of the bound array.
    // One placeholder, constant SQL, so it validates like any other query.
    final tagged = await db.fetchAll<String>(
      r'SELECT title FROM example_articles WHERE tags @> $1 ORDER BY id',
      const <Object?>[
        <String>['sqlite'],
      ],
      (row) => row.read<String>('title'),
    );
    print('tagged sqlite: ${tagged.unwrapOrElse((_) => const <String>[])}');

    // `unnest` goes the other way, turning the array into rows to group over.
    final counts = await db.fetchAll<String>(
      '''
SELECT tag, count(*) AS n
FROM example_articles, unnest(tags) AS tag
GROUP BY tag
ORDER BY n DESC, tag
''',
      const [],
      (row) => '${row.read<String>('tag')}=${row.read<int>('n')}',
    );
    print('counts: ${counts.unwrapOrElse((_) => const <String>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_articles', const []);
    await db.close();
  }
}
