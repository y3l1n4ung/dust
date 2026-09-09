import 'dart:io';

import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Configuring the connection, not the queries.
///
/// [SqliteConnectOptions] is one const value holding everything that must be
/// true before the first statement runs. Applying it at open time rather than
/// as ad-hoc `PRAGMA` statements is what keeps a connection from being
/// half-configured: there is no window where a query can run against a database
/// whose foreign keys are not yet on.
///
/// ```shell
/// dart run example/connect_options.dart
/// ```
Future<void> main() async {
  final directory = Directory.systemTemp.createTempSync('dust_sqlite');
  final db = Sqlite3Driver.open(
    '${directory.path}/app.db',
    options: const SqliteConnectOptions(
      // Enforce references rather than merely declaring them. Off by default in
      // SQLite itself, which surprises everyone exactly once.
      foreignKeys: true,
      // WAL lets readers work while a writer holds the database.
      journalMode: SqliteJournalMode.wal,
      // With WAL, `normal` is the usual durability trade: a commit is not
      // fsynced, but a crash cannot corrupt the database.
      synchronous: SqliteSynchronousMode.normal,
      // How long a statement waits on a lock before returning an error.
      busyTimeout: Duration(seconds: 5),
      // Anything else SQLite exposes as a pragma. Name and value are checked,
      // so a typo fails at open rather than being silently ignored.
      pragmas: <String, Object>{'cache_size': -8000},
    ),
  );

  try {
    final journal =
        await db.fetchScalar<String>('PRAGMA journal_mode', const []);
    final keys = await db.fetchScalar<int>('PRAGMA foreign_keys', const []);
    print('journal: ${journal.unwrapOrElse((_) => '?')}');
    print('foreign keys: ${keys.unwrapOrElse((_) => -1)}');
  } finally {
    await db.close();
    directory.deleteSync(recursive: true);
  }
}
