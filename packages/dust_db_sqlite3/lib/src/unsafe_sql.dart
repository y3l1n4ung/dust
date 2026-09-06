part of 'sqlite_pool.dart';

/// Unchecked SQL over one SQLite driver.
///
/// A generated database facade exposes this as `unsafe`. It is not reachable
/// from an executor, so a request handler cannot get here.
final class Sqlite3UnsafeSql implements UnsafeSql {
  /// Wraps one open SQLite driver for unchecked access.
  const Sqlite3UnsafeSql(this._driver);

  final Sqlite3Driver _driver;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  ) async {
    return _driver._queryResult(sql, parameters);
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAs<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    return _driver._queryResult(sql, parameters).andThen((rows) {
      try {
        return Ok<List<T>, SqlxError>(<T>[for (final row in rows) mapper(row)]);
      } on SqlxError catch (error) {
        return Err<List<T>, SqlxError>(error);
      } catch (error) {
        return Err<List<T>, SqlxError>(
          _sqliteDecodeError(
            'Unchecked SQL row mapping failed.',
            cause: error,
            operation: sql,
          ),
        );
      }
    });
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    return _driver._executeResult(sql, parameters);
  }
}
