import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// Committing, reverting, and nesting.
///
/// `package:postgres` reverts a transaction when its callback throws, and Dust
/// reports a failed query as a value, so the two error models have to be made
/// to agree. Nesting has no API at all — a transaction session cannot open a
/// transaction — so it is savepoints, issued here.

const _ddl = '''
CREATE TABLE tx_notes (
  id   BIGSERIAL PRIMARY KEY,
  body TEXT NOT NULL
)''';

void main() {
  // No in-memory PostgreSQL, so a suite cannot bring its own database.
  // Reporting a skip is how the absence stays visible.
  if (databaseUrl == null) {
    test('transactions', () {}, skip: skipWithoutDatabase);
    return;
  }

  late PostgresDriver db;

  setUp(() async {
    db = await tableFor(_ddl, 'tx_notes');
  });

  /// Inserts one note through [executor].
  Future<Result<ExecResult, SqlxError>> write(Executor executor, String body) {
    return executor.execute(
      r'INSERT INTO tx_notes (body) VALUES ($1)',
      <Object?>[body],
    );
  }

  /// Every note in the table, oldest first.
  Future<List<String>> notes() async {
    return expectOk(
      await db.fetchAll<String>(
        'SELECT body FROM tx_notes ORDER BY id',
        const <Object?>[],
        (row) => row.read<String>('body'),
      ),
    );
  }

  test('Ok commits', () async {
    final result = await db.transaction<Unit>((tx) async {
      expectOk(await write(tx, 'kept'));
      return const Ok<Unit, SqlxError>(unit);
    });

    expect(result.isOk, isTrue);
    expect(await notes(), <String>['kept']);
  });

  test('Err reverts, and the Err is what comes back', () async {
    final result = await db.transaction<Unit>((tx) async {
      expectOk(await write(tx, 'dropped'));
      return Err<Unit, SqlxError>(SqlxError.query('no', operation: 'test'));
    });

    expect(expectErr(result).message, 'no');
    expect(await notes(), isEmpty);
  });

  test('a throw reverts and is reported as a transaction failure', () async {
    final result = await db.transaction<Unit>((tx) async {
      expectOk(await write(tx, 'dropped'));
      throw StateError('boom');
    });

    expect(expectErr(result).category, SqlxErrorCategory.transaction);
    expect(await notes(), isEmpty);
  });

  test('the transaction result is carried out unchanged', () async {
    final result = await db.transaction<int>((tx) async {
      expectOk(await write(tx, 'counted'));
      return const Ok<int, SqlxError>(7);
    });

    expect(expectOk(result), 7);
  });

  group('nested', () {
    test('an inner Err rolls back to its savepoint and the outer commits',
        () async {
      final result = await db.transaction<Unit>((tx) async {
        expectOk(await write(tx, 'outer'));

        final inner = await tx.transaction<Unit>((nested) async {
          expectOk(await write(nested, 'inner'));
          return Err<Unit, SqlxError>(SqlxError.query('undo', operation: 't'));
        });
        expect(inner.isErr, isTrue);

        return const Ok<Unit, SqlxError>(unit);
      });

      expect(result.isOk, isTrue, reason: 'the outer transaction survives');
      expect(await notes(), <String>['outer']);
    });

    test('an inner Ok releases its savepoint and both are kept', () async {
      final result = await db.transaction<Unit>((tx) async {
        expectOk(await write(tx, 'outer'));
        final inner = await tx.transaction<Unit>((nested) async {
          expectOk(await write(nested, 'inner'));
          return const Ok<Unit, SqlxError>(unit);
        });
        expect(inner.isOk, isTrue);
        return const Ok<Unit, SqlxError>(unit);
      });

      expect(result.isOk, isTrue);
      expect(await notes(), <String>['outer', 'inner']);
    });

    test('an inner throw rolls back to its savepoint', () async {
      final result = await db.transaction<Unit>((tx) async {
        expectOk(await write(tx, 'outer'));

        final inner = await tx.transaction<Unit>((nested) async {
          expectOk(await write(nested, 'inner'));
          throw StateError('boom');
        });
        expect(expectErr(inner).category, SqlxErrorCategory.transaction);

        return const Ok<Unit, SqlxError>(unit);
      });

      expect(result.isOk, isTrue);
      expect(await notes(), <String>['outer']);
    });

    test('savepoints nest more than one deep', () async {
      final result = await db.transaction<Unit>((tx) async {
        expectOk(await write(tx, 'first'));
        return tx.transaction<Unit>((second) async {
          expectOk(await write(second, 'second'));
          return second.transaction<Unit>((third) async {
            expectOk(await write(third, 'third'));
            return const Ok<Unit, SqlxError>(unit);
          });
        });
      });

      expect(result.isOk, isTrue);
      expect(await notes(), <String>['first', 'second', 'third']);
    });

    test('a savepoint inside an aborted transaction is reported', () async {
      // PostgreSQL aborts a transaction after a failed statement and refuses
      // everything until it ends — `SAVEPOINT` included. The nested call has
      // nowhere to go, and says so rather than throwing.
      final result = await db.transaction<Unit>((tx) async {
        expect(
          (await tx.execute('SELECT * FROM no_such_table', const [])).isErr,
          isTrue,
        );

        final nested = await tx.transaction<Unit>((inner) async {
          return const Ok<Unit, SqlxError>(unit);
        });
        expect(expectErr(nested).category, SqlxErrorCategory.transaction);

        return const Ok<Unit, SqlxError>(unit);
      });

      expect(result.isErr, isTrue,
          reason: 'the aborted transaction cannot commit');
    });
  });
}
