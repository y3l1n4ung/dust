import 'package:test/test.dart';

import '../../example/blobs.dart' as blobs;
import '../../example/connect_options.dart' as connect_options;
import '../../example/constraint_violations.dart' as constraint_violations;
import '../../example/dates_and_times.dart' as dates_and_times;
import '../../example/dust_db_sqlite3_example.dart' as tour;
import '../../example/error_handling.dart' as error_handling;
import '../../example/execute.dart' as execute;
import '../../example/fetch_all.dart' as fetch_all;
import '../../example/fetch_one.dart' as fetch_one;
import '../../example/fetch_optional.dart' as fetch_optional;
import '../../example/fetch_scalar.dart' as fetch_scalar;
import '../../example/foreign_keys.dart' as foreign_keys;
import '../../example/migrations.dart' as migrations;
import '../../example/nullable_columns.dart' as nullable_columns;
import '../../example/open_a_file.dart' as open_a_file;
import '../../example/open_in_memory.dart' as open_in_memory;
import '../../example/placeholders.dart' as placeholders;
import '../../example/read_only.dart' as read_only;
import '../../example/returning.dart' as returning;
import '../../example/row_mappers.dart' as row_mappers;
import '../../example/savepoints.dart' as savepoints;
import '../../example/set_membership.dart' as set_membership;
import '../../example/transactions.dart' as transactions;
import '../../example/unchecked_sql.dart' as unchecked_sql;
import 'capture.dart';

void main() {
  group('connecting', () {
    test('open_in_memory', () async {
      expect(await runExample(open_in_memory.main), <String>['users: 1']);
    });

    test('open_a_file', () async {
      expect(await runExample(open_a_file.main), <String>[
        'survived close: written once',
      ]);
    });

    test('connect_options', () async {
      expect(await runExample(connect_options.main), <String>[
        'journal: wal',
        'foreign keys: 1',
      ]);
    });

    test('read_only', () async {
      expect(await runExample(read_only.main), <String>[
        'reads: [acme]',
        'write refused: true',
      ]);
    });

    test('migrations', () async {
      expect(await runExample(migrations.main), <String>[
        'applied: [0001_create_users.sql, 0002_add_display_name.sql]',
      ]);
    });
  });

  group('reading', () {
    test('fetch_one', () async {
      expect(await runExample(fetch_one.main), <String>[
        'found: Ada',
        'missing: cardinality',
      ]);
    });

    test('fetch_optional', () async {
      expect(await runExample(fetch_optional.main), <String>[
        'present: Ada',
        'absent: null',
      ]);
    });

    test('fetch_all', () async {
      expect(await runExample(fetch_all.main), <String>[
        'ids: [3, 1]',
        'empty: []',
      ]);
    });

    test('fetch_scalar', () async {
      expect(await runExample(fetch_scalar.main), <String>[
        'orders: 3',
        'largest: 40.0',
        'over 1000: null',
      ]);
    });

    test('row_mappers', () async {
      expect(await runExample(row_mappers.main), <String>[
        'User(1, ada@example.com, Ada)',
        'User(2, anon@example.com, -)',
      ]);
    });

    test('nullable_columns', () async {
      expect(await runExample(nullable_columns.main), <String>[
        'Ada: https://example.com/ada.png',
        'Grace: no avatar',
      ]);
    });
  });

  group('writing', () {
    test('execute', () async {
      expect(await runExample(execute.main), <String>[
        'inserted id: 1',
        'updated: 1',
        'no match: 0',
      ]);
    });

    test('returning', () async {
      expect(await runExample(returning.main), <String>[
        'created: #1 in EUR',
        'deleted: [99.5]',
      ]);
    });

    test('transactions', () async {
      expect(await runExample(transactions.main), <String>[
        'small transfer: true',
        'overdraft: false',
        'balances: [70, 30]',
      ]);
    });

    test('savepoints', () async {
      expect(await runExample(savepoints.main), <String>[
        'kept: [outer]',
        'escaped handle refused: true',
      ]);
    });
  });

  group('values and parameters', () {
    test('placeholders', () async {
      expect(await runExample(placeholders.main), <String>[
        'reused: [Grace]',
        r'literal: $1 is not a placeholder here',
      ]);
    });

    test('set_membership', () async {
      expect(await runExample(set_membership.main), <String>[
        'selected: [first, third]',
        'empty: []',
      ]);
    });

    test('blobs', () async {
      expect(await runExample(blobs.main), <String>[
        'bytes: 11',
        'decoded: hello, blob',
      ]);
    });

    test('dates_and_times', () async {
      expect(await runExample(dates_and_times.main), <String>[
        'round trip: 2026-03-14 09:26:53.000Z',
        'on 2026-03-14: 1',
      ]);
    });
  });

  group('when it goes wrong', () {
    test('error_handling', () async {
      expect(await runExample(error_handling.main), <String>[
        'category: query',
        'driver: sqlite3',
        'users: 0',
        'thrown at the boundary: cardinality',
      ]);
    });

    test('constraint_violations', () async {
      expect(await runExample(constraint_violations.main), <String>[
        'first: created',
        'again: already taken',
        'upsert rows: 0',
      ]);
    });

    test('foreign_keys', () async {
      expect(await runExample(foreign_keys.main), <String>[
        'without enforcement, orphan accepted: true',
        'with enforcement, orphan refused: true',
      ]);
    });

    test('unchecked_sql', () async {
      final lines = await runExample(unchecked_sql.main);
      // The query plan's wording is SQLite's, so assert that the index is used
      // rather than on the sentence it uses to say so.
      expect(lines[0], contains('users_email'));
      expect(lines.sublist(1), <String>[
        'columns: [id, email]',
        'vacuumed: true',
      ]);
    });
  });

  test('the tour runs end to end', () async {
    expect(await runExample(tour.main), <String>[
      'created user 1',
      'user 1 is Ada',
      'renamed: true',
      'users: [Grace]',
    ]);
  });
}
