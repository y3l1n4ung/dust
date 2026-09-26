import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

// A value built with a narrow type and read through a wider one.
//
// Dart generics are covariant, so a `Result<int, NotFound>` can be held in a
// `Result<int, AppError>` variable. Until 0.3.0, `andThen`, `orElse`,
// `unwrapOr`, and `unwrapOrElse` were members, checked their arguments against
// the type the value was built with, and threw a `TypeError` here although the
// analyzer accepted every call. As extensions their types come from the call
// site, so none of these may throw.

sealed class AppError {}

final class NotFound extends AppError {}

final class Invalid extends AppError {}

Result<int, NotFound> find(int id) => id > 0 ? Ok(id) : Err(NotFound());
Result<int, Invalid> validate(int value) =>
    value < 100 ? Ok(value) : Err(Invalid());

Result<int, AppError> update(int id) {
  final Result<int, AppError> found = find(id);
  return found.andThen(validate);
}

String describe(Result<int, AppError> result) =>
    result.match(ok: (value) => 'ok $value', err: (e) => '${e.runtimeType}');

void main() {
  group('Result read through a wider type', () {
    test('andThen accepts a step that fails with another error', () {
      expect(describe(update(5)), 'ok 5');
      expect(describe(update(-1)), 'NotFound');
      expect(describe(update(500)), 'Invalid');
    });

    test('orElse accepts a recovery of the wider type', () {
      const Result<num, String> narrow = Err<int, String>('missing');

      expect(
        narrow.orElse((_) => const Ok<num, String>(2.5)),
        const Ok<num, String>(2.5),
      );
    });

    test('unwrapOr and unwrapOrElse accept a fallback of the wider type', () {
      const Result<num, String> narrow = Err<int, String>('missing');

      expect(narrow.unwrapOr(2.5), 2.5);
      expect(narrow.unwrapOrElse((_) => 2.5), 2.5);
    });
  });

  group('Option read through a wider type', () {
    test('unwrapOr and unwrapOrElse accept a fallback of the wider type', () {
      const Option<num> narrow = None<int>();

      expect(narrow.unwrapOr(2.5), 2.5);
      expect(narrow.unwrapOrElse(() => 2.5), 2.5);
      expect(const Some<int>(1).unwrapOr(0), 1);
    });
  });
}
