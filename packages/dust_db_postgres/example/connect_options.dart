import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Settings the URL cannot carry.
///
/// [PgConnectOptions] is a pass-through to the driver's own pool settings
/// rather than a second model of them. Explicit options win; the URL decides
/// what they leave unsaid, so a deployment can set `?sslmode=` once in its
/// connection string and the code need not know.
///
/// `applicationName` is worth setting on every service: it is what turns a row
/// in `pg_stat_activity` from an anonymous connection into a name you can page.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/connect_options.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(
    url,
    options: const PgConnectOptions(
      // How long to wait for the connection itself.
      connectTimeout: Duration(seconds: 5),
      // A ceiling on any one statement. Without it a lock wait is unbounded.
      queryTimeout: Duration(seconds: 30),
      // Shows up in pg_stat_activity and in the server log.
      applicationName: 'dust-example',
      // Left unset here, so the URL's ?sslmode= decides. Setting it would
      // override a URL that already says what it wants.
      // sslMode: PgSslMode.verifyFull,
    ),
  );

  try {
    final name = await db.fetchScalar<String>(
      r'SELECT current_setting($1)',
      const <Object?>['application_name'],
    );
    print('application_name: ${name.unwrapOrElse((error) => throw error)}');
  } finally {
    await db.close();
  }
}
