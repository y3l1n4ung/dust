import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

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
/// dart run example/fetch_one.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': 'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);',
    },
  );

  try {
    await db.execute(
      r'INSERT INTO users (id, name) VALUES ($1, $2)',
      const <Object?>[1, 'Ada'],
    );

    final found = await db.fetchOne<String>(
      r'SELECT name FROM users WHERE id = $1',
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
      r'SELECT name FROM users WHERE id = $1',
      const <Object?>[404],
      (row) => row.read<String>('name'),
    );
    print('missing: ${missing.match(
      ok: (value) => value,
      err: (error) => error.category.name,
    )}');
  } finally {
    await db.close();
  }
}
