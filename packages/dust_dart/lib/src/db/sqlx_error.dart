import 'annotations.dart';
import 'sqlx_error_kinds.dart';

export 'sqlx_error_kinds.dart';

/// Base class for Database SQLx-style runtime errors.
sealed class SqlxError implements Exception {
  /// Creates one SQLx error.
  const SqlxError();

  /// Creates a driver error.
  factory SqlxError.driver(
    String message, {
    Object? cause,
    SqlxErrorCategory category = SqlxErrorCategory.driver,
    Driver? driver,
    String? operation,
  }) {
    return SqlxDriverError(
      message,
      cause: cause,
      category: category,
      driver: driver,
      operation: operation,
    );
  }

  /// Creates a connection error.
  factory SqlxError.connection(
    String message, {
    Object? cause,
    Driver? driver,
    String? operation,
  }) {
    return SqlxDriverError(
      message,
      cause: cause,
      category: SqlxErrorCategory.connection,
      driver: driver,
      operation: operation,
    );
  }

  /// Creates a migration error.
  factory SqlxError.migration(
    String message, {
    Object? cause,
    Driver? driver,
    String? operation,
  }) {
    return SqlxDriverError(
      message,
      cause: cause,
      category: SqlxErrorCategory.migration,
      driver: driver,
      operation: operation,
    );
  }

  /// Creates a query execution error.
  factory SqlxError.query(
    String message, {
    Object? cause,
    Driver? driver,
    String? operation,
    SqlxErrorKind? kind,
  }) {
    return SqlxDriverError(
      message,
      cause: cause,
      category: SqlxErrorCategory.query,
      driver: driver,
      operation: operation,
      kind: kind,
    );
  }

  /// Creates a transaction error.
  factory SqlxError.transaction(
    String message, {
    Object? cause,
    Driver? driver,
    String? operation,
    SqlxErrorKind? kind,
  }) {
    return SqlxDriverError(
      message,
      cause: cause,
      category: SqlxErrorCategory.transaction,
      driver: driver,
      operation: operation,
      kind: kind,
    );
  }

  /// Creates a decode error.
  factory SqlxError.decode(
    String message, {
    Object? cause,
    Driver? driver,
    String? operation,
  }) {
    return SqlxDecodeError(
      message,
      cause: cause,
      driver: driver,
      operation: operation,
    );
  }

  /// Creates a no-rows cardinality error.
  factory SqlxError.noRows(
    String query, {
    Driver? driver,
    String? operation,
  }) {
    return SqlxCardinalityError(
      query: query,
      expected: '1',
      actual: 0,
      driver: driver,
      operation: operation,
    );
  }

  /// Creates a null-column decode error.
  factory SqlxError.nullColumn(
    String column, {
    Driver? driver,
    String? operation,
  }) {
    return SqlxDecodeError(
      'Column `$column` is null.',
      driver: driver,
      operation: operation,
    );
  }

  /// Creates a cardinality error for too many rows.
  factory SqlxError.tooManyRows({
    required int expected,
    required int actual,
    String query = '',
    Driver? driver,
    String? operation,
  }) {
    return SqlxCardinalityError(
      query: query,
      expected: expected.toString(),
      actual: actual,
      driver: driver,
      operation: operation,
    );
  }

  /// Error category used for filtering or structured logs.
  SqlxErrorCategory get category;

  /// Short app-facing message.
  String get message;

  /// Database driver that produced the error, when known.
  Driver? get driver;

  /// Operation that failed, such as a SQL string or migration name.
  String? get operation;

  /// Original lower-level error, when available.
  Object? get cause;

  /// The integrity constraint the statement broke, when the driver knows.
  ///
  /// Null for every other failure, and for a constraint the driver did not
  /// recognise.
  SqlxErrorKind? get kind => null;
}

/// Error reported by a database driver.
final class SqlxDriverError extends SqlxError {
  /// Creates one driver error.
  const SqlxDriverError(
    this.message, {
    this.cause,
    this.category = SqlxErrorCategory.driver,
    this.driver,
    this.operation,
    this.kind,
  });

  @override
  final SqlxErrorCategory category;

  @override
  final SqlxErrorKind? kind;

  /// Human-readable error message.
  @override
  final String message;

  @override
  final Driver? driver;

  @override
  final String? operation;

  /// Original driver error, when available.
  @override
  final Object? cause;

  @override
  String toString() => cause == null ? message : '$message Cause: $cause';
}

/// Error produced while decoding a row into a Dart object.
final class SqlxDecodeError extends SqlxError {
  /// Creates one decode error.
  const SqlxDecodeError(
    this.message, {
    this.cause,
    this.driver,
    this.operation,
  });

  @override
  SqlxErrorCategory get category => SqlxErrorCategory.decode;

  /// Human-readable error message.
  @override
  final String message;

  @override
  final Driver? driver;

  @override
  final String? operation;

  /// Original decode error, when available.
  @override
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
    this.driver,
    this.operation,
  });

  @override
  SqlxErrorCategory get category => SqlxErrorCategory.cardinality;

  /// Query or generated method name.
  final String query;

  /// Expected row count description.
  final String expected;

  /// Actual row count.
  final int actual;

  @override
  final Driver? driver;

  @override
  final String? operation;

  @override
  Object? get cause => null;

  @override
  String get message => toString();

  @override
  String toString() {
    final prefix = query.isEmpty ? 'SQL query' : 'SQL query `$query`';
    return '$prefix expected $expected row(s), got $actual.';
  }
}
