import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Reading a row that may not exist.
///
/// `fetchOptional` returns `Ok(null)` for no row and `Err` only for a real
/// failure, which keeps "not found" separate from "the query broke". Over a
/// network that distinction matters more than it does in-process: a caller that
/// conflates them reports a connection timeout as a 404.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/fetch_optional.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_users (email TEXT PRIMARY KEY, name TEXT NOT NULL)',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_users (email, name) VALUES ($1, $2)',
      const <Object?>['ada@example.com', 'Ada'],
    );

    Future<String?> lookup(String email) async {
      final result = await db.fetchOptional<String>(
        r'SELECT name FROM example_users WHERE email = $1',
        <Object?>[email],
        (row) => row.read<String>('name'),
      );
      // An Err here is a database failure, not an absent user.
      return result.unwrapOrElse((error) => throw error);
    }

    print('present: ${await lookup('ada@example.com')}');
    print('absent: ${await lookup('nobody@example.com')}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.close();
  }
}
