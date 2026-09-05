import 'package:dust_dart/db.dart';
import 'package:test/test.dart';

/// Returns the `Ok` value, failing the test with the error otherwise.
///
/// The query terminals return `Result`, and a test that reaches for the value
/// is asserting the call succeeded. Spelling that out at every call site buries
/// what each test is actually checking, and `unwrapOr` would hide a failure
/// behind a fallback.
T expectOk<T>(Result<T, SqlxError> result) {
  return result.unwrapOrElse((error) => fail('expected Ok, got $error'));
}
