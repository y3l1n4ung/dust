import 'dart:io';

import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Evolving a schema without a migration tool.
///
/// Migrations are a `Map` of name to SQL, applied in name order, each recorded
/// once. Numbered names are what make that order stable — `0002` before `0010`,
/// which `2` and `10` would not give you.
///
/// They are const strings compiled into the program, so the schema ships with
/// the code that expects it: there is no separate directory to forget to
/// deploy. SQLite applies them while opening, before any query can run.
///
/// ```shell
/// dart run example/migrations.dart
/// ```
Future<void> main() async {
  const migrations = <String, String>{
    '0001_create_users.sql': '''
CREATE TABLE users (
  id    INTEGER PRIMARY KEY,
  email TEXT NOT NULL UNIQUE
);
''',
    '0002_add_display_name.sql': '''
ALTER TABLE users ADD COLUMN display_name TEXT;
''',
  };

  final directory = Directory.systemTemp.createTempSync('dust_sqlite');
  final path = '${directory.path}/app.db';

  final first = Sqlite3Driver.open(path, migrations: migrations);
  await first.execute(
    r'INSERT INTO users (email, display_name) VALUES ($1, $2)',
    const <Object?>['ada@example.com', 'Ada'],
  );
  await first.close();

  // Re-opening with the same map applies nothing: `0002` would fail outright if
  // it ran twice, since the column already exists.
  final second = Sqlite3Driver.open(path, migrations: migrations);
  try {
    final applied = await second.fetchAll<String>(
      'SELECT name FROM __dust_schema_migrations ORDER BY name',
      const [],
      (row) => row.read<String>('name'),
    );
    print('applied: ${applied.unwrapOrElse((_) => const <String>[])}');
  } finally {
    await second.close();
    directory.deleteSync(recursive: true);
  }
}
