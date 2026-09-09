import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

import 'support/expect_ok.dart';
import 'support/user_name.dart';

SqlitePool _pool() {
  return SqlitePool.open(
    ':memory:',
    migrations: const <String, String>{
      '0001.sql': '''
CREATE TABLE users (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL
);
''',
    },
  );
}

void main() {
  test('unchecked SQL fetches, decodes, and executes', () async {
    final pool = _pool();
    addTearDown(() async {
      await pool.close();
    });
    final unsafe = Sqlite3UnsafeSql(pool);

    final inserted = expectOk(
      await unsafe.execute(
        r'INSERT INTO users (id, name) VALUES (?, ?)',
        [1, 'Ada'],
      ),
    );
    expect(inserted.rowsAffected, 1);

    final rows = expectOk(
      await unsafe.fetch('SELECT id, name FROM users', const <Object?>[]),
    );
    expect(rows.single.read<String>('name'), 'Ada');

    // The decoder is passed explicitly. Generated terminals exist only for
    // validated queries, and that asymmetry is the point.
    final users = expectOk(
      await unsafe.fetchAs<UserName>(
        'SELECT id, name FROM users',
        const <Object?>[],
        UserName.fromRow,
      ),
    );
    expect(users.single.name, 'Ada');
  });

  test('a failing mapper reports a decode error rather than throwing',
      () async {
    final pool = _pool();
    addTearDown(() async {
      await pool.close();
    });
    final unsafe = Sqlite3UnsafeSql(pool);
    expectOk(
      await unsafe.execute(
        r'INSERT INTO users (id, name) VALUES (?, ?)',
        [1, 'Ada'],
      ),
    );

    final result = await unsafe.fetchAs<UserName>(
      'SELECT id FROM users',
      const <Object?>[],
      UserName.fromRow,
    );

    expect(result.isErr, isTrue);
  });

  test('an executor is not a database facade, so it cannot reach unsafe SQL',
      () async {
    final pool = _pool();
    addTearDown(() async {
      await pool.close();
    });

    // `unsafe` lives on DatabaseClient, and a driver is not one. This is the
    // difference from the old `raw`, which sat on Executor: `db as Executor`
    // always succeeded because every pool, connection and transaction
    // implements it.
    expect(pool, isA<Executor>());
    expect(pool, isNot(isA<DatabaseClient>()));

    final escaped = await pool.transaction<bool>((tx) async {
      return Ok<bool, SqlxError>(tx is DatabaseClient);
    });
    expect(expectOk(escaped), isFalse);
  });
}
