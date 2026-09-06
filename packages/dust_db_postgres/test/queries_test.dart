import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// The five executor primitives, and what each does when the query is wrong.

const _ddl = '''
CREATE TABLE query_orders (
  id       BIGSERIAL PRIMARY KEY,
  item     TEXT NOT NULL,
  quantity INTEGER NOT NULL
)''';

/// Reads one row of `query_orders`.
({int id, String item, int quantity}) _order(Row row) => (
      id: row.read<int>('id'),
      item: row.read<String>('item'),
      quantity: row.read<int>('quantity'),
    );

void main() {
  // No in-memory PostgreSQL, so a suite cannot bring its own database.
  // Reporting a skip is how the absence stays visible.
  if (databaseUrl == null) {
    test('queries', () {}, skip: skipWithoutDatabase);
    return;
  }

  late PostgresDriver db;

  setUp(() async {
    db = await tableFor(_ddl, 'query_orders');
    for (final item in const <String>['shirt', 'socks']) {
      expectOk(
        await db.execute(
          r'INSERT INTO query_orders (item, quantity) VALUES ($1, 1)',
          <Object?>[item],
        ),
      );
    }
  });

  group('fetchOne', () {
    test('returns the single row', () async {
      final order = await db.fetchOne(
        r'SELECT id, item, quantity FROM query_orders WHERE item = $1',
        <Object?>['shirt'],
        _order,
      );

      expect(expectOk(order).item, 'shirt');
    });

    test('reports no rows', () async {
      final order = await db.fetchOne(
        r'SELECT id, item, quantity FROM query_orders WHERE item = $1',
        <Object?>['absent'],
        _order,
      );

      expect(expectErr(order).category, SqlxErrorCategory.cardinality);
    });

    test('reports more rows than one', () async {
      final order = await db.fetchOne(
        'SELECT id, item, quantity FROM query_orders',
        const <Object?>[],
        _order,
      );

      expect(expectErr(order).category, SqlxErrorCategory.cardinality);
    });

    test('reports a failing mapper as a decode error', () async {
      final order = await db.fetchOne(
        'SELECT id FROM query_orders LIMIT 1',
        const <Object?>[],
        _order,
      );

      expect(expectErr(order).category, SqlxErrorCategory.decode);
    });
  });

  group('fetchOptional', () {
    test('returns null for no rows', () async {
      final order = await db.fetchOptional(
        r'SELECT id, item, quantity FROM query_orders WHERE item = $1',
        <Object?>['absent'],
        _order,
      );

      expect(expectOk(order), isNull);
    });

    test('returns the row when there is one', () async {
      final order = await db.fetchOptional(
        r'SELECT id, item, quantity FROM query_orders WHERE item = $1',
        <Object?>['socks'],
        _order,
      );

      expect(expectOk(order)?.item, 'socks');
    });

    test('reports more rows than one', () async {
      final order = await db.fetchOptional(
        'SELECT id, item, quantity FROM query_orders',
        const <Object?>[],
        _order,
      );

      expect(expectErr(order).category, SqlxErrorCategory.cardinality);
    });

    test('reports a failing mapper as a decode error', () async {
      final order = await db.fetchOptional(
        'SELECT id FROM query_orders LIMIT 1',
        const <Object?>[],
        _order,
      );

      expect(expectErr(order).category, SqlxErrorCategory.decode);
    });
  });

  group('fetchAll', () {
    test('returns every row', () async {
      final orders = await db.fetchAll(
        'SELECT id, item, quantity FROM query_orders ORDER BY item',
        const <Object?>[],
        _order,
      );

      expect(
        expectOk(orders).map((order) => order.item),
        <String>['shirt', 'socks'],
      );
    });

    test('returns nothing for no rows', () async {
      final orders = await db.fetchAll(
        r'SELECT id, item, quantity FROM query_orders WHERE item = $1',
        <Object?>['absent'],
        _order,
      );

      expect(expectOk(orders), isEmpty);
    });

    test('reports a failing mapper as a decode error', () async {
      final orders = await db.fetchAll(
        'SELECT id FROM query_orders',
        const <Object?>[],
        _order,
      );

      expect(expectErr(orders).category, SqlxErrorCategory.decode);
    });
  });

  group('fetchScalar', () {
    test('reads column zero', () async {
      final count = await db.fetchScalar<int>(
        'SELECT count(*) FROM query_orders',
        const <Object?>[],
      );

      expect(expectOk(count), 2);
    });

    test('reports no rows', () async {
      final item = await db.fetchScalar<String>(
        r'SELECT item FROM query_orders WHERE item = $1',
        <Object?>['absent'],
      );

      expect(expectErr(item).category, SqlxErrorCategory.cardinality);
    });

    test('reports more rows than one', () async {
      final item = await db.fetchScalar<String>(
        'SELECT item FROM query_orders',
        const <Object?>[],
      );

      expect(expectErr(item).category, SqlxErrorCategory.cardinality);
    });

    test('reports a null scalar rather than returning it', () async {
      final item = await db.fetchScalar<String>(
        'SELECT NULL::text',
        const <Object?>[],
      );

      expect(expectErr(item).category, SqlxErrorCategory.decode);
    });

    test('returns a null scalar when the type admits one', () async {
      // What `QueryScalar.fetchOptional` asks for. An aggregate over no rows
      // is NULL, so a caller that declared the value optional gets it rather
      // than a decode error.
      final total = await db.fetchScalar<int?>(
        r'SELECT max(id) FROM query_orders WHERE item = $1',
        <Object?>['absent'],
      );

      expect(expectOk(total), isNull);
    });

    test('returns null for no rows when the type admits one', () async {
      final item = await db.fetchScalar<String?>(
        r'SELECT item FROM query_orders WHERE item = $1',
        <Object?>['absent'],
      );

      expect(expectOk(item), isNull);
    });
  });

  group('execute', () {
    test('reports how many rows changed', () async {
      final deleted = await db.execute(
        'DELETE FROM query_orders',
        const <Object?>[],
      );

      expect(expectOk(deleted).rowsAffected, 2);
    });

    test('has no lastInsertId, because PostgreSQL has none', () async {
      final inserted = await db.execute(
        r'INSERT INTO query_orders (item, quantity) VALUES ($1, 1)',
        <Object?>['hat'],
      );

      expect(expectOk(inserted).lastInsertId, isNull);
    });

    test('reports a failing statement as a value', () async {
      final broken = await db.execute(
        'DELETE FROM no_such_table',
        const <Object?>[],
      );

      expect(expectErr(broken).category, SqlxErrorCategory.query);
    });
  });

  test('the driver reports which database it is', () async {
    expect(db.driver, Driver.postgres);
  });

  test('closing twice is not an error', () async {
    final closing = connect();
    expectOk(await closing.close());
    expectOk(await closing.close());
  });

  test('a mapper throwing something other than a SqlxError is a decode error',
      () async {
    // The narrow `on SqlxError` branch passes a driver error through; anything
    // else a mapper throws is still the row failing to decode.
    final orders = await db.fetchAll<int>(
      'SELECT id FROM query_orders',
      const <Object?>[],
      (row) => throw StateError('mapper blew up'),
    );

    expect(expectErr(orders).category, SqlxErrorCategory.decode);
  });
}
