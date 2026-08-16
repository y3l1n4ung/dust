import 'package:dust_dart/derive.dart';
import 'package:shelf/shelf.dart';

import '../response/rejection.dart';
import '../response/validation.dart';
import 'extractable.dart';

/// Runs the `Validate()` constraints after [inner] builds the value.
///
/// Decoding and validating are separate failures that deserve separate
/// messages: a body of the wrong shape never reaches the constraints, and a
/// body of the right shape with a bad value never looks like a parse error.
/// Both land on 422, so a client sees one error format either way.
///
/// ```dart
/// const createTodo = ValidatedExtractable(
///   JsonExtractable<CreateTodo>(CreateTodo.deserialize),
/// );
/// ```
///
/// Wrapping a body extractor keeps the body extractor's rules, which
/// `readsRequestBody` reports through the wrapper.
final class ValidatedExtractable<T extends Validatable>
    implements FromRequestParts<T> {
  /// Validates whatever [inner] produces.
  const ValidatedExtractable(this.inner);

  /// The extractor that builds the value.
  final FromRequestParts<T> inner;

  @override
  Future<Result<T, Rejection>> extract(Request request) async {
    switch (await inner.extract(request)) {
      case Err(:final error):
        return Err(error);
      case Ok(:final value):
        final rejection = value.validate().rejection;
        return rejection == null ? Ok(value) : Err(rejection);
    }
  }
}
