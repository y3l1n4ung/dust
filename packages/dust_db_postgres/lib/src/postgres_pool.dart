import 'dart:async';

import 'package:dust_dart/db.dart';
import 'package:postgres/postgres.dart' as pg;

part 'connect_options.dart';
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

  /// Names savepoints uniquely within a process.
  static int _savepointCounter = 0;

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
    if (executor == null) return _savepoint(fn);

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

  /// Runs [fn] inside a savepoint on the transaction already in progress.
  ///
  /// `package:postgres` exposes no savepoint API, but a `TxSession` is a
  /// `Session`, so the statements are available. After a successful
  /// `ROLLBACK TO SAVEPOINT` the driver clears the transaction's stale error,
  /// which is what lets the enclosing transaction still commit.
  Future<Result<T, SqlxError>> _savepoint<T>(
    Future<Result<T, SqlxError>> Function(Transaction tx) fn,
  ) async {
    final name = 'dust_sp_${_savepointCounter++}';
    try {
      await _session.execute('SAVEPOINT $name');
    } catch (error) {
      return Err<T, SqlxError>(
        _postgresTransactionError(
          'PostgreSQL savepoint failed.',
          cause: error,
          operation: 'SAVEPOINT',
        ),
      );
    }

    Result<T, SqlxError> result;
    try {
      result = await fn(_PostgresTransaction(_session));
    } catch (error) {
      await _releaseSavepoint(name, rollback: true);
      return Err<T, SqlxError>(
        _postgresTransactionError(
          'PostgreSQL nested transaction failed.',
          cause: error,
          operation: 'SAVEPOINT',
        ),
      );
    }

    await _releaseSavepoint(name, rollback: result.isErr);
    return result;
  }

  /// Ends a savepoint, either releasing it or rolling back to it.
  Future<void> _releaseSavepoint(String name, {required bool rollback}) async {
    final command =
        rollback ? 'ROLLBACK TO SAVEPOINT $name' : 'RELEASE SAVEPOINT $name';
    try {
      await _session.execute(command);
    } catch (_) {
      // The enclosing transaction owns the outcome from here: if the savepoint
      // cannot be ended the transaction is already failing, and reporting this
      // instead would hide the error that caused it.
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

/// PostgreSQL driver backed by one `package:postgres` pool.
///
/// A pool in `package:postgres` runs statements directly as well as handing out
/// transactions, so one type is both the pool and the connection Dust asks for.
final class PostgresDriver extends _PostgresSession implements Pool {
  PostgresDriver._(pg.Pool<Object?> pool, this._migrations)
      : _pool = pool,
        super(pool, pool);

  final pg.Pool<Object?> _pool;
  final Map<String, String> _migrations;

  /// Opens a pool from a connection URL and applies unapplied migrations.
  ///
  /// The URL is `postgres://user:password@host:port/database`.
  static PostgresDriver connect(
    String url, {
    Map<String, String> migrations = const <String, String>{},
    PgConnectOptions? options,
  }) {
    final pool = pg.Pool<Object?>.withEndpoints(
      <pg.Endpoint>[_endpointFor(url)],
      settings: pg.PoolSettings(
        // Explicit options win; the URL decides when they say nothing.
        sslMode: (options?.sslMode ?? _sslModeFor(url))?._driverMode,
        connectTimeout: options?.connectTimeout,
        queryTimeout: options?.queryTimeout,
        applicationName: options?.applicationName,
        maxConnectionAge: options?.maxConnectionAge,
      ),
    );
    return PostgresDriver._(pool, migrations);
  }

  /// Applies any migrations this driver was opened with.
  ///
  /// Separate from opening because it has to await: `connect` returns
  /// synchronously so a generated facade can hold one without its constructor
  /// becoming a future.
  Future<Result<Unit, SqlxError>> migrate() =>
      _applyMigrations(this, _migrations);

  /// Prepared statements held per pooled connection.
  final _StatementCache _statements = _StatementCache();

  /// Runs [sql] through a statement this connection already parsed.
  ///
  /// `withConnection` rather than the pool's own `execute`, because a prepared
  /// statement belongs to the connection that parsed it. Holding the connection
  /// for the call costs a little and the statement saves much more: measured
  /// against a local server, 1023us a call became 321us.
  @override
  Future<pg.Result> _run(String sql, List<Object?> parameters) {
    return _pool.withConnection(
      (connection) => _statements.run(connection, sql, parameters),
    );
  }

  @override
  Future<Result<Unit, SqlxError>> close() async {
    // Before the pool, so the cache never holds a statement belonging to a
    // connection that is already gone.
    await _statements.close();
    return super.close();
  }

  /// Unchecked SQL, for the administrative work validation cannot reach.
  UnsafeSql get unsafe => PostgresUnsafeSql(this);

  /// The underlying driver pool, for driver-specific work Dust does not wrap.
  pg.Pool<Object?> get pool => _pool;
}

/// Reads `?sslmode=` from a connection URL.
///
/// Every PostgreSQL tool carries the setting there — libpq, `psql`, SQLx — so a
/// URL that works elsewhere works here. Returns null when the URL says nothing,
/// leaving the driver's own default in charge.
///
/// `prefer` and `allow` are libpq modes this driver has no equivalent for. They
/// mean "try TLS, fall back to plaintext", and mapping them to either side
/// would silently decide something the caller asked to have decided per
/// connection, so they are rejected rather than guessed.
PgSslMode? _sslModeFor(String url) {
  final value = Uri.parse(url).queryParameters['sslmode'];
  return switch (value) {
    null => null,
    'disable' => PgSslMode.disable,
    'require' => PgSslMode.require,
    'verify-full' || 'verify_full' => PgSslMode.verifyFull,
    _ => throw ArgumentError.value(
        value,
        'sslmode',
        'Supported values are disable, require and verify-full',
      ),
  };
}

/// Parses a connection URL into the driver's endpoint type.
pg.Endpoint _endpointFor(String url) {
  final uri = Uri.parse(url);
  final userInfo = uri.userInfo.split(':');
  return pg.Endpoint(
    host: uri.host.isEmpty ? 'localhost' : uri.host,
    port: uri.hasPort ? uri.port : 5432,
    database: uri.pathSegments.isEmpty ? 'postgres' : uri.pathSegments.first,
    username:
        userInfo.isEmpty || userInfo.first.isEmpty ? null : userInfo.first,
    password: userInfo.length > 1 ? userInfo[1] : null,
  );
}

/// Backwards-compatible PostgreSQL pool name, as `sqlx-postgres` names it.
typedef PgPool = PostgresDriver;
