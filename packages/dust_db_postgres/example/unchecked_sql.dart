import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// The escape hatch, and why it is awkward.
///
/// `unsafe` runs SQL that build-time validation never sees: `EXPLAIN`, a
/// catalog query, a one-off administrative statement. Real cases, and rare
/// ones.
///
/// Three things about it are deliberate. It is named to sting, so it greps. Its
/// decoder is passed explicitly, so the checked path stays the ergonomic one.
/// And it is on the database, never on an [Executor] — a request handler holds
/// an executor and no cast takes it to the facade, so unchecked SQL is out of a
/// handler's reach by type rather than by convention.
///
/// Almost everything people reach for it has a checked form: an `IN` list is
/// one bound array, an optional filter is a `switch` over described queries,
/// and a sort column is a `switch` over an enum, since no dialect binds an
/// identifier.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/unchecked_sql.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    // DDL: no result shape to validate, and the reason `unsafe` exists at all.
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_users (id BIGSERIAL PRIMARY KEY, email TEXT NOT NULL)',
      const [],
    );
    await db.unsafe.execute(
      'CREATE INDEX example_users_email ON example_users (email)',
      const [],
    );

    // A query planner check — not a product query, and nothing to validate.
    final plan = await db.unsafe.fetchAs<String>(
      r'EXPLAIN SELECT id FROM example_users WHERE email = $1',
      const <Object?>['ada@example.com'],
      (row) => row.readIndex<String>(0),
    );
    // What the planner says depends on the server's version and statistics, so
    // print that it answered rather than what it chose.
    print(
        'plan returned: ${plan.unwrapOrElse((_) => const <String>[]).isNotEmpty}');

    // Untyped rows, when even the shape is not known ahead of time.
    final rows = await db.unsafe.fetch(
      r'''
SELECT column_name FROM information_schema.columns
WHERE table_name = $1
ORDER BY ordinal_position
''',
      const <Object?>['example_users'],
    );
    final columns = rows
        .unwrapOrElse((_) => const <Row>[])
        .map((row) => row.read<String>('column_name'))
        .toList();
    print('columns: $columns');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.close();
  }
}
