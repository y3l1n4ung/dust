part of 'postgres_pool.dart';

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

/// Backwards-compatible PostgreSQL pool name, as `sqlx-postgres` names it.
typedef PgPool = PostgresDriver;
