part of 'sqlite_pool.dart';

final class _SqliteRawSql implements RawSql {
  const _SqliteRawSql(this._driver);

  final Sqlite3Driver _driver;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  ) async {
    return _driver._queryResult(sql, parameters);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    return _driver._executeResult(sql, parameters);
  }
}
