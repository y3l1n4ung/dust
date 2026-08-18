import 'package:dust_dart/derive.dart';

import 'rejection.dart';

/// Turns the outcome of a `Validate()` derive into a rejection.
///
/// `dust_dart` reports failures as a flat list of `(field, message)` pairs,
/// while a 422 body groups them by field so a client can put each message
/// beside its input. This is the one place that regrouping happens, so every
/// application answers a failed constraint the same way.
extension ValidationRejection on ValidationResult {
  /// The 422 for these errors, or `null` when the value is valid.
  ///
  /// Every message survives: two failures on one field become two entries
  /// under that field rather than the first one winning. The response body is
  ///
  /// ```json
  /// {"error": "Validation failed", "fields": {"title": ["must not be empty"]}}
  /// ```
  Rejection? get rejection {
    if (errors.isEmpty) return null;
    return Rejection.unprocessable(
      groupValidationErrors(errors),
      message: 'Validation failed',
    );
  }
}

/// The 422 for an [error] thrown while building a value out of [subject].
///
/// A generated `deserialize` that enforces its own constraints throws
/// `ValidationException`, and those errors name the field that failed, so they
/// are handed back one field at a time. Anything else — a missing key, a value
/// of the wrong type — describes the shape rather than one field, and lands in
/// the message.
Rejection decodeRejection(Object error, {required String subject}) {
  if (error is ValidationException) {
    return Rejection.unprocessable(
      groupValidationErrors(error.errors),
      message: 'Validation failed',
    );
  }
  return Rejection.unprocessable(
    const {},
    message: '$subject does not match the expected shape: $error',
  );
}

/// Groups validation [errors] by the field they belong to, in report order.
Map<String, List<String>> groupValidationErrors(
  Iterable<ValidationError> errors,
) {
  final fields = <String, List<String>>{};
  for (final error in errors) {
    (fields[error.field] ??= <String>[]).add(error.message);
  }
  return fields;
}

/// Validating a value at the point a handler uses it.
extension ValidateOrReject on Validatable {
  /// Throws a 422 [Rejection] when this value fails its constraints.
  ///
  /// The counterpart to `validateOrThrow`, which throws a
  /// `ValidationException` that means nothing to HTTP. Use it inside a `guard`
  /// when the value did not come through `ValidatedExtractable`:
  ///
  /// ```dart
  /// return guard(() async {
  ///   final input = await const JsonExtractable(CreateTodo.deserialize)
  ///       .require(request);
  ///   input.validateOrReject();
  ///   ...
  /// });
  /// ```
  void validateOrReject() {
    final rejection = validate().rejection;
    if (rejection != null) throw rejection;
  }
}
