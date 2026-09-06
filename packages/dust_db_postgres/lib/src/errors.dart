part of 'postgres_pool.dart';

/// Wraps a connection failure as a driver error.
SqlxError _postgresConnectionError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.connection(
    message,
    cause: cause,
    driver: Driver.postgres,
    operation: operation,
  );
}

/// Wraps a failed statement as a driver error.
SqlxError _postgresQueryError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.query(
    message,
    cause: cause,
    driver: Driver.postgres,
    operation: operation,
  );
}

/// Wraps a row-decoding failure as a driver error.
SqlxError _postgresDecodeError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.decode(
    message,
    cause: cause,
    driver: Driver.postgres,
    operation: operation,
  );
}

/// Wraps a transaction control failure as a driver error.
SqlxError _postgresTransactionError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.transaction(
    message,
    cause: cause,
    driver: Driver.postgres,
    operation: operation,
  );
}

/// Reports a query that returned nothing where one row was required.
SqlxError _postgresNoRows(String query) {
  return SqlxError.noRows(query, driver: Driver.postgres, operation: query);
}

/// Reports a query that returned more rows than the terminal allows.
SqlxError _postgresTooManyRows({
  required int expected,
  required int actual,
  required String query,
}) {
  return SqlxError.tooManyRows(
    expected: expected,
    actual: actual,
    query: query,
    driver: Driver.postgres,
    operation: query,
  );
}

/// Reports a null in a column the row type declared non-nullable.
SqlxError _postgresNullColumn(String column, String operation) {
  return SqlxError.nullColumn(
    column,
    driver: Driver.postgres,
    operation: operation,
  );
}

/// Turns any thrown object from `package:postgres` into a `SqlxError`.
///
/// A `PgException` carries the server's own message, which is more useful than
/// anything this layer could invent, so it is passed through as the cause.
SqlxError _asPostgresError(Object error, String sql) {
  if (error is SqlxError) return error;
  return _postgresQueryError(
    'PostgreSQL query failed.',
    cause: error,
    operation: sql,
  );
}
