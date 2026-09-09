part of 'postgres_pool.dart';

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
