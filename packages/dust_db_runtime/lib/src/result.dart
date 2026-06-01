/// SQLx-style operation result.
sealed class Result<T, E> {
  /// Creates one result value.
  const Result();

  /// Chains another result-producing operation when this result is successful.
  Result<R, E> andThen<R>(Result<R, E> Function(T value) next);

  /// Pattern matches this result.
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  });
}

/// Successful result.
final class Ok<T, E> extends Result<T, E> {
  /// Creates one successful result.
  const Ok(this.value);

  /// Successful value.
  final T value;

  @override
  Result<R, E> andThen<R>(Result<R, E> Function(T value) next) {
    return next(value);
  }

  @override
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  }) {
    return ok(value);
  }
}

/// Failed result.
final class Err<T, E> extends Result<T, E> {
  /// Creates one failed result.
  const Err(this.error);

  /// Error value.
  final E error;

  @override
  Result<R, E> andThen<R>(Result<R, E> Function(T value) next) {
    return Err<R, E>(error);
  }

  @override
  R match<R>({
    required R Function(T value) ok,
    required R Function(E error) err,
  }) {
    return err(error);
  }
}

/// Empty success value.
final class Unit {
  /// Creates one unit value.
  const Unit();
}

/// Shared unit value.
const unit = Unit();

/// Result of a SQL execute statement.
final class ExecResult {
  /// Creates one execute result.
  const ExecResult({required this.rowsAffected, this.lastInsertId});

  /// Number of rows affected by the statement.
  final int rowsAffected;

  /// Last inserted row id when the driver can report it.
  final int? lastInsertId;
}

/// Base class for Dust DB SQLx-style runtime errors.
sealed class SqlxError {
  /// Creates one SQLx error.
  const SqlxError();

  /// Creates a driver error.
  factory SqlxError.driver(String message, {Object? cause}) {
    return SqlxDriverError(message, cause: cause);
  }

  /// Creates a decode error.
  factory SqlxError.decode(String message, {Object? cause}) {
    return SqlxDecodeError(message, cause: cause);
  }

  /// Creates a no-rows cardinality error.
  factory SqlxError.noRows(String query) {
    return SqlxCardinalityError(query: query, expected: '1', actual: 0);
  }

  /// Creates a null-column decode error.
  factory SqlxError.nullColumn(String column) {
    return SqlxDecodeError('Column `$column` is null.');
  }

  /// Creates a cardinality error for too many rows.
  factory SqlxError.tooManyRows({
    required int expected,
    required int actual,
    String query = '',
  }) {
    return SqlxCardinalityError(
      query: query,
      expected: expected.toString(),
      actual: actual,
    );
  }
}

/// Error reported by a database driver.
final class SqlxDriverError extends SqlxError {
  /// Creates one driver error.
  const SqlxDriverError(this.message, {this.cause});

  /// Human-readable error message.
  final String message;

  /// Original driver error, when available.
  final Object? cause;

  @override
  String toString() => cause == null ? message : '$message Cause: $cause';
}

/// Error produced while decoding a row into a Dart object.
final class SqlxDecodeError extends SqlxError {
  /// Creates one decode error.
  const SqlxDecodeError(this.message, {this.cause});

  /// Human-readable error message.
  final String message;

  /// Original decode error, when available.
  final Object? cause;

  @override
  String toString() => cause == null ? message : '$message Cause: $cause';
}

/// Error produced when a query returns the wrong number of rows.
final class SqlxCardinalityError extends SqlxError {
  /// Creates one cardinality error.
  const SqlxCardinalityError({
    required this.query,
    required this.expected,
    required this.actual,
  });

  /// Query or generated method name.
  final String query;

  /// Expected row count description.
  final String expected;

  /// Actual row count.
  final int actual;

  @override
  String toString() {
    final prefix = query.isEmpty ? 'SQL query' : 'SQL query `$query`';
    return '$prefix expected $expected row(s), got $actual.';
  }
}
