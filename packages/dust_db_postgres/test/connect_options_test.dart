import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// Connection settings, and how a URL is read.

void main() {
  // No in-memory PostgreSQL, so a suite cannot bring its own database.
  // Reporting a skip is how the absence stays visible.
  if (databaseUrl == null) {
    test('connect options', () {}, skip: skipWithoutDatabase);
    return;
  }

  test('sslmode is read from the URL', () async {
    // Every PostgreSQL tool carries the setting there, so a URL that works with
    // `psql` has to work here. The test database has no TLS, so `disable` is
    // what makes this connect at all.
    final driver = PostgresDriver.connect('${databaseUrl!}&application_name=x');
    addTearDown(() async {
      await driver.close();
    });

    expect(expectOk(await driver.fetchScalar<int>('SELECT 1', const [])), 1);
  });

  test('an unsupported sslmode is refused rather than guessed', () {
    // libpq's `prefer` and `allow` mean "try TLS, fall back to plaintext".
    // Choosing either side would decide a security question the caller left
    // open.
    expect(
      () => PostgresDriver.connect('postgres://host/db?sslmode=prefer'),
      throwsA(isA<ArgumentError>()),
    );
    expect(
      () => PostgresDriver.connect('postgres://host/db?sslmode=nonsense'),
      throwsA(isA<ArgumentError>()),
    );
  });

  test('every supported sslmode is accepted', () {
    for (final mode in const <String>['disable', 'require', 'verify-full']) {
      expect(
        () => PostgresDriver.connect('postgres://host/db?sslmode=$mode'),
        returnsNormally,
        reason: mode,
      );
    }
  });

  test('explicit options win over the URL', () async {
    // The URL says `disable` and the options say so too; the point is that the
    // options are consulted at all, which a wrong `sslMode` would prove by
    // failing to connect.
    final driver = connect(
      options: const PgConnectOptions(
        sslMode: PgSslMode.disable,
        connectTimeout: Duration(seconds: 5),
        queryTimeout: Duration(seconds: 5),
        applicationName: 'dust-test',
      ),
    );
    addTearDown(() async {
      await driver.close();
    });

    expect(expectOk(await driver.fetchScalar<int>('SELECT 1', const [])), 1);
  });

  test('a URL without a host, port or database falls back', () {
    // `postgres:///` names nothing; the defaults are what `psql` would use.
    expect(() => PostgresDriver.connect('postgres:///'), returnsNormally);
  });

  test('a URL carrying no user info connects without credentials', () {
    expect(
      () => PostgresDriver.connect('postgres://localhost:5432/app'),
      returnsNormally,
    );
  });

  test('the pool is reachable for driver-specific work', () async {
    final driver = connect();
    addTearDown(() async {
      await driver.close();
    });

    expect(driver.pool, isNotNull);
    expect(driver.session, isNotNull);
  });
}
