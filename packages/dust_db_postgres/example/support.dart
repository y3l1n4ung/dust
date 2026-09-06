import 'dart:io';

/// The database every example in this directory runs against.
///
/// There is no in-memory PostgreSQL, so unlike the SQLite examples these cannot
/// bring their own. Each file names the URL it needs and says so rather than
/// failing with a connection error.
String? get databaseUrl => Platform.environment['DUST_DATABASE_URL'];

/// Prints how to point the examples at a database.
void printMissingDatabaseUrl() {
  print(
    "Set DUST_DATABASE_URL to a PostgreSQL database this may write to, e.g. "
    "'postgres://user:pw@localhost:5432/app?sslmode=disable'.",
  );
}
