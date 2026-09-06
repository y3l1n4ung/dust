import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Reading one value out of one column.
///
/// `fetchScalar` reads column zero and needs no mapper, which makes it the
/// right terminal for a `count(*)`, a `max(...)`, or an `EXISTS`. Anything
/// wider wants a row type.
///
/// ```shell
/// dart run example/fetch_scalar.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': 'CREATE TABLE orders (id INTEGER PRIMARY KEY, total REAL);',
    },
  );

  try {
    for (final total in const [12.5, 40.0, 7.25]) {
      await db.execute(
        r'INSERT INTO orders (total) VALUES ($1)',
        <Object?>[total],
      );
    }

    final count = await db.fetchScalar<int>(
      'SELECT count(*) FROM orders',
      const [],
    );
    print('orders: ${count.unwrapOrElse((_) => -1)}');

    final largest = await db.fetchScalar<double>(
      'SELECT max(total) FROM orders',
      const [],
    );
    print('largest: ${largest.unwrapOrElse((_) => 0)}');

    // An aggregate over no rows returns NULL, so the scalar type has to admit
    // it. `sum` behaves the same way; `count` does not.
    final unmatched = await db.fetchScalar<double?>(
      r'SELECT max(total) FROM orders WHERE total > $1',
      const <Object?>[1000],
    );
    print('over 1000: ${unmatched.unwrapOrElse((_) => 0)}');
  } finally {
    await db.close();
  }
}
