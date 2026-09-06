import 'dart:convert';
import 'dart:typed_data';

import 'package:dust_dart/db.dart';
import 'package:sqlite3/sqlite3.dart' as sqlite;

import 'placeholders.dart';

part 'connect_options.dart';
part 'errors.dart';
part 'migrations.dart';
part 'operations.dart';
part 'row.dart';
part 'transaction.dart';
part 'statement_cache.dart';
part 'unsafe_sql.dart';

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
    _ConnectionState? connection,
    _TransactionScope? transactionScope,
  })  : _ownsDatabase = ownsDatabase,
        _connection = connection ?? _ConnectionState(),
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
  final _ConnectionState _connection;
  final _TransactionScope? _transactionScope;
  var _closed = false;

  @override
  sqlite.Database get database => _database;

  @override
  Driver get driver => Driver.sqlite3;

  @override
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    return _selectResult(sql, parameters).match(
      ok: (result) {
        if (result.isEmpty) return Ok<T?, SqlxError>(null);
        return _mapRow<T?>(sql, _firstRow(result), (row) => mapper(row));
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
    return _selectResult(sql, parameters).match(
      ok: (result) {
        try {
          // Mapped straight out of the result's own row data: the adapter is
          // what the mapper reads through, and nothing keeps it afterwards.
          final names = result.columnNames;
          final index = sqliteColumnIndex(names);
          return Ok<List<T>, SqlxError>(<T>[
            for (final data in result.rows)
              mapper(Sqlite3Row._shared(data, names, index)),
          ]);
        } on SqlxError catch (error) {
          return Err<List<T>, SqlxError>(error);
        } catch (error) {
          return Err<List<T>, SqlxError>(
            _sqliteDecodeError(
              'SQLite row decode failed.',
              cause: error,
              operation: sql,
            ),
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
    return _selectResult(sql, parameters).match(
      ok: (result) {
        if (result.isEmpty) {
          return Err<T, SqlxError>(_sqliteNoRows(sql));
        }
        if (result.length > 1) {
          return Err<T, SqlxError>(
            _sqliteTooManyRows(expected: 1, actual: result.length, query: sql),
          );
        }
        return _mapRow<T>(sql, _firstRow(result), mapper);
      },
      err: (error) => Err<T, SqlxError>(error),
    );
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) async {
    return _selectResult(sql, parameters).match(
      ok: (result) {
        if (result.isEmpty) {
          if (null is T) return Ok<T, SqlxError>(null as T);
          return Err<T, SqlxError>(_sqliteNoRows(sql));
        }
        if (result.length > 1) {
          return Err<T, SqlxError>(
            _sqliteTooManyRows(expected: 1, actual: result.length, query: sql),
          );
        }
        final row = _firstRow(result);
        try {
          if (null is T) {
            return Ok<T, SqlxError>(row.readIndexNullable<Object?>(0) as T);
          }
          return Ok<T, SqlxError>(row.readIndex<T>(0));
        } on SqlxError catch (error) {
          return Err<T, SqlxError>(error);
        } catch (error) {
          return Err<T, SqlxError>(
            _sqliteDecodeError(
              'SQLite scalar decode failed.',
              cause: error,
              operation: sql,
            ),
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
    Future<Result<T, SqlxError>> Function(Transaction tx) fn,
  ) async {
    return _runTransaction(fn);
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    if (!_ownsDatabase || _closed) return const Ok<Unit, SqlxError>(unit);
    _closed = true;
    try {
      // Before the database, so the cache never holds a statement belonging to
      // a database that is already gone.
      _connection.statements.close();
      _database.close();
      return const Ok<Unit, SqlxError>(unit);
    } catch (error) {
      return Err<Unit, SqlxError>(
        _sqliteConnectionError(
          'SQLite connection close failed.',
          cause: error,
          operation: 'close',
        ),
      );
    }
  }

  /// Runs [sql], reporting a failure as a value rather than a throw.
  Result<sqlite.ResultSet, SqlxError> _selectResult(
    String sql,
    List<Object?> parameters,
  ) {
    try {
      return Ok<sqlite.ResultSet, SqlxError>(
        _selectUnchecked(sql, parameters),
      );
    } on SqlxError catch (error) {
      return Err<sqlite.ResultSet, SqlxError>(error);
    } on PlaceholderBindError catch (error) {
      return Err<sqlite.ResultSet, SqlxError>(
        _sqliteQueryError(error.message, operation: error.sql),
      );
    } catch (error) {
      return Err<sqlite.ResultSet, SqlxError>(
        _sqliteQueryError(
          'SQLite query failed.',
          cause: error,
          operation: sql,
        ),
      );
    }
  }

  /// Wraps the first row of [result] for a one-row terminal.
  ///
  /// No index is built here. There is one row, so there is nothing to share it
  /// with, and the adapter builds one only if a name is actually read —
  /// `fetchScalar` reads column zero and never needs one.
  Sqlite3Row _firstRow(sqlite.ResultSet result) {
    return Sqlite3Row._shared(result.rows.first, result.columnNames, null);
  }

  /// Runs [sql] and wraps every row, for callers that need them all as rows.
  ///
  /// Only unchecked SQL does: it has no mapper, so the rows are what it
  /// returns. The typed terminals wrap what they read instead.
  Result<List<Row>, SqlxError> _queryResult(
    String sql,
    List<Object?> parameters,
  ) {
    return _selectResult(sql, parameters).map((result) {
      final names = result.columnNames;
      final index = sqliteColumnIndex(names);
      return <Row>[
        for (final data in result.rows) Sqlite3Row._shared(data, names, index),
      ];
    });
  }

  Result<ExecResult, SqlxError> _executeResult(
    String sql,
    List<Object?> parameters,
  ) {
    try {
      return Ok<ExecResult, SqlxError>(
        _executeUnchecked(sql, parameters),
      );
    } on SqlxError catch (error) {
      return Err<ExecResult, SqlxError>(error);
    } on PlaceholderBindError catch (error) {
      return Err<ExecResult, SqlxError>(
        _sqliteQueryError(error.message, operation: error.sql),
      );
    } catch (error) {
      return Err<ExecResult, SqlxError>(
        _sqliteQueryError(
          'SQLite execute failed.',
          cause: error,
          operation: sql,
        ),
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
        _sqliteDecodeError(
          'SQLite row decode failed.',
          cause: error,
          operation: sql,
        ),
      );
    }
  }

  void _checkOpen() {
    final error = _closedError();
    if (error != null) throw error;
  }

  SqlxError? _closedError() {
    if (_closed) {
      return _sqliteConnectionError(
        'SQLite connection is closed.',
        operation: 'checkOpen',
      );
    }
    final scope = _transactionScope;
    if (scope != null && !scope.active) {
      return _sqliteTransactionError(
        'SQLite transaction is closed.',
        operation: 'checkTransactionOpen',
      );
    }
    return null;
  }
}

/// The SQLite pool, named as `sqlx-sqlite` names it.
///
/// SQLx separates `SqlitePool` from `SqliteConnection` because a pool hands out
/// connections. This driver holds one connection and is both, so there is one
/// type here under the name callers reach for.
typedef SqlitePool = Sqlite3Driver;
