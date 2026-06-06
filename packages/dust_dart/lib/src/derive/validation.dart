import 'package:collection/collection.dart';

import 'base.dart';

/// Custom validator callback used by field-level `@Validate(custom: ...)`.
typedef FieldValidator<T> = ValidationError? Function(T value);

/// Generates validation methods and configures field validators.
final class Validate<T> extends DeriveTrait {
  /// Creates one validation derive marker or field-level validator config.
  const Validate({
    this.email = false,
    this.url = false,
    this.length,
    this.range,
    this.contains,
    this.doesNotContain,
    this.regex,
    this.mustMatch,
    this.nested = false,
    this.custom,
    this.required = false,
    this.message,
  });

  /// Validates that a string field has email shape.
  final bool email;

  /// Validates that a string field has URL shape.
  final bool url;

  /// Validates string or collection length.
  final Length? length;

  /// Validates numeric range.
  final Range? range;

  /// Validates that a string field contains this pattern.
  final String? contains;

  /// Validates that a string field does not contain this pattern.
  final String? doesNotContain;

  /// Validates that a string field matches this regular expression.
  final String? regex;

  /// Validates that this field equals another field.
  final String? mustMatch;

  /// Validates a nested object with a generated `validate()` method.
  final bool nested;

  /// Custom field validator function reference.
  final FieldValidator<T>? custom;

  /// Fails nullable fields when the value is `null`.
  final bool required;

  /// Custom message applied to validators in this annotation.
  final String? message;
}

/// Type-safe length validator configuration.
final class Length {
  /// Creates one length validator.
  const Length({this.min, this.max, this.exact});

  /// Minimum allowed length.
  final int? min;

  /// Maximum allowed length.
  final int? max;

  /// Exact required length.
  final int? exact;

  @override
  bool operator ==(Object other) {
    return other is Length &&
        other.min == min &&
        other.max == max &&
        other.exact == exact;
  }

  @override
  int get hashCode => Object.hash(min, max, exact);
}

/// Type-safe numeric range validator configuration.
final class Range {
  /// Creates one numeric range validator.
  const Range({this.min, this.max});

  /// Minimum allowed value.
  final num? min;

  /// Maximum allowed value.
  final num? max;

  @override
  bool operator ==(Object other) {
    return other is Range && other.min == min && other.max == max;
  }

  @override
  int get hashCode => Object.hash(min, max);
}

/// Result returned by generated validation.
sealed class ValidationResult {
  /// Creates one validation result.
  const ValidationResult();

  /// Returns `true` when this result has no validation errors.
  bool get isValid;

  /// Returns validation errors, or an empty list when valid.
  List<ValidationError> get errors;
}

/// Successful validation result.
final class Valid extends ValidationResult {
  /// Creates one successful validation result.
  const Valid();

  @override
  bool get isValid => true;

  @override
  List<ValidationError> get errors => const [];

  @override
  bool operator ==(Object other) => other is Valid;

  @override
  int get hashCode => 0;
}

/// Failed validation result.
final class Invalid extends ValidationResult {
  /// Creates one failed validation result.
  const Invalid(this.errors);

  @override
  final List<ValidationError> errors;

  @override
  bool get isValid => false;

  @override
  bool operator ==(Object other) {
    return other is Invalid && _errorListEquality.equals(other.errors, errors);
  }

  @override
  int get hashCode => _errorListEquality.hash(errors);
}

/// One validation failure.
final class ValidationError {
  /// Creates one validation error.
  const ValidationError({required this.field, required this.message});

  /// Field path that failed validation.
  final String field;

  /// Human readable validation failure message.
  final String message;

  /// Converts this error to a JSON-compatible map.
  Map<String, Object?> toJson() {
    return <String, Object?>{'field': field, 'message': message};
  }

  @override
  bool operator ==(Object other) {
    return other is ValidationError &&
        other.field == field &&
        other.message == message;
  }

  @override
  int get hashCode => Object.hash(field, message);
}

/// Exception thrown by generated `validateOrThrow()`.
final class ValidationException implements Exception {
  /// Creates one validation exception.
  const ValidationException(this.errors);

  /// Validation errors that caused the exception.
  final List<ValidationError> errors;

  @override
  String toString() => 'ValidationException($errors)';
}

/// Shared helpers used by generated validation code.
abstract final class ValidationHelper {
  static final RegExp _emailPattern = RegExp(r'^[^@\s]+@[^@\s]+\.[^@\s]+$');

  /// Returns `true` when [value] has a basic email shape.
  static bool isEmail(String value) => _emailPattern.hasMatch(value);

  /// Returns `true` when [value] is an absolute URL with scheme and host.
  static bool isUrl(String value) {
    final uri = Uri.tryParse(value);
    return uri != null && uri.scheme.isNotEmpty && uri.host.isNotEmpty;
  }
}

const _errorListEquality = ListEquality<ValidationError>();
