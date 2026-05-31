import 'pool.dart';
import 'row_mapper.dart';

import 'result.dart';

/// Typed row query.
final class QueryAs<T> {
  const QueryAs(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<T> fetchOne(SqlxDriver db) async {
    final rows = _unwrapRows(await db.fetch(sql, parameters));
    if (rows.length != 1) {
      throw StateError('Expected exactly one row, got ${rows.length}.');
    }
    return RowMapperRegistry.map<T>(rows.single);
  }

  Future<T?> fetchOptional(SqlxDriver db) async {
    final rows = _unwrapRows(await db.fetch(sql, parameters));
    if (rows.length > 1) {
      throw StateError('Expected zero or one row, got ${rows.length}.');
    }
    return rows.isEmpty ? null : RowMapperRegistry.map<T>(rows.single);
  }

  Future<List<T>> fetchAll(SqlxDriver db) async {
    final rows = _unwrapRows(await db.fetch(sql, parameters));
    return rows.map(RowMapperRegistry.map<T>).toList();
  }
}

/// Scalar query returning the first selected column.
final class QueryScalar<T> {
  const QueryScalar(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<T> fetchOne(SqlxDriver db) async {
    final rows = _unwrapRows(await db.fetch(sql, parameters));
    if (rows.length != 1) {
      throw StateError('Expected exactly one row, got ${rows.length}.');
    }
    return rows.single.readIndex<T>(0);
  }

  Future<T?> fetchOptional(SqlxDriver db) async {
    final rows = _unwrapRows(await db.fetch(sql, parameters));
    if (rows.length > 1) {
      throw StateError('Expected zero or one row, got ${rows.length}.');
    }
    return rows.isEmpty ? null : rows.single.readIndexOrNull<T>(0);
  }
}

/// Untyped row query.
final class QueryRaw {
  const QueryRaw(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<List<Row>> fetch(SqlxDriver db) async {
    return _unwrapRows(await db.fetch(sql, parameters));
  }
}

/// Statement query.
final class QueryExecute {
  const QueryExecute(this.sql, this.parameters);

  final String sql;
  final List<Object?> parameters;

  Future<ExecResult> execute(SqlxDriver db) async {
    return _unwrapExec(await db.execute(sql, parameters));
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

List<Row> _unwrapRows(Result<List<Row>, SqlxError> result) {
  return result.match(
    ok: (rows) => rows,
    err: (error) => throw StateError('SQL query failed: $error'),
  );
}

ExecResult _unwrapExec(Result<ExecResult, SqlxError> result) {
  return result.match(
    ok: (value) => value,
    err: (error) => throw StateError('SQL execute failed: $error'),
  );
}
