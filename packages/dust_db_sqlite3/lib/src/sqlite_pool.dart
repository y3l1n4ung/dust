import 'package:dust_dart/db.dart';
import 'package:sqlite3/sqlite3.dart' as sqlite;

part 'connect_options.dart';
part 'migrations.dart';
part 'raw_sql.dart';
part 'row.dart';
part 'transaction.dart';

/// SQLite-backed executor with access to the underlying native database.
abstract interface class Sqlite3Executor implements Executor {
  /// Native `package:sqlite3` database used by this executor.
  sqlite.Database get database;
}

/// SQLite driver backed by one `package:sqlite3` database connection.
final class Sqlite3Driver implements Pool, Sqlite3Executor {
  Sqlite3Driver._(
    this._database, {
    required bool ownsDatabase,
    _TransactionCoordinator? transactions,
    _TransactionScope? transactionScope,
  })  : _ownsDatabase = ownsDatabase,
        _transactions = transactions ?? _TransactionCoordinator(),
        _transactionScope = transactionScope;

  /// Opens a database at [path] and applies unapplied migrations in name order.
  factory Sqlite3Driver.open(
    String path, {
    Map<String, String> migrations = const <String, String>{},
    SqliteConnectOptions? options,
  }) {
    return Sqlite3Driver.connect(
      (options ?? const SqliteConnectOptions())._withPath(path),
      migrations: migrations,
    );
  }

  /// Opens a database from explicit SQLite connection [options].
  factory Sqlite3Driver.connect(
    SqliteConnectOptions options, {
    Map<String, String> migrations = const <String, String>{},
  }) {
    options._validate(hasMigrations: migrations.isNotEmpty);
    final database = _openDatabase(options);
    try {
      _applyConnectOptions(database, options);
      _applyMigrations(database, migrations);
      return Sqlite3Driver._(database, ownsDatabase: true);
    } catch (_) {
      database.close();
      rethrow;
    }
  }

  final sqlite.Database _database;
  final bool _ownsDatabase;
  final _TransactionCoordinator _transactions;
  final _TransactionScope? _transactionScope;
  var _closed = false;

  @override
  sqlite.Database get database => _database;

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
          return Ok<List<T>, SqlxError>([for (final row in rows) mapper(row)]);
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
            return Ok<T, SqlxError>(
              rows.single.readIndexNullable<Object?>(0) as T,
            );
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
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) async {
    return _runTransaction(fn);
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    if (!_ownsDatabase || _closed) return const Ok<Unit, SqlxError>(unit);
    _closed = true;
    _database.close();
    return const Ok<Unit, SqlxError>(unit);
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

  Result<T, SqlxError> _mapRow<T>(String sql, Row row, RowMapper<T> mapper) {
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
    final error = _closedError();
    if (error != null) throw error;
  }

  SqlxError? _closedError() {
    if (_closed) return SqlxError.driver('SQLite connection is closed.');
    final scope = _transactionScope;
    if (scope != null && !scope.active) {
      return SqlxError.driver('SQLite transaction is closed.');
    }
    return null;
  }

  List<Row> _queryUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final result = _database.select(sql, parameters);
    return <Row>[for (final row in result) Sqlite3Row(row)];
  }

  ExecResult _executeUnchecked(String sql, List<Object?> parameters) {
    _checkOpen();
    final statement = _database.prepare(sql);
    try {
      statement.execute(parameters);
      return ExecResult(
        rowsAffected: _database.updatedRows,
        lastInsertId: _database.lastInsertRowId,
      );
    } finally {
      statement.close();
    }
  }

  static sqlite.Database _openDatabase(SqliteConnectOptions options) {
    try {
      if (options.inMemory) return sqlite.sqlite3.openInMemory();
      return sqlite.sqlite3.open(options.path!, mode: options._openMode);
    } catch (error) {
      throw SqlxError.driver(
        'SQLite database open failed for `${options._label}`.',
        cause: error,
      );
    }
  }

  static void _applyConnectOptions(
    sqlite.Database database,
    SqliteConnectOptions options,
  ) {
    for (final statement in options._pragmaStatements()) {
      try {
        database.execute(statement);
      } catch (error) {
        throw SqlxError.driver(
          'SQLite connection option failed: `$statement`.',
          cause: error,
        );
      }
    }
  }
}

/// Backwards-compatible SQLite pool name.
typedef SqlitePool = Sqlite3Driver;
