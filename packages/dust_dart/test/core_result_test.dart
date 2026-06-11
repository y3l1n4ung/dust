import 'package:dust_dart/core.dart';
import 'package:dust_dart/db.dart' as db;
import 'package:test/test.dart';

void main() {
  test('Ok maps chains and unwraps successful values', () {
    const result = Ok<int, String>(2);

    expect(result.isOk, isTrue);
    expect(result.isErr, isFalse);
    expect(result.map((value) => value * 3), const Ok<int, String>(6));
    expect(result.mapErr((error) => error.length), const Ok<int, int>(2));
    expect(
      result.andThen((value) => Ok<String, String>('value:$value')),
      const Ok<String, String>('value:2'),
    );
    expect(
      result.orElse<int>((error) => Err<int, int>(error.length)),
      const Ok<int, int>(2),
    );
    expect(result.unwrapOr(0), 2);
    expect(result.unwrapOrElse((error) => error.length), 2);
    expect(result.match(ok: (value) => value + 1, err: (_) => 0), 3);
    expect(result.toString(), 'Ok(2)');
  });

  test('Err maps errors chains fallback and unwraps default values', () {
    const result = Err<int, String>('bad');

    expect(result.isOk, isFalse);
    expect(result.isErr, isTrue);
    expect(result.map((value) => value * 3), const Err<int, String>('bad'));
    expect(result.mapErr((error) => error.length), const Err<int, int>(3));
    expect(
      result.andThen((value) => Ok<String, String>('value:$value')),
      const Err<String, String>('bad'),
    );
    expect(
      result.orElse<int>((error) => Ok<int, int>(error.length)),
      const Ok<int, int>(3),
    );
    expect(result.unwrapOr(7), 7);
    expect(result.unwrapOrElse((error) => error.length), 3);
    expect(
      result.match(ok: (value) => value + 1, err: (error) => error.length),
      3,
    );
    expect(result.toString(), 'Err(bad)');
  });

  test('Unit is a stable empty success value', () {
    expect(unit, const Unit());
    expect(unit.hashCode, const Unit().hashCode);
    expect(unit.toString(), 'unit');
  });

  test('DB library re-exports core result primitives for generated code', () {
    const result = db.Ok<int, db.SqlxError>(1);

    expect(result, isA<db.Result<int, db.SqlxError>>());
    expect(db.unit, const db.Unit());
  });
}
