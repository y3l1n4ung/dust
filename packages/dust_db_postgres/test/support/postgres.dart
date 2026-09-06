import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

/// The database these tests run against, or null when there is none.
///
/// There is no in-memory PostgreSQL, so a suite cannot bring its own. Every
/// test that needs a server is skipped rather than failing when
/// `DUST_DATABASE_URL` is unset, and says so.
String? get databaseUrl => Platform.environment['DUST_DATABASE_URL'];

/// Reason a test is skipped, or null when it can run.
String? get skipWithoutDatabase =>
    databaseUrl == null ? 'set DUST_DATABASE_URL to run' : null;

/// Returns the `Ok` value, failing the test with the error otherwise.
T expectOk<T>(Result<T, SqlxError> result) =>
    result.unwrapOrElse((error) => fail('expected Ok, got $error'));

/// Returns the error, failing the test when the result succeeded.
SqlxError expectErr<T>(Result<T, SqlxError> result) => result.match(
      ok: (value) => fail('expected Err, got $value'),
      err: (error) => error,
    );

/// Opens a driver against the test database.
PostgresDriver connect({
  Map<String, String> migrations = const <String, String>{},
  PgConnectOptions? options,
}) {
  return PostgresDriver.connect(
    databaseUrl!,
    migrations: migrations,
    options: options,
  );
}

/// Drops [tables], leaving nothing behind.
///
/// A temporary table would belong to one pooled connection and be invisible to
/// the next statement, so these tests own real tables and remove them.
///
/// The migration bookkeeping is left alone unless [withMigrations] is set: test
/// files run concurrently against one database, and every file dropping the
/// shared table would race the file that is migrating.
Future<void> reset(
  PostgresDriver driver,
  List<String> tables, {
  bool withMigrations = false,
}) async {
  final all = <String>[
    ...tables,
    if (withMigrations) '__dust_schema_migrations',
  ];
  for (final table in all) {
    await driver.unsafe.execute('DROP TABLE IF EXISTS $table', const []);
  }
}

/// Creates a table for one test and drops it afterwards.
Future<PostgresDriver> tableFor(String ddl, String table) async {
  final driver = connect();
  await reset(driver, <String>[table]);
  expectOk(await driver.unsafe.execute(ddl, const []));
  addTearDown(() async {
    await reset(driver, <String>[table]);
    await driver.close();
  });
  return driver;
}
