import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Opening a pool.
///
/// [PostgresDriver.connect] takes the URL every PostgreSQL tool takes, returns
/// synchronously, and connects lazily — so a generated database facade can hold
/// one without its constructor becoming a future.
///
/// It is a pool, not a connection: statements may land on different backends,
/// which is why a `TEMP` table created by one statement is invisible to the
/// next. A transaction holds one connection for its whole closure.
///
/// `?sslmode=` in the URL is read, so a URL that works in `psql` works here.
/// Leaving it out means TLS is required, which is the right default and the
/// usual surprise against a local server.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/connect.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    final version = await db.fetchScalar<String>(
      r'SELECT current_setting($1)',
      const <Object?>['server_version'],
    );
    print('server: ${version.unwrapOrElse((error) => throw error)}');

    // Two statements, not necessarily the same backend.
    final backends = await db.fetchAll<int>(
      'SELECT pg_backend_pid()',
      const [],
      (row) => row.read<int>('pg_backend_pid'),
    );
    print('answered: ${backends.unwrapOrElse((_) => const <int>[]).length}');
  } finally {
    await db.close();
  }
}
