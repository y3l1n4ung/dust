import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Storing a timestamp in a database that has no timestamp type.
///
/// SQLite stores dates as TEXT, INTEGER, or REAL and leaves the choice to you.
/// Pick ISO-8601 text in UTC: it sorts correctly as a string, `date()` and
/// friends read it, and it is legible in a shell — which the epoch-integer
/// alternative is not.
///
/// Bind and read it as a `String`, converting at the edge. Doing it in the row
/// mapper keeps the conversion in one place instead of at every call site, and
/// keeps a naive local `DateTime` from reaching the database at all.
///
/// PostgreSQL has real `timestamptz`, so this file is SQLite's alone.
///
/// ```shell
/// dart run example/dates_and_times.dart
/// ```
Future<void> main() async {
  final db = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const {
      '0001.sql': '''
CREATE TABLE sessions (
  id         INTEGER PRIMARY KEY,
  created_at TEXT NOT NULL
);
''',
    },
  );

  try {
    final createdAt = DateTime.utc(2026, 3, 14, 9, 26, 53);
    await db.execute(
      r'INSERT INTO sessions (created_at) VALUES ($1)',
      <Object?>[createdAt.toIso8601String()],
    );

    final read = await db.fetchOne<DateTime>(
      'SELECT created_at FROM sessions',
      const [],
      // `toUtc` because a stored value without a zone parses as local time, and
      // a comparison against `DateTime.now().toUtc()` would then be off by the
      // offset of whichever machine ran it.
      (row) => DateTime.parse(row.read<String>('created_at')).toUtc(),
    );
    print('round trip: ${read.unwrapOrElse((error) => throw error)}');

    // Text dates still work with SQLite's date functions and with ordering.
    final onDay = await db.fetchScalar<int>(
      r'SELECT count(*) FROM sessions WHERE date(created_at) = $1',
      const <Object?>['2026-03-14'],
    );
    print('on 2026-03-14: ${onDay.unwrapOrElse((_) => -1)}');
  } finally {
    await db.close();
  }
}
