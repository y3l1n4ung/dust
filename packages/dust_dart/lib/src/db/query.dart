import '../fp/result.dart';
import 'exec_result.dart';
import 'pool.dart';
import 'row_mapper.dart';
import 'sqlx_error.dart';

/// Typed row query.
final class QueryAs<T> {
  /// Creates one typed row query.
  const QueryAs(this.sql, this.parameters);

  /// Static SQL source.
  final String sql;

  /// Positional SQL parameter values.
  final List<Object?> parameters;

  /// Fetches exactly one row, decoding it with [mapper].
  ///
  /// `@Derive([FromRow()])` generates `fetchOne` on top of this, so a row type
  /// Dust owns needs no mapper at the call. Reach for this directly only for a
  /// row type Dust does not generate.
  Future<Result<T, SqlxError>> fetchOneWith(
    DatabaseExecutor db,
    RowMapper<T> mapper,
  ) {
    return db.fetchOne<T>(sql, parameters, mapper);
  }

  /// Fetches zero or one row, decoding it with [mapper] when present.
  Future<Result<T?, SqlxError>> fetchOptionalWith(
    DatabaseExecutor db,
    RowMapper<T> mapper,
  ) {
    return db.fetchOptional<T>(sql, parameters, mapper);
  }

  /// Fetches every row, decoding each with [mapper].
  Future<Result<List<T>, SqlxError>> fetchAllWith(
    DatabaseExecutor db,
    RowMapper<T> mapper,
  ) {
    return db.fetchAll<T>(sql, parameters, mapper);
  }
}

/// Scalar query returning the first selected column.
final class QueryScalar<T> {
  /// Creates one scalar query.
  const QueryScalar(this.sql, this.parameters);

  /// Static SQL source.
  final String sql;

  /// Positional SQL parameter values.
  final List<Object?> parameters;

  /// Fetches exactly one scalar value.
  Future<Result<T, SqlxError>> fetchOne(DatabaseExecutor db) {
    return db.fetchScalar<T>(sql, parameters);
  }

  /// Fetches zero or one scalar value.
  Future<Result<T?, SqlxError>> fetchOptional(DatabaseExecutor db) {
    return db.fetchScalar<T?>(sql, parameters);
  }
}

/// Statement query.
final class QueryExecute {
  /// Creates one execute statement query.
  const QueryExecute(this.sql, this.parameters);

  /// Static SQL source.
  final String sql;

  /// Positional SQL parameter values.
  final List<Object?> parameters;

  /// Executes this statement and returns execution metadata.
  Future<Result<ExecResult, SqlxError>> execute(DatabaseExecutor db) {
    return db.execute(sql, parameters);
  }
}

/// Creates a typed row query helper.
QueryAs<T> queryAs<T>(String sql, List<Object?> parameters) {
  return QueryAs<T>(sql, parameters);
}

/// Creates a scalar query helper.
QueryScalar<T> queryScalar<T>(String sql, List<Object?> parameters) {
  return QueryScalar<T>(sql, parameters);
}

/// Creates an execute statement query helper.
QueryExecute queryExecute(String sql, List<Object?> parameters) {
  return QueryExecute(sql, parameters);
}
