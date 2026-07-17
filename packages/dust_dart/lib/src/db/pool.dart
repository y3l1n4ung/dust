import 'annotations.dart';
import '../fp/result.dart';
import '../fp/unit.dart';
import 'exec_result.dart';
import 'row_mapper.dart';
import 'sqlx_error.dart';

/// Executes typed SQLx-style queries against a database connection,
/// transaction, or driver.
///
/// Generated DAO code receives a [DatabaseExecutor] and calls `fetchOptional`,
/// `fetchAll`, `fetchOne`, `fetchScalar`, or `execute` depending on the
/// annotated method return type.
abstract interface class DatabaseExecutor {
  /// Database driver used by this executor.
  Driver get driver;

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
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  );

  /// Closes resources owned by this driver.
  Future<Result<Unit, SqlxError>> close();
}

/// Open generated application database facade.
abstract interface class DatabaseClient {
  /// Open database connection used by generated DAOs.
  DatabaseConnection get connection;
}

/// Convenience methods for generated application database facades.
extension DatabaseClientExecution on DatabaseClient {
  /// Typed query executor for this database.
  DatabaseExecutor get executor => connection;

  /// Runs [fn] inside a database transaction.
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) {
    return connection.transaction(fn);
  }

  /// Closes resources owned by this database connection.
  Future<Result<Unit, SqlxError>> close() {
    return connection.close();
  }
}

/// Legacy name for [DatabaseExecutor] with explicit raw SQL access.
///
/// New generated DAO code should depend on [DatabaseExecutor]. Use [Executor]
/// only when an advanced raw SQL escape hatch is required.
abstract interface class Executor implements DatabaseExecutor {
  /// Explicit unchecked SQL access for dynamic/admin queries.
  RawSql get raw;
}

/// Backwards-compatible name for the DB execution contract.
typedef SqlxDriver = DatabaseExecutor;

/// Open database connection.
abstract interface class DatabaseConnection implements DatabaseExecutor {}

/// Transaction-scoped database executor.
abstract interface class DatabaseTransaction implements DatabaseExecutor {}

/// Backwards-compatible long-lived database pool name.
abstract interface class Pool implements DatabaseConnection, Executor {}

/// Backwards-compatible single database connection name.
abstract interface class Connection implements DatabaseConnection, Executor {}

/// Backwards-compatible transaction-scoped executor name.
abstract interface class Transaction implements DatabaseTransaction, Executor {}

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

  final Executor _db;

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

/// Driver-agnostic typed view over one database result row.
///
/// This mirrors sqlx's `Row` role. Driver packages own concrete row adapters,
/// while generated `FromRow` mappers read through this interface.
abstract interface class Row {
  /// Reads a non-null value by column name.
  T read<T>(String column);

  /// Reads a nullable value by column name.
  T? readNullable<T>(String column);

  /// Reads a non-null value by column index.
  ///
  /// Generated row mappers never use this. It exists for scalar fetch internals
  /// and raw-query escape hatches.
  T readIndex<T>(int index);

  /// Reads a nullable value by column index.
  ///
  /// Generated row mappers never use this. It exists for scalar fetch internals
  /// and raw-query escape hatches.
  T? readIndexNullable<T>(int index);

  /// Reads a SQLite/Postgres boolean-compatible column.
  bool readBool(String column);

  /// Reads a nullable SQLite/Postgres boolean-compatible column.
  bool? readBoolNullable(String column);

  /// Reads an ISO-8601 date/time text column and normalizes it to UTC.
  DateTime readDateTime(String column);

  /// Reads a nullable ISO-8601 date/time text column and normalizes it to UTC.
  DateTime? readDateTimeNullable(String column);
}

/// Compatibility helpers for generated code emitted before the Executor/Row
/// naming cleanup.
extension RowCompatibility on Row {
  /// Alias for [readNullable].
  T? readOrNull<T>(String column) => readNullable<T>(column);

  /// Alias for [readIndexNullable].
  T? readIndexOrNull<T>(int index) => readIndexNullable<T>(index);

  /// Alias for [readBoolNullable].
  bool? readBoolOrNull(String column) => readBoolNullable(column);

  /// Alias for [readDateTimeNullable].
  DateTime? readDateTimeOrNull(String column) => readDateTimeNullable(column);
}
