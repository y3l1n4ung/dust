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
    this.sslMode = PgSslMode.require,
    this.connectTimeout,
    this.queryTimeout,
    this.applicationName,
  });

  /// Whether the connection requires TLS.
  final PgSslMode sslMode;

  /// How long to wait for a connection to be established.
  final Duration? connectTimeout;

  /// How long to wait for a statement before giving up.
  final Duration? queryTimeout;

  /// Name reported to the server, which shows up in `pg_stat_activity`.
  final String? applicationName;

  /// These settings as the driver's own type.
  pg.ConnectionSettings get _settings => pg.ConnectionSettings(
        sslMode: switch (sslMode) {
          PgSslMode.disable => pg.SslMode.disable,
          PgSslMode.require => pg.SslMode.require,
          PgSslMode.verifyFull => pg.SslMode.verifyFull,
        },
        connectTimeout: connectTimeout,
        queryTimeout: queryTimeout,
        applicationName: applicationName,
      );
}

/// How strictly a connection requires TLS.
enum PgSslMode {
  /// No TLS. For a local socket or a trusted network only.
  disable,

  /// TLS without verifying the server certificate.
  require,

  /// TLS with full certificate verification.
  verifyFull,
}
