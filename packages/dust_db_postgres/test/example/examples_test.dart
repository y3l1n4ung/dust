@TestOn('vm')
library;

import 'package:test/test.dart';

import '../../example/arrays.dart' as arrays;
import '../../example/bytea.dart' as bytea;
import '../../example/connect.dart' as connect;
import '../../example/connect_options.dart' as connect_options;
import '../../example/constraint_violations.dart' as constraint_violations;
import '../../example/dust_db_postgres_example.dart' as tour;
import '../../example/error_handling.dart' as error_handling;
import '../../example/execute.dart' as execute;
import '../../example/fetch_all.dart' as fetch_all;
import '../../example/fetch_one.dart' as fetch_one;
import '../../example/fetch_optional.dart' as fetch_optional;
import '../../example/fetch_scalar.dart' as fetch_scalar;
import '../../example/json_columns.dart' as json_columns;
import '../../example/migrations.dart' as migrations;
import '../../example/nullable_columns.dart' as nullable_columns;
import '../../example/placeholders.dart' as placeholders;
import '../../example/returning.dart' as returning;
import '../../example/row_mappers.dart' as row_mappers;
import '../../example/savepoints.dart' as savepoints;
import '../../example/set_membership.dart' as set_membership;
import '../../example/timestamps.dart' as timestamps;
import '../../example/transactions.dart' as transactions;
import '../../example/unchecked_sql.dart' as unchecked_sql;
import '../support/postgres.dart';
import 'capture.dart';

void main() {
  group('connecting', () {
    test('connect', () async {
      final lines = await runExample(connect.main);
      // The version is the server's, so assert that it answered with one.
      expect(lines[0], startsWith('server: '));
      expect(lines[1], 'answered: 1');
    }, skip: skipWithoutDatabase);

    test('connect_options', () async {
      expect(await runExample(connect_options.main), <String>[
        'application_name: dust-example',
      ]);
    }, skip: skipWithoutDatabase);

    test('migrations', () async {
      expect(await runExample(migrations.main), <String>[
        'applied: true',
        'again: true',
        'columns: [id, name, region]',
      ]);
    }, skip: skipWithoutDatabase);
  });

  group('reading', () {
    test('fetch_one', () async {
      expect(await runExample(fetch_one.main), <String>[
        'found: Ada',
        'missing: cardinality',
      ]);
    }, skip: skipWithoutDatabase);

    test('fetch_optional', () async {
      expect(await runExample(fetch_optional.main), <String>[
        'present: Ada',
        'absent: null',
      ]);
    }, skip: skipWithoutDatabase);

    test('fetch_all', () async {
      expect(await runExample(fetch_all.main), <String>[
        'ids: [3, 1]',
        'empty: []',
      ]);
    }, skip: skipWithoutDatabase);

    test('fetch_scalar', () async {
      expect(await runExample(fetch_scalar.main), <String>[
        'orders: 3',
        'largest: 40.0',
        'over 1000: null',
      ]);
    }, skip: skipWithoutDatabase);

    test('row_mappers', () async {
      expect(await runExample(row_mappers.main), <String>[
        'User(ada@example.com, active: true, name: Ada)',
        'User(anon@example.com, active: true, name: -)',
      ]);
    }, skip: skipWithoutDatabase);

    test('nullable_columns', () async {
      expect(await runExample(nullable_columns.main), <String>[
        'Ada: https://example.com/ada.png',
        'Grace: no avatar',
      ]);
    }, skip: skipWithoutDatabase);
  });

  group('writing', () {
    test('execute', () async {
      expect(await runExample(execute.main), <String>[
        'inserted rows: 1',
        'lastInsertId: null',
        'updated: 1',
        'no match: 0',
      ]);
    }, skip: skipWithoutDatabase);

    test('returning', () async {
      expect(await runExample(returning.main), <String>[
        'created: #1 in EUR',
        'deleted: [99.5]',
      ]);
    }, skip: skipWithoutDatabase);

    test('transactions', () async {
      expect(await runExample(transactions.main), <String>[
        'small transfer: true',
        'overdraft: false',
        'balances: [70, 30]',
      ]);
    }, skip: skipWithoutDatabase);

    test('savepoints', () async {
      expect(await runExample(savepoints.main), <String>[
        'kept: [outer]',
        'escaped handle refused: true',
      ]);
    }, skip: skipWithoutDatabase);
  });

  group('values and parameters', () {
    test('placeholders', () async {
      expect(await runExample(placeholders.main), <String>[
        'reused: [Grace]',
        'sorted: [Ada, Grace]',
      ]);
    }, skip: skipWithoutDatabase);

    test('set_membership', () async {
      expect(await runExample(set_membership.main), <String>[
        'selected: [first, third]',
        'empty: []',
      ]);
    }, skip: skipWithoutDatabase);

    test('arrays', () async {
      expect(await runExample(arrays.main), <String>[
        'tags: [sql, postgres]',
        'tagged sqlite: [SQLite in practice]',
        'counts: [sql=2, postgres=1, sqlite=1]',
      ]);
    }, skip: skipWithoutDatabase);

    test('json_columns', () async {
      expect(await runExample(json_columns.main), <String>[
        'payload: {kind: signup, plan: pro, seats: 5}',
        'pro events: 1',
        'seats: [5, null]',
      ]);
    }, skip: skipWithoutDatabase);

    test('timestamps', () async {
      expect(await runExample(timestamps.main), <String>[
        'round trip: 2026-03-14 09:26:53.000Z',
        'expires: null',
        'in the last 30 days: 1',
      ]);
    }, skip: skipWithoutDatabase);

    test('bytea', () async {
      expect(await runExample(bytea.main), <String>[
        'bytes: 12',
        'decoded: hello, bytea',
        'size without reading: 12',
      ]);
    }, skip: skipWithoutDatabase);
  });

  group('when it goes wrong', () {
    test('error_handling', () async {
      expect(await runExample(error_handling.main), <String>[
        'category: query',
        'driver: postgres',
        'one: 1',
        'thrown at the boundary: cardinality',
      ]);
    }, skip: skipWithoutDatabase);

    test('constraint_violations', () async {
      expect(await runExample(constraint_violations.main), <String>[
        'first: created',
        'again: already taken',
        'upsert rows: 0',
      ]);
    }, skip: skipWithoutDatabase);

    test('unchecked_sql', () async {
      expect(await runExample(unchecked_sql.main), <String>[
        'plan returned: true',
        'columns: [id, email]',
      ]);
    }, skip: skipWithoutDatabase);
  });

  test('the tour runs end to end', () async {
    expect(await runExample(tour.main), <String>[
      'migrated: true',
      'created a user',
      'the user is Ada',
      'renamed: true',
      'users: [Grace]',
    ]);
  }, skip: skipWithoutDatabase);
}
