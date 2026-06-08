import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:test/test.dart';

import 'support/user_name.dart';

void main() {
  test('sqlite transaction rolls back on error', () async {
    final pool = _userPool();
    addTearDown(() async {
      await pool.close();
    });

    final result = await pool.transaction((tx) async {
      await queryExecute(r'INSERT INTO users (id, name) VALUES (?, ?)', [
        1,
        'Ada',
      ]).execute(tx);
      throw StateError('boom');
    });
    expect(result, isA<Err<void, SqlxError>>());

    final rows = await queryRaw('SELECT id FROM users', []).fetch(pool);
    expect(rows, isEmpty);
  });

  test(
    'sqlite transaction exposes full driver API inside nested scope',
    () async {
      final pool = _userPool();
      addTearDown(() async {
        await pool.close();
      });

      final result = await pool.transaction((tx) async {
        expect(tx.driver, Driver.sqlite3);
        expect(
          (tx as Sqlite3Executor).database
              .select('SELECT 1')
              .single
              .columnAt(0),
          1,
        );
        await tx.execute(r'INSERT INTO users (id, name) VALUES (?, ?)', [
          1,
          'Ada',
        ]);
        final optional = await tx.fetchOptional<UserName>(
          r'SELECT id, name FROM users WHERE id = ?',
          [1],
          UserName.fromRow,
        );
        final one = await tx.fetchOne<UserName>(
          r'SELECT id, name FROM users WHERE id = ?',
          [1],
          UserName.fromRow,
        );
        final all = await tx.fetchAll<UserName>(
          r'SELECT id, name FROM users',
          const [],
          UserName.fromRow,
        );
        final count = await tx.fetchScalar<int>(
          'SELECT COUNT(*) FROM users',
          const [],
        );
        final rawRows = await tx.raw.fetch(
          'SELECT id, name FROM users',
          const [],
        );
        final nested = await tx.transaction((nestedTx) => nestedTx.close());

        expect(
          optional.match(ok: (value) => value?.name, err: (_) => 'err'),
          'Ada',
        );
        expect(one.match(ok: (value) => value.name, err: (_) => 'err'), 'Ada');
        expect(all.match(ok: (value) => value.length, err: (_) => -1), 1);
        expect(count.match(ok: (value) => value, err: (_) => -1), 1);
        expect(rawRows.match(ok: (rows) => rows.length, err: (_) => -1), 1);
        expect(nested, isA<Ok<Unit, SqlxError>>());
        return const Ok<String, SqlxError>('committed');
      });

      expect(
        result.match(ok: (value) => value, err: (_) => 'err'),
        'committed',
      );
    },
  );

  test('sqlite transaction rolls back when callback returns Err', () async {
    final pool = _userPool();
    addTearDown(() async {
      await pool.close();
    });

    final result = await pool.transaction<Unit>((tx) async {
      await queryExecute(r'INSERT INTO users (id, name) VALUES (?, ?)', [
        1,
        'Ada',
      ]).execute(tx);
      return Err<Unit, SqlxError>(SqlxError.driver('abort'));
    });
    expect(result, isA<Err<Unit, SqlxError>>());

    final rows = await queryRaw('SELECT id FROM users', []).fetch(pool);
    expect(rows, isEmpty);
  });

  test('sqlite raw channel reports driver errors without throwing', () async {
    final pool = SqlitePool.open(':memory:');
    addTearDown(() async {
      await pool.close();
    });

    final fetch = await pool.raw.fetch('SELECT * FROM missing_table', const []);
    final execute = await pool.raw.execute(
      'INSERT INTO missing_table VALUES (1)',
      const [],
    );

    expect(fetch, isA<Err<List<Row>, SqlxError>>());
    expect(execute, isA<Err<ExecResult, SqlxError>>());
  });
}

SqlitePool _userPool() {
  return SqlitePool.open(
    ':memory:',
    migrations: const <String, String>{
      '0001.sql':
          'CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL);',
    },
  );
}
