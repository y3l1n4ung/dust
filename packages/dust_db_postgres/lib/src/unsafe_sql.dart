part of 'postgres_pool.dart';

/// Unchecked SQL over one PostgreSQL session.
///
/// A generated database facade exposes this as `unsafe`. It is not reachable
/// from an executor, so a request handler cannot get here.
final class PostgresUnsafeSql implements UnsafeSql {
  /// Wraps one open driver for unchecked access.
  const PostgresUnsafeSql(this._executor);

  final PostgresExecutor _executor;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(
    String sql,
    List<Object?> parameters,
  ) async {
    return _executor.fetchAll<Row>(sql, parameters, (row) => row);
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAs<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final rows = await fetch(sql, parameters);
    return rows.andThen((rows) {
      try {
        return Ok<List<T>, SqlxError>(<T>[for (final row in rows) mapper(row)]);
      } on SqlxError catch (error) {
        return Err<List<T>, SqlxError>(error);
      } catch (error) {
        return Err<List<T>, SqlxError>(
          _postgresDecodeError(
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
  ) {
    return _executor.execute(sql, parameters);
  }
}
