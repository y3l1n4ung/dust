import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

import 'db_runtime_support.dart';

void main() {
  test('DB annotations and value types expose stable configuration', () {
    const database = SqlxDatabase(driver: Driver.postgres);
    const typed = SqlxDatabase(type: SqlxDatabaseType.sqlite);
    const dao = SqlxDao();
    const query = Query(r'SELECT 1');
    const fromRow = FromRow();
    const sqlx = Sqlx(rename: 'display_name', defaultValue: 'anon');
    const tryFrom = IntStringTryFrom();
    const result = ExecResult(rowsAffected: 2, lastInsertId: 9);

    expect(database.type, SqlxDatabaseType.postgres);
    expect(database.migrations, './migrations');
    expect(typed.type, SqlxDatabaseType.sqlite);
    expect(dao, isA<SqlxDao>());
    expect(query.sql, r'SELECT 1');
    expect(fromRow, isA<FromRow>());
    expect(sqlx.rename, 'display_name');
    expect(sqlx.defaultValue, 'anon');
    expect(tryFrom.decode('9'), 9);
    expect(result.rowsAffected, 2);
    expect(result.lastInsertId, 9);
  });

  test('SqlxError variants expose useful string output', () {
    final driver = SqlxError.connection(
      'driver failed',
      driver: Driver.sqlite3,
      operation: 'open',
    );
    final driverCause = SqlxError.driver('driver failed', cause: 'boom');
    final decode = SqlxError.decode(
      'decode failed',
      driver: Driver.sqlite3,
      operation: 'read:name',
    );
    final decodeCause = SqlxError.decode('decode failed', cause: 'bad');
    final noRows = SqlxError.noRows('SELECT 1');
    final nullColumn = SqlxError.nullColumn('name');
    final tooMany = SqlxError.tooManyRows(expected: 1, actual: 2);

    expect(driver.toString(), 'driver failed');
    expect(driver.category, SqlxErrorCategory.connection);
    expect(driver.driver, Driver.sqlite3);
    expect(driver.operation, 'open');
    expect(driverCause.toString(), 'driver failed Cause: boom');
    expect(decode.toString(), 'decode failed');
    expect(decode.category, SqlxErrorCategory.decode);
    expect(decode.driver, Driver.sqlite3);
    expect(decode.operation, 'read:name');
    expect(decodeCause.toString(), 'decode failed Cause: bad');
    expect(noRows.toString(), 'SQL query `SELECT 1` expected 1 row(s), got 0.');
    expect(noRows.category, SqlxErrorCategory.cardinality);
    expect(nullColumn.toString(), 'Column `name` is null.');
    expect(tooMany.toString(), 'SQL query expected 1 row(s), got 2.');
  });

  test('DB JSON helper decodes objects and rejects non-objects', () {
    expect(decodeJsonObject('{"id":1}'), <String, Object?>{'id': 1});
    expect(() => decodeJsonObject('[1]'), throwsA(isA<FormatException>()));
  });
}
