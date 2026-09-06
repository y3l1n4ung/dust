import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// Applying migrations, and what happens when one cannot be applied.

const _first = '''
CREATE TABLE migrated_users (
  id    BIGSERIAL PRIMARY KEY,
  email TEXT NOT NULL
);
CREATE INDEX migrated_users_email ON migrated_users (email);
''';

const _second = 'ALTER TABLE migrated_users ADD COLUMN name TEXT;';

void main() {
  // No in-memory PostgreSQL, so a suite cannot bring its own database.
  // Reporting a skip is how the absence stays visible.
  if (databaseUrl == null) {
    test('migrations', () {}, skip: skipWithoutDatabase);
    return;
  }

  Future<PostgresDriver> driverWith(Map<String, String> migrations) async {
    final driver = connect(migrations: migrations);
    final names = migrations.keys.toList();
    await reset(driver, <String>['migrated_users'], migrations: names);
    addTearDown(() async {
      await reset(driver, <String>['migrated_users'], migrations: names);
      await driver.close();
    });
    return driver;
  }

  test('a migration with several statements is applied whole', () async {
    // The extended query protocol allows one command per prepared statement,
    // so a file holding two of them has to go through simple query mode.
    final driver = await driverWith(const <String, String>{'0001.sql': _first});

    expectOk(await driver.migrate());

    final columns = expectOk(
      await driver.unsafe.fetchAs<String>(
        r'SELECT column_name FROM information_schema.columns '
        r'WHERE table_name = $1 ORDER BY column_name',
        const <Object?>['migrated_users'],
        (row) => row.read<String>('column_name'),
      ),
    );
    expect(columns, <String>['email', 'id']);
  });

  test('migrations run in name order', () async {
    final driver = await driverWith(const <String, String>{
      '0002.sql': _second,
      '0001.sql': _first,
    });

    expectOk(await driver.migrate());

    final columns = expectOk(
      await driver.unsafe.fetchAs<String>(
        r'SELECT column_name FROM information_schema.columns '
        r'WHERE table_name = $1 ORDER BY column_name',
        const <Object?>['migrated_users'],
        (row) => row.read<String>('column_name'),
      ),
    );
    expect(columns, <String>['email', 'id', 'name']);
  });

  test('an applied migration is not applied twice', () async {
    final driver = await driverWith(const <String, String>{'0001.sql': _first});

    expectOk(await driver.migrate());
    // The second run would fail on `CREATE TABLE` if the bookkeeping were not
    // consulted, so this passing is the assertion.
    expectOk(await driver.migrate());

    final applied = expectOk(
      await driver.unsafe.fetchAs<String>(
        'SELECT name FROM __dust_schema_migrations ORDER BY name',
        const <Object?>[],
        (row) => row.read<String>('name'),
      ),
    );
    expect(applied, <String>['0001.sql']);
  });

  test('a down migration is never applied going forward', () async {
    final driver = await driverWith(const <String, String>{
      '0001.up.sql': _first,
      '0001.down.sql': 'DROP TABLE migrated_users;',
    });

    expectOk(await driver.migrate());

    final applied = expectOk(
      await driver.unsafe.fetchAs<String>(
        'SELECT name FROM __dust_schema_migrations',
        const <Object?>[],
        (row) => row.read<String>('name'),
      ),
    );
    expect(applied, <String>['0001.up.sql']);
  });

  test('no migrations is not an error', () async {
    final driver = await driverWith(const <String, String>{});

    expectOk(await driver.migrate());
  });

  test('a failing migration is named, and nothing is left behind', () async {
    final driver = await driverWith(const <String, String>{
      '0001.sql': _first,
      '0002_broken.sql': 'ALTER TABLE no_such_table ADD COLUMN x TEXT;',
    });

    final error = expectErr(await driver.migrate());

    expect(error.category, SqlxErrorCategory.migration);
    expect(error.message, contains('0002_broken.sql'));

    // The whole run is one transaction, so the first migration is rolled back
    // with the one that failed rather than leaving a half-migrated schema.
    final tables = expectOk(
      await driver.unsafe.fetchAs<String>(
        r'SELECT table_name FROM information_schema.tables '
        r'WHERE table_name = $1',
        const <Object?>['migrated_users'],
        (row) => row.read<String>('table_name'),
      ),
    );
    expect(tables, isEmpty);
  });

  test('a migration that removes the bookkeeping is reported', () async {
    // Contrived, but it is the one way recording an applied migration can fail,
    // and the failure has to name the migration rather than escape as a driver
    // error from a table nobody asked about.
    final driver = await driverWith(const <String, String>{
      '0001.sql': 'DROP TABLE IF EXISTS __dust_schema_migrations;',
    });

    final error = expectErr(await driver.migrate());

    expect(error.category, SqlxErrorCategory.migration);
    expect(error.message, contains('0001.sql'));
  });
}
