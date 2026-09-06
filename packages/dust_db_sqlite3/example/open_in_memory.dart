import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// The smallest database that works.
///
/// An in-memory database lives for as long as the driver holds it open and
/// costs nothing to create, which makes it the right default for a test, a
/// script, or reading an example like this one.
///
/// ```shell
/// dart run example/open_in_memory.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001_create_users.sql': 'CREATE TABLE users (name TEXT NOT NULL);',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO users (name) VALUES ($1)',
      const <Object?>['Ada'],
    );
    final count = await db.fetchScalar<int>(
      'SELECT count(*) FROM users',
      const [],
    );
    print('users: ${count.unwrapOrElse((_) => -1)}');
  } finally {
    // Closing frees the database. Nothing written here survives it.
    await db.close();
  }
}
