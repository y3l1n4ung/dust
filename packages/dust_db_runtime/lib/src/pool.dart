import 'package:dust_db_annotation/dust_db_annotation.dart';

import 'result.dart';
import 'row_mapper.dart';

/// Executes typed SQLx-style queries against a database pool or transaction.
abstract interface class SqlxDriver {
  /// Database driver used by this SQLx driver.
  Driver get driver;

  /// Explicit unchecked SQL access for dynamic/admin queries.
  RawSql get raw;

  /// Runs a checked row query that returns zero or one row.
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  );

  /// Runs a checked row query that returns all rows.
  Future<Result<List<T>, SqlxError>> fetchAll<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  );

  /// Runs a checked row query that must return exactly one row.
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  );

  /// Runs a checked scalar query and reads column index zero.
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  );

  /// Runs a checked statement and returns execution metadata.
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  );

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
    return _db.raw.fetch(sql, parameters);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) {
    return _db.raw.execute(sql, parameters);
  }
}

/// One database result row.
abstract interface class Row {
  /// Reads a non-null value by column name.
  T read<T>(String column);

  /// Reads a nullable value by column name.
  T? readOrNull<T>(String column);

  /// Reads a non-null value by column index.
  ///
  /// Generated row mappers never use this. It exists for scalar fetch internals
  /// and raw-query escape hatches.
  T readIndex<T>(int index);

  /// Reads a nullable value by column index.
  ///
  /// Generated row mappers never use this. It exists for scalar fetch internals
  /// and raw-query escape hatches.
  T? readIndexOrNull<T>(int index);

  /// Reads a SQLite/Postgres boolean-compatible column.
  bool readBool(String column);

  /// Reads a nullable SQLite/Postgres boolean-compatible column.
  bool? readBoolOrNull(String column);

  /// Reads an ISO-8601 date/time text column and normalizes it to UTC.
  DateTime readDateTime(String column);

  /// Reads a nullable ISO-8601 date/time text column and normalizes it to UTC.
  DateTime? readDateTimeOrNull(String column);
}
