import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Reading a row that may not exist.
///
/// `fetchOptional` returns `Ok(null)` for no row and `Err` only for a real
/// failure, which keeps "not found" separate from "the query broke". A caller
/// that conflates the two logs a database outage as a 404.
///
/// ```shell
/// dart run example/fetch_optional.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': 'CREATE TABLE users (email TEXT PRIMARY KEY, name TEXT);',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO users (email, name) VALUES ($1, $2)',
      const <Object?>['ada@example.com', 'Ada'],
    );

    Future<String?> lookup(String email) async {
      final result = await db.fetchOptional<String>(
        r'SELECT name FROM users WHERE email = $1',
        <Object?>[email],
        (row) => row.read<String>('name'),
      );
      // An Err here is a database failure, not an absent user.
      return result.unwrapOrElse((error) => throw error);
    }

    print('present: ${await lookup('ada@example.com')}');
    print('absent: ${await lookup('nobody@example.com')}');
  } finally {
    await db.close();
  }
}
