import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:dust_db_sqlite3/src/placeholders.dart';
import 'package:test/test.dart';

import 'support/expect_ok.dart';

/// The branches that only run when something has gone wrong.

SqlitePool _pool() {
  return SqlitePool.open(
    ':memory:',
    migrations: const <String, String>{
      '0001.sql': 'CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT);',
    },
  );
}

void main() {
  test('an unterminated string literal runs to the end of the statement', () {
    // The database is what rejects the SQL; the scanner only has to stop
    // counting placeholders inside what looks like a literal.
    final rewrite = rewritePlaceholders(r"SELECT 'unterminated $1");

    expect(rewrite.parameterOrder, isEmpty);
    expect(rewrite.sql, r"SELECT 'unterminated $1");
  });

  test('a bind error reads as its message', () {
    const error = PlaceholderBindError('too few arguments', 'SELECT ?');

    expect(error.toString(), 'too few arguments');
    expect(error.sql, 'SELECT ?');
  });

  group('unchecked SQL', () {
    test('a mapper throwing a SqlxError passes it through', () async {
      final pool = _pool();
      addTearDown(() async {
        await pool.close();
      });
      expectOk(
        await queryExecute(
          r'INSERT INTO items (name) VALUES ($1)',
          ['first'],
        ).execute(pool),
      );

      final result = await Sqlite3UnsafeSql(pool).fetchAs<int>(
        'SELECT name FROM items',
        const <Object?>[],
        (row) => row.read<int>('name'),
      );

      expect(result.isErr, isTrue);
    });

    test('a mapper throwing anything else is still a decode error', () async {
      final pool = _pool();
      addTearDown(() async {
        await pool.close();
      });
      expectOk(
        await queryExecute(
          r'INSERT INTO items (name) VALUES ($1)',
          ['first'],
        ).execute(pool),
      );

      final result = await Sqlite3UnsafeSql(pool).fetchAs<int>(
        'SELECT name FROM items',
        const <Object?>[],
        (row) => throw StateError('mapper blew up'),
      );

      expect(
        result.match(ok: (_) => null, err: (error) => error.category),
        SqlxErrorCategory.decode,
      );
    });
  });

  group('a closed database', () {
    test('reports a query rather than throwing', () async {
      final pool = _pool();
      await pool.close();

      final rows = await queryRawFetch(pool);

      expect(rows.isErr, isTrue);
    });

    test('reports a statement rather than throwing', () async {
      final pool = _pool();
      await pool.close();

      final result = await queryExecute(
        'DELETE FROM items',
        const <Object?>[],
      ).execute(pool);

      expect(result.isErr, isTrue);
    });

    test('reports a transaction rather than throwing', () async {
      final pool = _pool();
      await pool.close();

      final result = await pool.transaction<Unit>(
        (tx) async => const Ok<Unit, SqlxError>(unit),
      );

      expect(result.isErr, isTrue);
    });
  });

  group('a handle closed underneath the driver', () {
    // Disposing the native database mid-transaction is what a control
    // statement failing looks like from here. It is hostile, but it is the only
    // way to reach the branches that report `COMMIT` and `ROLLBACK` failing,
    // and those exist so a failure there is a value rather than a throw.

    test('a failing commit is reported', () async {
      final pool = _pool();
      final result = await pool.transaction<Unit>((tx) async {
        (pool as Sqlite3Executor).database.close();
        return const Ok<Unit, SqlxError>(unit);
      });

      expect(result.isErr, isTrue);
    });

    test('a failing rollback is reported', () async {
      final pool = _pool();
      final result = await pool.transaction<Unit>((tx) async {
        (pool as Sqlite3Executor).database.close();
        return Err<Unit, SqlxError>(SqlxError.query('no', operation: 't'));
      });

      expect(result.isErr, isTrue);
    });

    test('a throw with a failing rollback is reported', () async {
      final pool = _pool();
      final result = await pool.transaction<Unit>((tx) async {
        (pool as Sqlite3Executor).database.close();
        throw StateError('boom');
      });

      expect(result.isErr, isTrue);
    });

    test('a failing savepoint release is reported', () async {
      final pool = _pool();
      final result = await pool.transaction<Unit>((tx) async {
        return tx.transaction<Unit>((nested) async {
          (pool as Sqlite3Executor).database.close();
          return const Ok<Unit, SqlxError>(unit);
        });
      });

      expect(result.isErr, isTrue);
    });

    test('a failing savepoint rollback is reported', () async {
      final pool = _pool();
      final result = await pool.transaction<Unit>((tx) async {
        return tx.transaction<Unit>((nested) async {
          (pool as Sqlite3Executor).database.close();
          return Err<Unit, SqlxError>(SqlxError.query('no', operation: 't'));
        });
      });

      expect(result.isErr, isTrue);
    });

    test('a nested throw with a failing rollback is reported', () async {
      final pool = _pool();
      final result = await pool.transaction<Unit>((tx) async {
        return tx.transaction<Unit>((nested) async {
          (pool as Sqlite3Executor).database.close();
          throw StateError('boom');
        });
      });

      expect(result.isErr, isTrue);
    });
  });

  group('a native handle disposed without closing', () {
    // `close()` sets a flag the driver checks, and that check raises a
    // `SqlxError` the narrow branch handles. Disposing the handle directly
    // leaves the flag alone, so the failure arrives as the driver's own
    // exception and takes the wider branch.

    test('a query is reported rather than throwing', () async {
      final pool = _pool();
      (pool as Sqlite3Executor).database.close();

      final rows = await Sqlite3UnsafeSql(pool).fetch(
        'SELECT id FROM items',
        const <Object?>[],
      );

      expect(rows.isErr, isTrue);
    });

    test('a statement is reported rather than throwing', () async {
      final pool = _pool();
      (pool as Sqlite3Executor).database.close();

      final result = await Sqlite3UnsafeSql(pool).execute(
        'DELETE FROM items',
        const <Object?>[],
      );

      expect(result.isErr, isTrue);
    });
  });

  group('migrations', () {
    test('a migration SQLite refuses is reported and rolled back', () {
      expect(
        () => SqlitePool.open(
          ':memory:',
          migrations: const <String, String>{
            '0001.sql': 'CREATE TABLE ok (id INTEGER);',
            '0002.sql': 'THIS IS NOT SQL;',
          },
        ),
        throwsA(isA<SqlxError>()),
      );
    });
  });
}

/// Runs an unchecked read, so the closed-database tests read the same way.
Future<Result<List<Row>, SqlxError>> queryRawFetch(SqlitePool pool) {
  return Sqlite3UnsafeSql(pool)
      .fetch('SELECT id FROM items', const <Object?>[]);
}
