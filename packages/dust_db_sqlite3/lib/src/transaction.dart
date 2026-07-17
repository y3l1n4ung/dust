part of 'sqlite_pool.dart';

final class _TransactionCoordinator {
  var _nextSavepointId = 0;

  Future<Result<T, SqlxError>> runRoot<T>(
    sqlite.Database database,
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) async {
    final begin = _executeControl(
      database,
      'BEGIN',
      'SQLite transaction begin failed.',
    );
    if (begin != null) return Err<T, SqlxError>(begin);

    final tx = _SingleConnectionPool(database, this);
    try {
      final result = await fn(tx);
      return result.match<Future<Result<T, SqlxError>>>(
        ok: (value) async {
          final commit = _executeControl(
            database,
            'COMMIT',
            'SQLite transaction commit failed.',
          );
          if (commit != null) return Err<T, SqlxError>(commit);
          return Ok<T, SqlxError>(value);
        },
        err: (error) async {
          final rollback = _executeControl(
            database,
            'ROLLBACK',
            'SQLite transaction rollback failed.',
          );
          if (rollback != null) return Err<T, SqlxError>(rollback);
          return Err<T, SqlxError>(error);
        },
      );
    } catch (error) {
      final rollback = _executeControl(
        database,
        'ROLLBACK',
        'SQLite transaction rollback failed.',
      );
      if (rollback != null) return Err<T, SqlxError>(rollback);
      return Err<T, SqlxError>(
        SqlxError.driver('SQLite transaction failed.', cause: error),
      );
    } finally {
      tx._deactivate();
    }
  }

  Future<Result<T, SqlxError>> runSavepoint<T>(
    sqlite.Database database,
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) async {
    final name = '_dust_tx_${++_nextSavepointId}';
    final begin = _executeControl(
      database,
      'SAVEPOINT $name',
      'SQLite savepoint begin failed.',
    );
    if (begin != null) return Err<T, SqlxError>(begin);

    final tx = _SingleConnectionPool(database, this);
    try {
      final result = await fn(tx);
      return result.match<Future<Result<T, SqlxError>>>(
        ok: (value) async {
          final release = _executeControl(
            database,
            'RELEASE SAVEPOINT $name',
            'SQLite savepoint release failed.',
          );
          if (release != null) return Err<T, SqlxError>(release);
          return Ok<T, SqlxError>(value);
        },
        err: (error) async {
          final rollback = _rollbackSavepoint(database, name);
          if (rollback != null) return Err<T, SqlxError>(rollback);
          return Err<T, SqlxError>(error);
        },
      );
    } catch (error) {
      final rollback = _rollbackSavepoint(database, name);
      if (rollback != null) return Err<T, SqlxError>(rollback);
      return Err<T, SqlxError>(
        SqlxError.driver('SQLite transaction failed.', cause: error),
      );
    } finally {
      tx._deactivate();
    }
  }

  SqlxError? _rollbackSavepoint(sqlite.Database database, String name) {
    final rollback = _executeControl(
      database,
      'ROLLBACK TO SAVEPOINT $name',
      'SQLite savepoint rollback failed.',
    );
    final release = _executeControl(
      database,
      'RELEASE SAVEPOINT $name',
      'SQLite savepoint release after rollback failed.',
    );
    return rollback ?? release;
  }
}

final class _TransactionScope {
  var active = true;
}

final class _SingleConnectionPool implements Transaction, Sqlite3Executor {
  _SingleConnectionPool(
    sqlite.Database database,
    _TransactionCoordinator transactions,
  ) : this._scoped(database, transactions, _TransactionScope());

  _SingleConnectionPool._scoped(
    sqlite.Database database,
    this._transactions,
    this._scope,
  ) : _driver = Sqlite3Driver._(
          database,
          ownsDatabase: false,
          transactions: _transactions,
          transactionScope: _scope,
        );

  final Sqlite3Driver _driver;
  final _TransactionCoordinator _transactions;
  final _TransactionScope _scope;

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
    final error = _driver._closedError();
    if (error != null) return Future.value(Err<T, SqlxError>(error));
    return _transactions.runSavepoint(database, fn);
  }

  @override
  Future<Result<Unit, SqlxError>> close() {
    _deactivate();
    return Future.value(const Ok<Unit, SqlxError>(unit));
  }

  void _deactivate() {
    _scope.active = false;
  }
}

extension _Sqlite3TransactionRunner on Sqlite3Driver {
  Future<Result<T, SqlxError>> _runTransaction<T>(
    Future<Result<T, SqlxError>> Function(Executor tx) fn,
  ) {
    final error = _closedError();
    if (error != null) return Future.value(Err<T, SqlxError>(error));
    if (!_ownsDatabase) return _transactions.runSavepoint(database, fn);
    return _transactions.runRoot(database, fn);
  }
}

SqlxError? _executeControl(
  sqlite.Database database,
  String sql,
  String message,
) {
  try {
    database.execute(sql);
    return null;
  } catch (error) {
    return SqlxError.driver(message, cause: error);
  }
}
