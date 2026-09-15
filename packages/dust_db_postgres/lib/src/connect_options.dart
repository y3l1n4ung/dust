part of 'postgres_pool.dart';

/// Connection settings for a PostgreSQL pool, named as `sqlx-postgres` names
/// them.
///
/// A thin pass-through to `package:postgres` rather than a second model of the
/// same settings: the driver owns connecting, and duplicating its options here
/// would be one more thing to keep in step.
final class PgConnectOptions {
  /// Creates one set of connection settings.
  const PgConnectOptions({
    this.sslMode,
    this.connectTimeout,
    this.queryTimeout,
    this.applicationName,
    this.maxConnectionAge,
    this.maxConnections,
  });

  /// Whether the connection requires TLS.
  ///
  /// Null leaves it to the connection URL's `?sslmode=`, and then to the
  /// driver's own default, which requires TLS. Deciding here instead would
  /// override a URL that already says what it wants.
  final PgSslMode? sslMode;

  /// How long to wait for a connection to be established.
  final Duration? connectTimeout;

  /// How long to wait for a statement before giving up.
  final Duration? queryTimeout;

  /// Name reported to the server, which shows up in `pg_stat_activity`.
  final String? applicationName;

  /// How long the pool keeps one connection before retiring it.
  ///
  /// Worth setting behind a proxy or load balancer that drops idle connections
  /// on its own schedule: retiring first means the pool replaces a connection
  /// rather than handing out one the far end has already closed.
  ///
  /// Null leaves connections in the pool for as long as it wants them.
  final Duration? maxConnectionAge;

  /// The most connections the pool opens at once.
  ///
  /// Null means [defaultMaxConnections], which is `sqlx`'s default. Every
  /// transaction holds one connection until it ends, so this is also the most
  /// transactions that can run at the same time; the rest wait for one.
  ///
  /// Keep the total across every process under the server's
  /// `max_connections`, which is 100 on a stock PostgreSQL and often lower on
  /// a hosted one.
  final int? maxConnections;

  /// The pool size used when [maxConnections] is null.
  static const defaultMaxConnections = 10;
}

/// How strictly a connection requires TLS.
enum PgSslMode {
  /// No TLS. For a local socket or a trusted network only.
  disable,

  /// TLS without verifying the server certificate.
  ///
  /// This accepts *every* certificate, so it stops eavesdropping but not an
  /// attacker who can position themselves in the middle. Prefer [verifyFull]
  /// anywhere the network is not already trusted.
  require,

  /// TLS with full certificate verification.
  verifyFull;

  /// This mode as the driver's own enum.
  pg.SslMode get _driverMode => switch (this) {
        PgSslMode.disable => pg.SslMode.disable,
        PgSslMode.require => pg.SslMode.require,
        PgSslMode.verifyFull => pg.SslMode.verifyFull,
      };
}
