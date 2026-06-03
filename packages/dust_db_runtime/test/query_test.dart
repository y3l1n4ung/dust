import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

import 'support/fakes.dart';

void main() {
  tearDown(RowMapperRegistry.resetForTest);

  test('queryAs maps through generated registry', () async {
    registerRowMapper<UserRow>((row) => UserRow(row.read<int>('id')));
    final pool = FakePool([
      FakeRow([7], {'id': 7}),
    ]);

    final row = await queryAs<UserRow>('SELECT id FROM users', []).fetchOne(pool);
    final optional = await queryAs<UserRow>(
      'SELECT id FROM users',
      const [],
    ).fetchOptional(pool);
    final rows = await queryAs<UserRow>(
      'SELECT id FROM users',
      const [],
    ).fetchAll(pool);

    expect(row.id, 7);
    expect(optional?.id, 7);
    expect(rows.single.id, 7);
    expect(pool.lastSql, 'SELECT id FROM users');
    expect(pool.lastParameters, isEmpty);
  });

  test('row mapper registry supports passthrough, reset, and missing mappers', () {
    final row = FakeRow([1], {'id': 1});

    expect(RowMapperRegistry.map<Row>(row), same(row));
    expect(() => RowMapperRegistry.map<UserRow>(row), throwsA(isA<StateError>()));

    registerRowMapper<UserRow>((row) => UserRow(row.read<int>('id')));
    expect(RowMapperRegistry.map<UserRow>(row).id, 1);

    RowMapperRegistry.resetForTest();
    expect(() => RowMapperRegistry.map<UserRow>(row), throwsA(isA<StateError>()));
  });

  test('queryScalar reads first selected column', () async {
    final pool = FakePool([
      FakeRow([3], {'count': 3}),
    ]);

    final count = await queryScalar<int>(
      'SELECT COUNT(*) FROM users',
      [],
    ).fetchOne(pool);
    final optional = await queryScalar<int>(
      'SELECT COUNT(*) FROM users',
      const [],
    ).fetchOptional(pool);

    expect(count, 3);
    expect(optional, 3);
  });

  test('queryExecute returns affected row count', () async {
    final pool = FakePool([]);

    final result = await queryExecute(r'UPDATE users SET name = $1', [
      'Ada',
    ]).execute(pool);

    expect(result.rowsAffected, 2);
    expect(pool.lastParameters, ['Ada']);
  });

  test('queryRaw and RawSqlx delegate unchecked SQL to driver raw channel', () async {
    final pool = FakePool([
      FakeRow([1], {'id': 1}),
    ]);

    final rows = await queryRaw(r'SELECT * FROM users WHERE id = $1', [
      1,
    ]).fetch(pool);
    final rawRows = await RawSqlx(pool).fetch(r'SELECT * FROM users', const []);
    final rawExec = await RawSqlx(pool).execute(r'DELETE FROM users', const []);

    expect(rows.single.read<int>('id'), 1);
    expect(
      rawRows.match(ok: (rows) => rows.single.read<int>('id'), err: (_) => -1),
      1,
    );
    expect(rawExec.match(ok: (result) => result.rowsAffected, err: (_) => -1), 2);
  });

  test('query helpers throw useful StateError when driver returns Err', () async {
    final pool = FakePool([], error: SqlxError.driver('down'));

    await expectLater(
      queryExecute('DELETE FROM users', const []).execute(pool),
      throwsA(
        isA<StateError>().having(
          (e) => e.toString(),
          'message',
          contains('down'),
        ),
      ),
    );
  });
}
