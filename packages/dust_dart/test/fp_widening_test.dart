import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

// Pins what the README's "Build a result as the type you read it as" section
// says. In 0.2.x, andThen, orElse, unwrapOr and unwrapOrElse check their
// arguments against the type a value was built with, so reading a narrow value
// through a wider type compiles and then throws. If that changes, update the
// README section and these expectations together.

sealed class AppError {}

final class NotFound extends AppError {}

final class Invalid extends AppError {}

// Crashes: each function names its own narrow error type.
Result<int, NotFound> findNarrow(int id) => id > 0 ? Ok(id) : Err(NotFound());
Result<int, Invalid> validateNarrow(int value) =>
    value < 100 ? Ok(value) : Err(Invalid());

Result<int, AppError> updateNarrow(int id) {
  final Result<int, AppError> found = findNarrow(id);
  return found.andThen(validateNarrow);
}

// Works: each function returns the error type its caller reads.
Result<int, AppError> find(int id) => id > 0 ? Ok(id) : Err(NotFound());
Result<int, AppError> validate(int value) =>
    value < 100 ? Ok(value) : Err(Invalid());

Result<int, AppError> update(int id) => find(id).andThen(validate);

// Works: widen a narrow result before chaining.
Result<int, AppError> updateWidened(int id) =>
    findNarrow(id).mapErr<AppError>((error) => error).andThen(
        (value) => validateNarrow(value).mapErr<AppError>((error) => error));

void main() {
  test('narrow crashes',
      () => expect(() => updateNarrow(5), throwsA(isA<TypeError>())));
  test('declared wide works', () {
    expect(update(5), const Ok<int, AppError>(5));
    expect(update(-1).isErr, isTrue);
    expect(update(500).match(ok: (_) => '', err: (e) => '${e.runtimeType}'),
        'Invalid');
  });
  test('widened works', () {
    expect(
        updateWidened(500).match(ok: (_) => '', err: (e) => '${e.runtimeType}'),
        'Invalid');
    expect(
        updateWidened(-1).match(ok: (_) => '', err: (e) => '${e.runtimeType}'),
        'NotFound');
    expect(updateWidened(5), const Ok<int, AppError>(5));
  });
  test('Option unwrapOr crash and fix', () {
    const Option<num> narrow = None<int>();
    expect(() => narrow.unwrapOr(2.5), throwsA(isA<TypeError>()));
    const Option<num> wide = None<num>();
    expect(wide.unwrapOr(2.5), 2.5);
  });
}
