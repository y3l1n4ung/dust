import 'dart:async';

import 'package:dust_dart/db.dart';
import 'package:postgres/postgres.dart' as pg;

part 'connect_options.dart';
part 'connection_url.dart';
part 'driver.dart';
part 'errors.dart';
part 'migrations.dart';
part 'row.dart';
part 'statement_cache.dart';
part 'transaction.dart';
part 'unsafe_sql.dart';

/// A PostgreSQL executor with access to the underlying driver session.
abstract interface class PostgresExecutor implements Executor {
  /// Native `package:postgres` session used by this executor.
  pg.Session get session;
}

/// Runs Dust queries against one `package:postgres` session.
///
/// The SQL reaches the server unchanged. Postgres reads `$1` natively, and
/// values bind with an unspecified type so the server infers them, which is why
/// a plain `List<Object?>` works against a plain `String` query and why nothing
/// on this side rewrites the text. SQLite needs the opposite: its driver
/// rewrites `$n` to `?` at bind time.
abstract base class _PostgresSession implements PostgresExecutor {
  /// Binds queries to a session, and transactions to a session executor.
  ///
  /// The session executor is null inside a transaction, where the driver hands
  /// out a `TxSession` that cannot open a transaction of its own.
  _PostgresSession(this._session, this._sessionExecutor);

  final pg.Session _session;
  final pg.SessionExecutor? _sessionExecutor;

  @override
  pg.Session get session => _session;

  @override
  Driver get driver => Driver.postgres;

  /// Runs [sql] and returns the driver's own result.
  ///
  /// The driver overrides this: it owns the pool, so it can hold a prepared
  /// statement per connection. A transaction cannot — its statements would be
  /// parsed and thrown away with it.
  Future<pg.Result> _run(String sql, List<Object?> parameters) {
    return _session.execute(sql, parameters: parameters);
  }

  /// Runs [sql], reporting a failure as a value rather than a throw.
  Future<Result<pg.Result, SqlxError>> _result(
    String sql,
    List<Object?> parameters,
  ) async {
    try {
      return Ok<pg.Result, SqlxError>(await _run(sql, parameters));
    } catch (error) {
      return Err<pg.Result, SqlxError>(_asPostgresError(error, sql));
    }
  }

  /// Decodes every row of [result] with [mapper].
  Result<List<T>, SqlxError> _decode<T>(
    pg.Result result,
    RowMapper<T> mapper,
    String sql,
  ) {
    try {
      // One index for the whole result rather than one map per row, and mapped
      // straight out of it rather than through a list of adapters.
      final index = postgresColumnIndex(result.schema);
      return Ok<List<T>, SqlxError>(<T>[
        for (final row in result) mapper(PostgresRow._shared(row, index, sql)),
      ]);
    } on SqlxError catch (error) {
      return Err<List<T>, SqlxError>(error);
    } catch (error) {
      return Err<List<T>, SqlxError>(
        _postgresDecodeError(
          'PostgreSQL row mapping failed.',
          cause: error,
          operation: sql,
        ),
      );
    }
  }

  /// Decodes the single row of [result] with [mapper].
  Result<T, SqlxError> _decodeOne<T>(
    pg.Result result,
    RowMapper<T> mapper,
    String sql,
  ) {
    try {
      return Ok<T, SqlxError>(mapper(_singleRow(result, sql)));
    } on SqlxError catch (error) {
      return Err<T, SqlxError>(error);
    } catch (error) {
      return Err<T, SqlxError>(
        _postgresDecodeError(
          'PostgreSQL row mapping failed.',
          cause: error,
          operation: sql,
        ),
      );
    }
  }

  /// Wraps the single row of [result] for a one-row terminal.
  ///
  /// No index is built here. There is one row, so there is nothing to share it
  /// with, and the adapter builds one only if a name is actually read —
  /// `fetchScalar` reads column zero and never needs one.
  PostgresRow _singleRow(pg.Result result, String sql) {
    return PostgresRow(result.single, operation: sql);
  }

  @override
  Future<Result<T, SqlxError>> fetchOne<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final result = await _result(sql, parameters);
    return result.andThen((result) {
      if (result.isEmpty) return Err<T, SqlxError>(_postgresNoRows(sql));
      if (result.length > 1) {
        return Err<T, SqlxError>(
          _postgresTooManyRows(expected: 1, actual: result.length, query: sql),
        );
      }
      return _decodeOne(result, mapper, sql);
    });
  }

  @override
  Future<Result<T?, SqlxError>> fetchOptional<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final result = await _result(sql, parameters);
    return result.andThen((result) {
      if (result.isEmpty) return Ok<T?, SqlxError>(null);
      if (result.length > 1) {
        return Err<T?, SqlxError>(
          _postgresTooManyRows(expected: 1, actual: result.length, query: sql),
        );
      }
      return _decodeOne(result, mapper, sql).map<T?>((value) => value);
    });
  }

  @override
  Future<Result<List<T>, SqlxError>> fetchAll<T>(
    String sql,
    List<Object?> parameters,
    RowMapper<T> mapper,
  ) async {
    final result = await _result(sql, parameters);
    return result.andThen((result) => _decode(result, mapper, sql));
  }

  @override
  Future<Result<T, SqlxError>> fetchScalar<T>(
    String sql,
    List<Object?> parameters,
  ) async {
    final result = await _result(sql, parameters);
    return result.andThen((result) {
      // A nullable T is what `QueryScalar.fetchOptional` asks for, so no row
      // and a NULL value are both answers rather than failures. A non-nullable
      // one keeps saying so: the caller declared the value has to be there.
      final nullable = null is T;
      if (result.isEmpty) {
        if (nullable) return Ok<T, SqlxError>(null as T);
        return Err<T, SqlxError>(_postgresNoRows(sql));
      }
      if (result.length > 1) {
        return Err<T, SqlxError>(
          _postgresTooManyRows(expected: 1, actual: result.length, query: sql),
        );
      }
      final row = _singleRow(result, sql);
      try {
        if (nullable) {
          return Ok<T, SqlxError>(row.readIndexNullable<Object?>(0) as T);
        }
        return Ok<T, SqlxError>(row.readIndex<T>(0));
      } on SqlxError catch (error) {
        return Err<T, SqlxError>(error);
      }
    });
  }

  /// Runs a statement and reports how many rows it changed.
  ///
  /// There is no `lastInsertId`: Postgres has no counterpart to SQLite's
  /// `last_insert_rowid()`, so a caller that needs the new row asks for it with
  /// `RETURNING` and reads it like any other query.
  @override
  Future<Result<ExecResult, SqlxError>> execute(
    String sql,
    List<Object?> parameters,
  ) async {
    try {
      final result = await _session.execute(sql, parameters: parameters);
      return Ok<ExecResult, SqlxError>(
        ExecResult(rowsAffected: result.affectedRows),
      );
    } catch (error) {
      return Err<ExecResult, SqlxError>(_asPostgresError(error, sql));
    }
  }

  /// Runs [fn] in a transaction, committing on `Ok` and reverting otherwise.
  ///
  /// A nested call becomes a savepoint on the enclosing transaction, because a
  /// `TxSession` cannot open one of its own.
  @override
  Future<Result<T, SqlxError>> transaction<T>(
    Future<Result<T, SqlxError>> Function(Transaction tx) fn,
  ) async {
    final executor = _sessionExecutor;
    if (executor == null) return _runInSavepoint(_session, fn);

    try {
      return await executor.runTx<Result<T, SqlxError>>((tx) async {
        final result = await fn(_PostgresTransaction(tx));
        // `runTx` reverts only when the callback throws, and an `Err` is an
        // ordinary return value, so it has to leave as a throw.
        if (result.isErr) throw _RollbackSignal(result);
        return result;
      });
    } on _RollbackSignal catch (signal) {
      return signal.result as Result<T, SqlxError>;
    } catch (error) {
      return Err<T, SqlxError>(
        _postgresTransactionError(
          'PostgreSQL transaction failed.',
          cause: error,
          operation: 'transaction',
        ),
      );
    }
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    try {
      await _sessionExecutor?.close();
      return const Ok<Unit, SqlxError>(unit);
      // `package:postgres` does not fail this call today: closing an already
      // closed pool returns normally, and so does closing one with a statement
      // in flight — both were tried. The guard stays because a driver that
      // starts throwing should not turn into an unhandled exception here, and
      // it is excluded rather than covered by a test that cannot be written.
      // coverage:ignore-start
    } catch (error) {
      return Err<Unit, SqlxError>(
        _postgresConnectionError(
          'PostgreSQL close failed.',
          cause: error,
          operation: 'close',
        ),
      );
      // coverage:ignore-end
    }
  }
}
