import 'package:dust_db_postgres/dust_db_postgres.dart';

import 'support.dart';

/// Storing a point in time.
///
/// Use `timestamptz`, always. Despite the name it stores no zone: it converts
/// the value to UTC on the way in and back to the session's zone on the way
/// out, so it names one instant. `timestamp` stores the digits you gave it,
/// which means two servers in different zones disagree about what it meant.
///
/// The driver decodes both to a `DateTime`, and `readDateTime` normalises it to
/// UTC — so a comparison in Dart cannot go wrong on whichever machine ran it.
/// SQLite has no date type at all, which is why that file is about choosing a
/// representation and this one is not.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/timestamps.dart
/// ```
Future<void> main() async {
  final url = databaseUrl;
  if (url == null) return printMissingDatabaseUrl();

  final db = PostgresDriver.connect(url);
  try {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_sessions', const []);
    await db.unsafe.execute(
      '''
CREATE TABLE example_sessions (
  id         BIGSERIAL PRIMARY KEY,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ
)''',
      const [],
    );

    final createdAt = DateTime.utc(2026, 3, 14, 9, 26, 53);
    await db.execute(
      r'INSERT INTO example_sessions (id, created_at, expires_at) VALUES ($1, $2, $3)',
      <Object?>[1, createdAt, null],
    );
    // The column's DEFAULT is the server's clock, not this process's.
    await db.execute(
      r'INSERT INTO example_sessions (id) VALUES ($1)',
      const <Object?>[2],
    );

    final read = await db.fetchOne<DateTime>(
      'SELECT created_at FROM example_sessions WHERE id = 1',
      const [],
      (row) => row.readDateTime('created_at'),
    );
    print('round trip: ${read.unwrapOrElse((error) => throw error)}');

    // A nullable timestamp needs the nullable read, same as any other column.
    final expiry = await db.fetchOne<DateTime?>(
      'SELECT expires_at FROM example_sessions WHERE id = 1',
      const [],
      (row) => row.readDateTimeNullable('expires_at'),
    );
    print('expires: ${expiry.unwrapOrElse((error) => throw error)}');

    // Let the database do the arithmetic: `now()` is the server's clock, which
    // is the one every other client is comparing against.
    final recent = await db.fetchScalar<int>(
      r"SELECT count(*) FROM example_sessions WHERE created_at > now() - $1::interval",
      const <Object?>['30 days'],
    );
    // The March row is older than that; the defaulted one is not.
    print('in the last 30 days: ${recent.unwrapOrElse((_) => -1)}');
  } finally {
    await db.unsafe.execute('DROP TABLE IF EXISTS example_sessions', const []);
    await db.close();
  }
}
