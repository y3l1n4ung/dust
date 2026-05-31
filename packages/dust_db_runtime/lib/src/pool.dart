import 'package:dust_db_annotation/dust_db_annotation.dart';

import 'result.dart';

/// Executes SQL against a database pool or transaction.
abstract interface class SqlxDriver {
  /// Database driver used by this SQLx driver.
  Driver get driver;

  /// Explicit unchecked SQL access for dynamic/admin queries.
  RawSql get raw;

  /// Fetches rows for a SQL query.
  Future<Result<List<Row>, SqlxError>> fetch(String sql, List<Object?> parameters);

  /// Runs a statement and returns the affected row count when available.
  Future<Result<ExecResult, SqlxError>> execute(String sql, List<Object?> parameters);

  /// Runs [fn] inside a database transaction.
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  );

  /// Closes resources owned by this driver.
  Future<Result<Unit, SqlxError>> close();
}

/// Long-lived database pool.
abstract interface class Pool implements SqlxDriver {}

/// Transaction-scoped SQLx driver.
abstract interface class Transaction implements SqlxDriver {}

/// Explicit unchecked SQL access.
abstract interface class RawSql {
  /// Runs unchecked SQL and returns rows.
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  );

  /// Runs an unchecked statement.
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  );
}

/// Public unchecked SQL wrapper for app-level composition.
final class RawSqlx implements RawSql {
  /// Creates one raw SQL wrapper.
  const RawSqlx(this._db);

  final SqlxDriver _db;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  ) {
    return _db.fetch(sql, parameters);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) {
    return _db.execute(sql, parameters);
  }
}

/// One database result row.
abstract interface class Row {
  /// Reads a non-null value by column name.
  T read<T>(String column);

  /// Reads a nullable value by column name.
  T? readOrNull<T>(String column);

  /// Reads a non-null value by column index.
  T readIndex<T>(int index);

  /// Reads a nullable value by column index.
  T? readIndexOrNull<T>(int index);

  /// Reads a SQLite/Postgres boolean-compatible column.
  bool readBool(String column);

  /// Reads a nullable SQLite/Postgres boolean-compatible column.
  bool? readBoolOrNull(String column);

  /// Reads an ISO-8601 date/time text column.
  DateTime readDateTime(String column);

  /// Reads a nullable ISO-8601 date/time text column.
  DateTime? readDateTimeOrNull(String column);
}
