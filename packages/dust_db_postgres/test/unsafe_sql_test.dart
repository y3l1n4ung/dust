import 'package:dust_dart/db.dart';
import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:test/test.dart';

import 'support/postgres.dart';

/// The escape hatch, and the reach it deliberately does not have.

const _ddl = '''
CREATE TABLE unsafe_items (
  id   BIGSERIAL PRIMARY KEY,
  name TEXT NOT NULL
)''';

void main() {
  // No in-memory PostgreSQL, so a suite cannot bring its own database.
  // Reporting a skip is how the absence stays visible.
  if (databaseUrl == null) {
    test('unchecked SQL', () {}, skip: skipWithoutDatabase);
    return;
  }

  late PostgresDriver db;

  setUp(() async {
    db = await tableFor(_ddl, 'unsafe_items');
    expectOk(
      await db.unsafe.execute(
        r'INSERT INTO unsafe_items (name) VALUES ($1), ($2)',
        const <Object?>['first', 'second'],
      ),
    );
  });

  test('fetch returns untyped rows', () async {
    final rows = expectOk(
      await db.unsafe.fetch(
        'SELECT name FROM unsafe_items ORDER BY id',
        const <Object?>[],
      ),
    );

    expect(rows.map((row) => row.read<String>('name')), <String>[
      'first',
      'second',
    ]);
  });

  test('fetchAs decodes with the mapper it is given', () async {
    // The decoder is passed by hand on purpose: generated terminals exist only
    // for validated queries, and the checked path stays the ergonomic one.
    final names = expectOk(
      await db.unsafe.fetchAs<String>(
        'SELECT name FROM unsafe_items ORDER BY id',
        const <Object?>[],
        (row) => row.read<String>('name'),
      ),
    );

    expect(names, <String>['first', 'second']);
  });

  test('execute reports how many rows changed', () async {
    final deleted = expectOk(
      await db.unsafe.execute('DELETE FROM unsafe_items', const <Object?>[]),
    );

    expect(deleted.rowsAffected, 2);
  });

  test('a failing statement is a value, not a throw', () async {
    final result = await db.unsafe.fetch(
      'SELECT * FROM no_such_table',
      const <Object?>[],
    );

    expect(expectErr(result).category, SqlxErrorCategory.query);
  });

  test('a failing mapper is a decode error', () async {
    final result = await db.unsafe.fetchAs<int>(
      'SELECT name FROM unsafe_items',
      const <Object?>[],
      (row) => row.read<int>('name'),
    );

    expect(expectErr(result).category, SqlxErrorCategory.decode);
  });

  test('a mapper reading a missing column is a decode error', () async {
    final result = await db.unsafe.fetchAs<String>(
      'SELECT name FROM unsafe_items',
      const <Object?>[],
      (row) => row.read<String>('absent'),
    );

    expect(expectErr(result).category, SqlxErrorCategory.decode);
  });

  test('an executor is not a database facade', () async {
    // `unsafe` is on `DatabaseClient`, and a driver is not one. This is the
    // difference from a `raw` channel hanging off the executor, where a cast
    // always succeeded.
    expect(db, isA<Executor>());
    expect(db, isNot(isA<DatabaseClient>()));

    final escaped = await db.transaction<bool>((tx) async {
      return Ok<bool, SqlxError>(tx is DatabaseClient);
    });
    expect(expectOk(escaped), isFalse);
  });

  test('a mapper throwing something other than a SqlxError is a decode error',
      () async {
    final result = await db.unsafe.fetchAs<int>(
      'SELECT name FROM unsafe_items',
      const <Object?>[],
      (row) => throw StateError('mapper blew up'),
    );

    expect(expectErr(result).category, SqlxErrorCategory.decode);
  });
}
