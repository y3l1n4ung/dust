import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:test/test.dart';

void main() {
  test('result andThen chains ok and preserves err', () {
    final ok = const Ok<int, SqlxError>(
      2,
    ).andThen<String>((value) => Ok<String, SqlxError>('value:$value'));
    final err = const Err<int, SqlxError>(
      SqlxDecodeError('bad'),
    ).andThen<String>((value) => Ok<String, SqlxError>('value:$value'));

    expect(ok, isA<Ok<String, SqlxError>>());
    expect(ok.match(ok: (value) => value, err: (_) => 'err'), 'value:2');
    expect(err, isA<Err<String, SqlxError>>());
  });

  test('sqlx error factories preserve details', () {
    final driver = SqlxError.driver('driver failed', cause: 'cause');
    final decode = SqlxError.decode('decode failed', cause: 'cause');
    final cardinality = SqlxError.tooManyRows(expected: 1, actual: 3);
    final noRows = SqlxError.noRows('SELECT 1');
    final nullColumn = SqlxError.nullColumn('name');

    expect(driver, isA<SqlxDriverError>());
    expect(decode, isA<SqlxDecodeError>());
    expect(cardinality, isA<SqlxCardinalityError>());
    expect((cardinality as SqlxCardinalityError).actual, 3);
    expect(noRows.toString(), contains('expected 1 row(s), got 0'));
    expect(nullColumn.toString(), contains('Column `name` is null'));
    expect(driver.toString(), contains('Cause: cause'));
    expect(decode.toString(), contains('Cause: cause'));
  });
}
