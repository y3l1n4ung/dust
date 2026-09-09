import 'annotations.dart';
import '../fp/result.dart';
import '../fp/unit.dart';
import 'exec_result.dart';
import 'row_mapper.dart';
import 'sqlx_error.dart';
import 'unsafe_sql.dart';

/// Executes typed SQLx-style queries against a database connection,
/// transaction, or driver.
///
/// Generated DAO code receives a [Executor] and calls `fetchOptional`,
/// `fetchAll`, `fetchOne`, `fetchScalar`, or `execute` depending on the
/// annotated method return type.
abstract interface class Executor {
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
    Future<Result<T, SqlxError>> Function(Transaction tx) fn,
  );

  /// Closes resources owned by this driver.
  Future<Result<Unit, SqlxError>> close();
}

/// Open generated application database facade.
abstract interface class DatabaseClient {
  /// Open database connection used by generated DAOs.
  Connection get connection;

  /// Applies any migrations the database was generated with.
  ///
  /// SQLite applies them while opening, so this has already happened and the
  /// call returns `Ok`. PostgreSQL is reached over a network and cannot, so it
  /// applies them here. Calling it either way is what lets one startup path
  /// serve both.
  Future<Result<Unit, SqlxError>> migrate();

  /// Unchecked SQL for administrative work.
  ///
  /// Deliberately here and not on [Executor]: a request handler holds
  /// an executor, and no cast takes an executor to a [DatabaseClient], so the
  /// escape hatch is out of reach from a handler by type rather than by
  /// convention.
  UnsafeSql get unsafe;
}

/// Convenience methods for generated application database facades.
extension DatabaseClientExecution on DatabaseClient {
  /// Typed query executor for this database.
  Executor get executor => connection;

  /// Runs [fn] inside a database transaction.
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(Transaction tx) fn,
  ) {
    return connection.transaction(fn);
  }

  /// Closes resources owned by this database connection.
  Future<Result<Unit, SqlxError>> close() {
    return connection.close();
  }
}

/// An open database connection.
///
/// SQLx's `Connection`.
abstract interface class Connection implements Executor {}

/// A transaction-scoped executor.
///
/// SQLx's `Transaction<'_, DB>`, scoped to a closure rather than to a guard:
/// Rust's `Drop` rolls an uncommitted transaction back and Dart has no
/// destructors, so the closure is the only shape that is safe by construction.
abstract interface class Transaction implements Executor {}

/// A long-lived database pool.
///
/// SQLx's `Pool<DB>`. Driver packages name their own — `SqlitePool`.
abstract interface class Pool implements Connection {}

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
