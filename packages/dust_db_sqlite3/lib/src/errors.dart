part of 'sqlite_pool.dart';

SqlxError _sqliteConnectionError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.connection(
    message,
    cause: cause,
    driver: Driver.sqlite3,
    operation: operation,
  );
}

SqlxError _sqliteMigrationError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.migration(
    message,
    cause: cause,
    driver: Driver.sqlite3,
    operation: operation,
  );
}

SqlxError _sqliteQueryError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.query(
    message,
    cause: cause,
    driver: Driver.sqlite3,
    operation: operation,
  );
}

SqlxError _sqliteDecodeError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.decode(
    message,
    cause: cause,
    driver: Driver.sqlite3,
    operation: operation,
  );
}

SqlxError _sqliteTransactionError(
  String message, {
  Object? cause,
  String? operation,
}) {
  return SqlxError.transaction(
    message,
    cause: cause,
    driver: Driver.sqlite3,
    operation: operation,
  );
}

SqlxError _sqliteNoRows(String query) {
  return SqlxError.noRows(query, driver: Driver.sqlite3, operation: query);
}

SqlxError _sqliteTooManyRows({
  required int expected,
  required int actual,
  required String query,
}) {
  return SqlxError.tooManyRows(
    expected: expected,
    actual: actual,
    query: query,
    driver: Driver.sqlite3,
    operation: query,
  );
}

SqlxError _sqliteNullColumn(String column, String operation) {
  return SqlxError.nullColumn(
    column,
    driver: Driver.sqlite3,
    operation: operation,
  );
}
