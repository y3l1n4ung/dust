import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// How many connections the pool opens, measured as distinct server backends.
///
/// `package:postgres` defaults to one connection, which runs every statement in
/// turn. Each query here sleeps long enough to overlap with the others, so the
/// number of backend process ids that answer is the number of connections the
/// pool really used.
void main() {
  if (databaseUrl == null) {
    test('pool size', () {}, skip: skipWithoutDatabase);
    return;
  }

  Future<Set<int>> backendsFor(PostgresDriver driver, int queries) async {
    final pids = await Future.wait([
      for (var i = 0; i < queries; i++)
        driver.fetchScalar<int>(
          'SELECT pg_backend_pid() FROM pg_sleep(0.3)',
          const [],
        ),
    ]);
    return {for (final pid in pids) expectOk(pid)};
  }

  test('maxConnections caps how many connections run at once', () async {
    final driver = connect(options: const PgConnectOptions(maxConnections: 3));
    addTearDown(driver.close);

    expect(await backendsFor(driver, 9), hasLength(3));
  });

  test('the default pool runs statements concurrently', () async {
    final driver = connect();
    addTearDown(driver.close);

    final backends = await backendsFor(
      driver,
      PgConnectOptions.defaultMaxConnections + 2,
    );
    expect(backends.length, greaterThan(1));
    expect(
      backends.length,
      lessThanOrEqualTo(PgConnectOptions.defaultMaxConnections),
    );
  });

  test('a pool needs at least one connection', () {
    expect(
      () => PostgresDriver.connect(
        'postgres://host/db',
        options: const PgConnectOptions(maxConnections: 0),
      ),
      throwsA(isA<ArgumentError>()),
    );
  });
}
