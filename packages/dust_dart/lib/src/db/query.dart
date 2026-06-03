import 'pool.dart';
import 'result.dart';
import 'row_mapper.dart';

/// Typed row query.
final class QueryAs<T> {
  const QueryAs(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<T> fetchOne(SqlxDriver db) async {
    return _unwrap(await db.fetchOne<T>(sql, parameters, RowMapperRegistry.map<T>));
  }

  Future<T?> fetchOptional(SqlxDriver db) async {
    return _unwrap(await db.fetchOptional<T>(sql, parameters, RowMapperRegistry.map<T>));
  }

  Future<List<T>> fetchAll(SqlxDriver db) async {
    return _unwrap(await db.fetchAll<T>(sql, parameters, RowMapperRegistry.map<T>));
  }
}

/// Scalar query returning the first selected column.
final class QueryScalar<T> {
  const QueryScalar(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<T> fetchOne(SqlxDriver db) async {
    return _unwrap(await db.fetchScalar<T>(sql, parameters));
  }

  Future<T?> fetchOptional(SqlxDriver db) async {
    return _unwrap(await db.fetchScalar<T?>(sql, parameters));
  }
}

/// Untyped row query.
final class QueryRaw {
  const QueryRaw(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<List<Row>> fetch(SqlxDriver db) async {
    return _unwrap(await db.raw.fetch(sql, parameters));
  }
}

/// Statement query.
final class QueryExecute {
  const QueryExecute(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<ExecResult> execute(SqlxDriver db) async {
    return _unwrap(await db.execute(sql, parameters));
  }
}

QueryAs<T> queryAs<T>(String sql, List<Object?> parameters) {
  return QueryAs<T>(sql, parameters);
}

QueryScalar<T> queryScalar<T>(String sql, List<Object?> parameters) {
  return QueryScalar<T>(sql, parameters);
}

QueryRaw queryRaw(String sql, List<Object?> parameters) {
  return QueryRaw(sql, parameters);
}

QueryExecute queryExecute(String sql, List<Object?> parameters) {
  return QueryExecute(sql, parameters);
}

T _unwrap<T>(Result<T, SqlxError> result) {
  return result.match(
    ok: (value) => value,
    err: (error) => throw StateError('SQL operation failed: $error'),
  );
}
