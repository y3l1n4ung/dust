import 'dart:io';

import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';

/// Opening a database that outlives the process.
///
/// [Sqlite3Driver.open] creates the file when it is missing, so first run and
/// every run after it take the same path. Pass `createIfMissing: false` when a
/// missing file should be an error rather than an empty database — a typo in a
/// deployment path is otherwise indistinguishable from a fresh install.
///
/// ```shell
/// dart run example/open_a_file.dart
/// ```
Future<void> main() async {
  final directory = Directory.systemTemp.createTempSync('dust_sqlite');
  final path = '${directory.path}/app.db';

  final first = Sqlite3Driver.open(
    path,
    migrations: const {
      '0001_create_notes.sql': 'CREATE TABLE notes (body TEXT NOT NULL);',
    },
  );
  await first.execute(
    r'INSERT INTO notes (body) VALUES ($1)',
    const <Object?>['written once'],
  );
  await first.close();

  // A second process would open it exactly like this. The migration is already
  // applied, so it is skipped rather than re-run.
  final second = Sqlite3Driver.open(
    path,
    migrations: const {
      '0001_create_notes.sql': 'CREATE TABLE notes (body TEXT NOT NULL);',
    },
  );
  try {
    final body = await second.fetchScalar<String>(
      'SELECT body FROM notes',
      const [],
    );
    print('survived close: ${body.unwrapOrElse((_) => '')}');
  } finally {
    await second.close();
    directory.deleteSync(recursive: true);
  }
}
