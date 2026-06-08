part of 'sqlite_pool.dart';

final class _SingleConnectionPool implements Transaction, Sqlite3Executor {
  _SingleConnectionPool(sqlite.Database database)
    : _driver = Sqlite3Driver._(database, ownsDatabase: false);

  final Sqlite3Driver _driver;

  @override
  sqlite.Database get database => _driver.database;

  @override
  Driver get driver => _driver.driver;

  @override
  RawSql get raw => _driver.raw;

  @override
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) {
    return _driver.fetchOptional(sql, parameters, mapper);
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAll<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) {
    return _driver.fetchAll(sql, parameters, mapper);
  }

  @override
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) {
    return _driver.fetchOne(sql, parameters, mapper);
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) {
    return _driver.fetchScalar(sql, parameters);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) {
    return _driver.execute(sql, parameters);
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) {
    return fn(this);
  }

  @override
  Future<Result<Unit, SqlxError>> close() {
    return _driver.close();
  }
}
