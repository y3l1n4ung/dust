import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:sqlite3/sqlite3.dart' as sqlite;

/// SQLite driver backed by one `package:sqlite3` database connection.
final class Sqlite3Driver implements Pool {
  Sqlite3Driver._(this._database, {required bool ownsDatabase})
    : _ownsDatabase = ownsDatabase;

  /// Opens a database at [path] and applies [migrations] in map order.
  factory Sqlite3Driver.open(
    String path, {
    Map<String, String> migrations = const <String, String>{},
  }) {
    final database = sqlite.sqlite3.open(path);
    for (final migration in migrations.entries) {
      database.execute(migration.value);
    }
    return Sqlite3Driver._(database, ownsDatabase: true);
  }

  final sqlite.Database _database;
  final bool _ownsDatabase;
  var _closed = false;

  @override
  Driver get driver => Driver.sqlite3;

  @override
  RawSql get raw => _SqliteRawSql(this);

  @override
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final rows = _queryResult(sql, parameters);
    return rows.match(
      ok: (rows) {
        if (rows.isEmpty) return Ok<T?, SqlxError>(null);
        return _mapRow<T?>(sql, rows.first, (row) => mapper(row));
      },
      err: (error) => Err<T?, SqlxError>(error),
    );
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAll<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final rows = _queryResult(sql, parameters);
    return rows.match(
      ok: (rows) {
        try {
          return Ok<List<T>, SqlxError>([
            for (final row in rows) mapper(row),
          ]);
        } on SqlxError catch (error) {
          return Err<List<T>, SqlxError>(error);
        } catch (error) {
          return Err<List<T>, SqlxError>(
            SqlxError.decode('SQLite row decode failed.', cause: error),
          );
        }
      },
      err: (error) => Err<List<T>, SqlxError>(error),
    );
  }

  @override
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final rows = _queryResult(sql, parameters);
    return rows.match(
      ok: (rows) {
        if (rows.isEmpty) return Err<T, SqlxError>(SqlxError.noRows(sql));
        if (rows.length > 1) {
          return Err<T, SqlxError>(
            SqlxError.tooManyRows(expected: 1, actual: rows.length, query: sql),
          );
        }
        return _mapRow<T>(sql, rows.single, mapper);
      },
      err: (error) => Err<T, SqlxError>(error),
    );
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) async {
    final rows = _queryResult(sql, parameters);
    return rows.match(
      ok: (rows) {
        if (rows.isEmpty) {
          if (null is T) return Ok<T, SqlxError>(null as T);
          return Err<T, SqlxError>(SqlxError.noRows(sql));
        }
        if (rows.length > 1) {
          return Err<T, SqlxError>(
            SqlxError.tooManyRows(expected: 1, actual: rows.length, query: sql),
          );
        }
        try {
          if (null is T) {
            return Ok<T, SqlxError>(rows.single.readIndexOrNull<Object?>(0) as T);
          }
          return Ok<T, SqlxError>(rows.single.readIndex<T>(0));
        } on SqlxError catch (error) {
          return Err<T, SqlxError>(error);
        } catch (error) {
          return Err<T, SqlxError>(
            SqlxError.decode('SQLite scalar decode failed.', cause: error),
          );
        }
      },
      err: (error) => Err<T, SqlxError>(error),
    );
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    return _executeResult(sql, parameters);
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  ) async {
    _checkOpen();
    if (!_ownsDatabase) return fn(this);
    _database.execute('BEGIN');
    final tx = _SingleConnectionPool(_database);
    try {
      final result = await fn(tx);
      return result.match(
        ok: (value) {
          _database.execute('COMMIT');
          return Ok<T, SqlxError>(value);
        },
        err: (error) {
          _database.execute('ROLLBACK');
          return Err<T, SqlxError>(error);
        },
      );
    } catch (error) {
      _database.execute('ROLLBACK');
      return Err<T, SqlxError>(
        SqlxError.driver('SQLite transaction failed.', cause: error),
      );
    }
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    if (!_ownsDatabase || _closed) return const Ok<Unit, SqlxError>(unit);
    _closed = true;
    try {
      _database.close();
      return const Ok<Unit, SqlxError>(unit);
    } catch (error) {
      return Err<Unit, SqlxError>(
        SqlxError.driver('SQLite close failed.', cause: error),
      );
    }
  }

  Result<List<Row>, SqlxError> _queryResult(
    String sql,
    List<Object?> parameters,
  ) {
    try {
      return Ok<List<Row>, SqlxError>(_queryUnchecked(sql, parameters));
    } on SqlxError catch (error) {
      return Err<List<Row>, SqlxError>(error);
    } catch (error) {
      return Err<List<Row>, SqlxError>(
        SqlxError.driver('SQLite query failed.', cause: error),
      );
    }
  }

  Result<ExecResult, SqlxError> _executeResult(
    String sql,
    List<Object?> parameters,
  ) {
    try {
      return Ok<ExecResult, SqlxError>(_executeUnchecked(sql, parameters));
    } on SqlxError catch (error) {
      return Err<ExecResult, SqlxError>(error);
    } catch (error) {
      return Err<ExecResult, SqlxError>(
        SqlxError.driver('SQLite execute failed.', cause: error),
      );
    }
  }

  Result<T, SqlxError> _mapRow<T>(
    String sql,
    Row row,
    RowMapper<T> mapper,
  ) {
    try {
      return Ok<T, SqlxError>(mapper(row));
    } on SqlxError catch (error) {
      return Err<T, SqlxError>(error);
    } catch (error) {
      return Err<T, SqlxError>(
        SqlxError.decode('SQLite row decode failed for `$sql`.', cause: error),
      );
    }
  }

  void _checkOpen() {
    if (_closed) {
      throw StateError('Sqlite3Driver is closed.');
    }
  }

  List<Row> _queryUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final prepared = rewriteOrdinalPlaceholdersForSqlite(sql, parameters);
    final result = _database.select(prepared.sql, prepared.parameters);
    return <Row>[
      for (final row in result) SqliteRow(row),
    ];
  }

  ExecResult _executeUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final prepared = rewriteOrdinalPlaceholdersForSqlite(sql, parameters);
    final statement = _database.prepare(prepared.sql);
    try {
      statement.execute(prepared.parameters);
      return ExecResult(
        rowsAffected: _database.updatedRows,
        lastInsertId: _database.lastInsertRowId,
      );
    } finally {
      statement.close();
    }
  }
}

/// Backwards-compatible SQLite pool name.
typedef SqlitePool = Sqlite3Driver;

final class _SingleConnectionPool implements Transaction {
  _SingleConnectionPool(sqlite.Database database)
    : _driver = Sqlite3Driver._(database, ownsDatabase: false);

  final Sqlite3Driver _driver;

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
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  ) {
    return fn(this);
  }

  @override
  Future<Result<Unit, SqlxError>> close() {
    return _driver.close();
  }
}

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

/// SQLite result row adapter.
final class SqliteRow implements Row {
  /// Creates one row adapter.
  const SqliteRow(this._row);

  final sqlite.Row _row;

  @override
  T read<T>(String column) {
    final value = readOrNull<T>(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
    }
    return value;
  }

  @override
  T? readOrNull<T>(String column) {
    return _coerce<T>(_row[column]);
  }

  @override
  T readIndex<T>(int index) {
    final value = readIndexOrNull<T>(index);
    if (value == null) {
      throw SqlxError.nullColumn('index $index');
    }
    return value;
  }

  @override
  T? readIndexOrNull<T>(int index) {
    return _coerce<T>(_row[index]);
  }

  @override
  bool readBool(String column) {
    final value = readBoolOrNull(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
    }
    return value;
  }

  @override
  bool? readBoolOrNull(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is bool) return value;
    if (value is int) return value != 0;
    if (value is String) return value == 'true' || value == '1';
    throw SqlxError.decode('Column `$column` cannot be read as bool.');
  }

  @override
  DateTime readDateTime(String column) {
    final value = readDateTimeOrNull(column);
    if (value == null) {
      throw SqlxError.nullColumn(column);
    }
    return value;
  }

  @override
  DateTime? readDateTimeOrNull(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is DateTime) return value.toUtc();
    if (value is String) return DateTime.parse(value).toUtc();
    throw SqlxError.decode('Column `$column` cannot be read as DateTime.');
  }

  T? _coerce<T>(Object? value) {
    if (value == null) return null;
    if (T == double && value is int) return value.toDouble() as T;
    if (T == num && value is num) return value as T;
    return value as T;
  }
}
