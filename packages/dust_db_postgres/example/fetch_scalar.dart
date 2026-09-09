import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Reading one value out of one column.
///
/// `fetchScalar` reads column zero and needs no mapper, which makes it the
/// right terminal for a `count(*)`, a `max(...)`, or an `EXISTS`. Anything
/// wider wants a row type.
///
/// `count(*)` is `bigint` in PostgreSQL, and a Dart `int` holds it.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/fetch_scalar.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_orders', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_orders (id BIGSERIAL PRIMARY KEY, total DOUBLE PRECISION NOT NULL)',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_orders (total) VALUES ($1), ($2), ($3)',
      const <Object?>[12.5, 40.0, 7.25],
    );

    final count = await db.fetchScalar<int>(
      'SELECT count(*) FROM example_orders',
      const [],
    );
    print('orders: ${count.unwrapOrElse((_) => -1)}');

    final largest = await db.fetchScalar<double>(
      'SELECT max(total) FROM example_orders',
      const [],
    );
    print('largest: ${largest.unwrapOrElse((_) => 0)}');

    // An aggregate over no rows returns NULL, so the scalar type has to admit
    // it. `sum` and `max` behave this way; `count` does not.
    final unmatched = await db.fetchScalar<double?>(
      r'SELECT max(total) FROM example_orders WHERE total > $1',
      const <Object?>[1000.0],
    );
    print('over 1000: ${unmatched.unwrapOrElse((_) => 0)}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_orders', const []);
    await db.close();
  }
}
