@Tags(<String>['postgres'])
library;

import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

/// Everything that needs a live server.
///
/// Skipped unless `DUST_DATABASE_URL` points at a PostgreSQL database this run
/// may write to. There is no in-memory Postgres, so this is the only way these
/// can run at all — the same reason `dust db build` needs a server and CI
/// validates from the committed query cache instead.
///
/// The database is left as it was found: every test works in a temporary table,
/// which the server drops when the session ends.
String? get _url => Platform.environment['DUST_DATABASE_URL'];

void main() {
  final url = _url;
  if (url == null) {
    // A skipped group still reports, so the absence is visible rather than
    // looking like a suite that passed.
    test('postgres integration', () {}, skip: 'set DUST_DATABASE_URL to run');
    return;
  }

  late PgPool pool;

  setUp(() async {
    pool = PgPool.connect(url);
    expectOk(await pool.execute(
      'CREATE TEMP TABLE orders ('
      'id BIGSERIAL PRIMARY KEY, '
      'item TEXT NOT NULL, '
      'quantity INTEGER NOT NULL, '
      'placed_at TIMESTAMPTZ NOT NULL DEFAULT now())',
      const <Object?>[],
    ));
  });

  tearDown(() async {
    await pool.close();
  });

  test(r'binds $n natively with a plain argument list', () async {
    expectOk(await pool.execute(
      r'INSERT INTO orders (item, quantity) VALUES ($1, $2)',
      <Object?>['shirt', 2],
    ));

    final quantity = await pool.fetchScalar<int>(
      r'SELECT quantity FROM orders WHERE item = $1',
      <Object?>['shirt'],
    );

    expect(quantity.unwrapOrElse((error) => fail('$error')), 2);
  });

  test('a repeated placeholder binds one argument once', () async {
    // The difference from SQLite, which expands a repeated `$1` into two binds.
    // Postgres reads the statement itself, so the argument list stays as
    // written.
    expectOk(await pool.execute(
      r'INSERT INTO orders (item, quantity) VALUES ($1, 1), ($1, 2)',
      <Object?>['socks'],
    ));

    final count = await pool.fetchScalar<int>(
      r'SELECT count(*) FROM orders WHERE item = $1',
      <Object?>['socks'],
    );

    expect(count.unwrapOrElse((error) => fail('$error')), 2);
  });

  test(r'a List binds as an array, so ANY($1) needs no encoding', () async {
    for (final item in const <String>['a', 'b', 'c']) {
      expectOk(await pool.execute(
        r'INSERT INTO orders (item, quantity) VALUES ($1, 1)',
        <Object?>[item],
      ));
    }

    final items = await pool.fetchAll<String>(
      r'SELECT item FROM orders WHERE item = ANY($1) ORDER BY item',
      <Object?>[
        <String>['a', 'c'],
      ],
      (row) => row.read<String>('item'),
    );

    expect(
      items.unwrapOrElse((error) => fail('$error')),
      <String>['a', 'c'],
    );
  });

  test('RETURNING stands in for lastInsertId', () async {
    // Postgres has no `last_insert_rowid()`, so a caller that needs the new row
    // asks for it and reads it like any other query.
    final inserted = await pool.fetchOne<int>(
      r'INSERT INTO orders (item, quantity) VALUES ($1, $2) RETURNING id',
      <Object?>['hat', 1],
      (row) => row.read<int>('id'),
    );

    expect(inserted.unwrapOrElse((error) => fail('$error')), greaterThan(0));
    expect(
      (await pool.execute(
        r'INSERT INTO orders (item, quantity) VALUES ($1, 1)',
        <Object?>['scarf'],
      ))
          .unwrapOrElse((error) => fail('$error'))
          .lastInsertId,
      isNull,
    );
  });

  test('a transaction commits on Ok and reverts on Err', () async {
    final committed = await pool.transaction<Unit>((tx) async {
      expectOk(await tx.execute(
        r'INSERT INTO orders (item, quantity) VALUES ($1, 1)',
        <Object?>['kept'],
      ));
      return const Ok<Unit, SqlxError>(unit);
    });
    expect(committed.isOk, isTrue);

    final reverted = await pool.transaction<Unit>((tx) async {
      expectOk(await tx.execute(
        r'INSERT INTO orders (item, quantity) VALUES ($1, 1)',
        <Object?>['dropped'],
      ));
      return Err<Unit, SqlxError>(SqlxError.query('no', operation: 'test'));
    });
    expect(reverted.isErr, isTrue);

    final kept = await pool.fetchAll<String>(
      r'SELECT item FROM orders WHERE item = ANY($1)',
      <Object?>[
        <String>['kept', 'dropped'],
      ],
      (row) => row.read<String>('item'),
    );
    expect(kept.unwrapOrElse((error) => fail('$error')), <String>['kept']);
  });

  test('a nested transaction is a savepoint the outer one survives', () async {
    // The gap `package:postgres` leaves: it has no savepoint API, so this is
    // Dust's own. The point of the test is the last assertion — the outer
    // transaction still commits after the inner one rolled back.
    final result = await pool.transaction<Unit>((tx) async {
      expectOk(await tx.execute(
        r'INSERT INTO orders (item, quantity) VALUES ($1, 1)',
        <Object?>['outer'],
      ));

      final inner = await tx.transaction<Unit>((nested) async {
        expectOk(await nested.execute(
          r'INSERT INTO orders (item, quantity) VALUES ($1, 1)',
          <Object?>['inner'],
        ));
        return Err<Unit, SqlxError>(SqlxError.query('undo', operation: 'test'));
      });
      expect(inner.isErr, isTrue);

      return const Ok<Unit, SqlxError>(unit);
    });

    expect(result.isOk, isTrue, reason: 'the outer transaction still commits');

    final items = await pool.fetchAll<String>(
      r'SELECT item FROM orders WHERE item = ANY($1) ORDER BY item',
      <Object?>[
        <String>['outer', 'inner'],
      ],
      (row) => row.read<String>('item'),
    );
    expect(items.unwrapOrElse((error) => fail('$error')), <String>['outer']);
  });

  test('a failing statement is a value, not a throw', () async {
    final result = await pool.execute('SELECT * FROM no_such_table', const []);

    expect(result.isErr, isTrue);
  });
}

/// Returns the `Ok` value, failing the test with the error otherwise.
T expectOk<T>(Result<T, SqlxError> result) =>
    result.unwrapOrElse((error) => fail('expected Ok, got $error'));
