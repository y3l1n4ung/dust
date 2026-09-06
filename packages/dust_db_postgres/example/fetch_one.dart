import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Reading a row that must exist.
///
/// `fetchOne` is for a query whose absence is a bug — a lookup by primary key
/// after an insert, a `count(*)`. It returns `Err` with a
/// [SqlxErrorCategory.cardinality] error when the row is not there, so the
/// caller never gets a null it forgot to consider.
///
/// When absence is ordinary, `fetchOptional` says so instead.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/fetch_one.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.unsafe.execute(
      'CREATE TABLE example_users (id BIGINT PRIMARY KEY, name TEXT NOT NULL)',
      const [],
    );
    await db.execute(
      r'INSERT INTO example_users (id, name) VALUES ($1, $2)',
      const <Object?>[1, 'Ada'],
    );

    final found = await db.fetchOne<String>(
      r'SELECT name FROM example_users WHERE id = $1',
      const <Object?>[1],
      (row) => row.read<String>('name'),
    );
    switch (found) {
      case Ok(:final value):
        print('found: $value');
      case Err(:final error):
        print('unreachable: $error');
    }

    final missing = await db.fetchOne<String>(
      r'SELECT name FROM example_users WHERE id = $1',
      const <Object?>[404],
      (row) => row.read<String>('name'),
    );
    print('missing: ${missing.match(
      ok: (value) => value,
      err: (error) => error.category.name,
    )}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_users', const []);
    await db.close();
  }
}
