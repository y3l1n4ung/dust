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

  /// Fetches exactly one row and maps it as [T].
  Future<T> fetchOne(Executor db) async {
    return _unwrap(
      await db.fetchOne<T>(sql, parameters, RowMapperRegistry.map<T>),
    );
  }

  /// Fetches zero or one row and maps it as [T] when present.
  Future<T?> fetchOptional(Executor db) async {
    return _unwrap(
      await db.fetchOptional<T>(sql, parameters, RowMapperRegistry.map<T>),
    );
  }

  /// Fetches all rows and maps each as [T].
  Future<List<T>> fetchAll(Executor db) async {
    return _unwrap(
      await db.fetchAll<T>(sql, parameters, RowMapperRegistry.map<T>),
    );
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
  Future<T> fetchOne(Executor db) async {
    return _unwrap(await db.fetchScalar<T>(sql, parameters));
  }

  /// Fetches zero or one scalar value.
  Future<T?> fetchOptional(Executor db) async {
    return _unwrap(await db.fetchScalar<T?>(sql, parameters));
  }
}

/// Untyped row query.
final class QueryRaw {
  /// Creates one raw row query.
  const QueryRaw(this.sql, this.parameters);

  /// Static SQL source.
  final String sql;

  /// Positional SQL parameter values.
  final List<Object?> parameters;

  /// Fetches raw rows through [Executor.raw].
  Future<List<Row>> fetch(Executor db) async {
    return _unwrap(await db.raw.fetch(sql, parameters));
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
  Future<ExecResult> execute(Executor db) async {
    return _unwrap(await db.execute(sql, parameters));
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

/// Creates a raw row query helper.
QueryRaw queryRaw(String sql, List<Object?> parameters) {
  return QueryRaw(sql, parameters);
}

/// Creates an execute statement query helper.
QueryExecute queryExecute(String sql, List<Object?> parameters) {
  return QueryExecute(sql, parameters);
}

T _unwrap<T>(Result<T, SqlxError> result) {
  return result.match(
    ok: (value) => value,
    err: (error) => throw StateError('SQL operation failed: $error'),
  );
}
