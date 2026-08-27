import '../fp/result.dart';
import 'exec_result.dart';
import 'pool.dart';
import 'row_mapper.dart';
import 'sqlx_error.dart';

/// Typed row query.
final class QueryAs<T> {
  /// Creates one typed row query.
  ///
  /// [using] takes the generated row deserializer, `const $TFromRow()`, which
  /// is what makes a missing row mapping an analyzer error rather than a
  /// runtime one. [mapper] is the plain-function escape hatch for a row type
  /// Dust did not generate.
  const QueryAs(
    this.sql,
    this.parameters, {
    RowMapper<T>? mapper,
    RowDeserializer<T>? using,
  })  : _mapper = mapper,
        _using = using;

  /// Static SQL source.
  final String sql;

  /// Positional SQL parameter values.
  final List<Object?> parameters;

  final RowMapper<T>? _mapper;

  final RowDeserializer<T>? _using;

  /// Fetches exactly one row and maps it as [T].
  Future<T> fetchOne(DatabaseExecutor db) async {
    return _unwrap(await db.fetchOne<T>(sql, parameters, _rowMapper));
  }

  /// Fetches zero or one row and maps it as [T] when present.
  Future<T?> fetchOptional(DatabaseExecutor db) async {
    return _unwrap(await db.fetchOptional<T>(sql, parameters, _rowMapper));
  }

  /// Fetches all rows and maps each as [T].
  Future<List<T>> fetchAll(DatabaseExecutor db) async {
    return _unwrap(await db.fetchAll<T>(sql, parameters, _rowMapper));
  }

  /// Returns a copy that maps rows with [mapper].
  QueryAs<T> withMapper(RowMapper<T> mapper) {
    return QueryAs<T>(sql, parameters, mapper: mapper);
  }

  /// Returns a copy that maps rows with [using].
  QueryAs<T> withDeserializer(RowDeserializer<T> using) {
    return QueryAs<T>(sql, parameters, using: using);
  }

  // An explicit mapper wins over an explicit deserializer, and both win over
  // the registry, which is the only one of the three that can fail at runtime.
  RowMapper<T> get _rowMapper =>
      _mapper ?? _using?.deserialize ?? RowMapperRegistry.map<T>;
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
  Future<T> fetchOne(DatabaseExecutor db) async {
    return _unwrap(await db.fetchScalar<T>(sql, parameters));
  }

  /// Fetches zero or one scalar value.
  Future<T?> fetchOptional(DatabaseExecutor db) async {
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
  Future<ExecResult> execute(DatabaseExecutor db) async {
    return _unwrap(await db.execute(sql, parameters));
  }
}

/// Creates a typed row query helper.
QueryAs<T> queryAs<T>(
  String sql,
  List<Object?> parameters, {
  RowMapper<T>? mapper,
  RowDeserializer<T>? using,
}) {
  return QueryAs<T>(sql, parameters, mapper: mapper, using: using);
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
