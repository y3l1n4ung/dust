import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Evolving a schema, safely, from more than one instance.
///
/// Migrations are a `Map` of name to SQL, applied in name order, each recorded
/// once in `__dust_schema_migrations`. Numbered names are what make that order
/// stable — `0002` before `0010`, which `2` and `10` would not give you.
///
/// Two differences from SQLite. They are applied by `migrate()` rather than
/// while opening, because PostgreSQL is reached over a network and connecting
/// cannot block on it. And they run under an advisory lock, so a deployment
/// that starts six replicas at once applies each migration exactly once instead
/// of six racing copies fighting over `CREATE TABLE`.
///
/// Calling `migrate()` is what lets one startup path serve both drivers: SQLite
/// has already done the work and returns `Ok`.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/migrations.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  const migrations = <String, String>{
    'example_0001_create_tenants.sql': '''
CREATE TABLE example_tenants (
  id   BIGSERIAL PRIMARY KEY,
  name TEXT NOT NULL
);
''',
    'example_0002_add_region.sql': '''
ALTER TABLE example_tenants ADD COLUMN region TEXT;
''',
  };

  final db = PostgresDriver.connect(url, migrations: migrations);
  try {
    print('applied: ${(await db.migrate()).isOk}');

    // Idempotent: a second call applies nothing. `0002` would fail outright if
    // it ran twice, since the column already exists.
    print('again: ${(await db.migrate()).isOk}');

    final columns = await db.fetchAll<String>(
      r'''
SELECT column_name FROM information_schema.columns
WHERE table_name = $1
ORDER BY ordinal_position
''',
      const <Object?>['example_tenants'],
      (row) => row.read<String>('column_name'),
    );
    print('columns: ${columns.unwrapOrElse((_) => const <String>[])}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_tenants', const []);
    // Only this example's own rows: the bookkeeping table is shared with
    // whatever else uses this database.
    await db.unsafe.execute(
      r'DELETE FROM __dust_schema_migrations WHERE name LIKE $1',
      const <Object?>['example_%'],
    );
    await db.close();
  }
}
