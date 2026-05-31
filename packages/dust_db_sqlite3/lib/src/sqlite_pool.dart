import 'package:dust_db_runtime/dust_db_runtime.dart';
import 'package:sqlite3/sqlite3.dart' as sqlite;

/// SQLite pool backed by one `package:sqlite3` database connection.
final class SqlitePool implements Pool {
  SqlitePool._(this._database);

  /// Opens a database at [path] and applies [migrations] in map order.
  factory SqlitePool.open(
    String path, {
    Map<String, String> migrations = const <String, String>{},
  }) {
    final database = sqlite.sqlite3.open(path);
    for (final migration in migrations.entries) {
      database.execute(migration.value);
    }
    return SqlitePool._(database);
  }

  final sqlite.Database _database;
  var _closed = false;

  @override
  Driver get driver => Driver.sqlite3;

  @override
  RawSql get raw => _SqliteRawSql(this);

  @override
  Future<Result<List<Row>, SqlxError>> fetch(String sql, List<Object?> parameters) async {
    try {
      return Ok<List<Row>, SqlxError>(_queryUnchecked(sql, parameters));
    } catch (error) {
      return Err<List<Row>, SqlxError>(
        SqlxDriverError('SQLite query failed.', cause: error),
      );
    }
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(String sql, List<Object?> parameters) async {
    try {
      return Ok<ExecResult, SqlxError>(
        ExecResult(rowsAffected: _executeUnchecked(sql, parameters)),
      );
    } catch (error) {
      return Err<ExecResult, SqlxError>(
        SqlxDriverError('SQLite execute failed.', cause: error),
      );
    }
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  ) async {
    _checkOpen();
    _database.execute('BEGIN');
    final tx = SqliteTransaction._(_database);
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
        SqlxDriverError('SQLite transaction failed.', cause: error),
      );
    }
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    if (_closed) return const Ok<Unit, SqlxError>(unit);
    _closed = true;
    try {
      _database.close();
      return const Ok<Unit, SqlxError>(unit);
    } catch (error) {
      return Err<Unit, SqlxError>(
        SqlxDriverError('SQLite close failed.', cause: error),
      );
    }
  }

  void _checkOpen() {
    if (_closed) {
      throw StateError('SqlitePool is closed.');
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

  int _executeUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final prepared = rewriteOrdinalPlaceholdersForSqlite(sql, parameters);
    final statement = _database.prepare(prepared.sql);
    try {
      statement.execute(prepared.parameters);
      return _database.updatedRows;
    } finally {
      statement.close();
    }
  }
}

/// SQLite transaction-scoped pool.
final class SqliteTransaction implements Transaction {
  SqliteTransaction._(this._database);

  final sqlite.Database _database;

  @override
  Driver get driver => Driver.sqlite3;

  @override
  RawSql get raw => _SqliteRawSql(this);

  @override
  Future<Result<List<Row>, SqlxError>> fetch(String sql, List<Object?> parameters) async {
    try {
      return Ok<List<Row>, SqlxError>(_queryUnchecked(sql, parameters));
    } catch (error) {
      return Err<List<Row>, SqlxError>(
        SqlxDriverError('SQLite query failed.', cause: error),
      );
    }
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(String sql, List<Object?> parameters) async {
    try {
      return Ok<ExecResult, SqlxError>(
        ExecResult(rowsAffected: _executeUnchecked(sql, parameters)),
      );
    } catch (error) {
      return Err<ExecResult, SqlxError>(
        SqlxDriverError('SQLite execute failed.', cause: error),
      );
    }
  }

  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(SqlxDriver tx) fn,
  ) {
    return fn(this);
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    return const Ok<Unit, SqlxError>(unit);
  }

  List<Row> _queryUnchecked(String sql, List<Object?> parameters) {
    final prepared = rewriteOrdinalPlaceholdersForSqlite(sql, parameters);
    final result = _database.select(prepared.sql, prepared.parameters);
    return <Row>[
      for (final row in result) SqliteRow(row),
    ];
  }

  int _executeUnchecked(String sql, List<Object?> parameters) {
    final prepared = rewriteOrdinalPlaceholdersForSqlite(sql, parameters);
    final statement = _database.prepare(prepared.sql);
    try {
      statement.execute(prepared.parameters);
      return _database.updatedRows;
    } finally {
      statement.close();
    }
  }
}

final class _SqliteRawSql implements RawSql {
  const _SqliteRawSql(this._pool);

  final SqlxDriver _pool;

  @override
  Future<Result<List<Row>, SqlxError>> fetch(String sql, List<Object?> parameters) {
    return _pool.fetch(sql, parameters);
  }

  @override
  Future<Result<ExecResult, SqlxError>> execute(String sql, List<Object?> parameters) {
    return _pool.execute(sql, parameters);
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
      throw StateError('Column `$column` is null.');
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
      throw StateError('Column index `$index` is null.');
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
      throw StateError('Column `$column` is null.');
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
    throw StateError('Column `$column` cannot be read as bool.');
  }

  @override
  DateTime readDateTime(String column) {
    final value = readDateTimeOrNull(column);
    if (value == null) {
      throw StateError('Column `$column` is null.');
    }
    return value;
  }

  @override
  DateTime? readDateTimeOrNull(String column) {
    final value = _row[column];
    if (value == null) return null;
    if (value is DateTime) return value;
    if (value is String) return DateTime.parse(value);
    throw StateError('Column `$column` cannot be read as DateTime.');
  }

  T? _coerce<T>(Object? value) {
    if (value == null) return null;
    if (T == double && value is int) return value.toDouble() as T;
    if (T == num && value is num) return value as T;
    return value as T;
  }
}
