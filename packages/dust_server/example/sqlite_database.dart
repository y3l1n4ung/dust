import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:dust_server/server.dart';

/// Serving queries from a database.
///
/// `state.dart` shows how a pool reaches a handler. This shows what to put in
/// one: a driver, opened once in `main`, attached with `withState`, and read
/// back by a handler that never opens or closes anything itself.
///
/// SQLite here because it needs nothing installed — the same handlers run over
/// PostgreSQL with a different two lines in `main`, which is what
/// [postgres_database.dart](postgres_database.dart) shows.
///
/// Migrations are applied while the file is opened, so by the time `serve`
/// runs the schema is there. PostgreSQL cannot do that and asks for a separate
/// `migrate()`.
///
/// ```shell
/// dart run example/sqlite_database.dart
///
/// curl localhost:8080/notes
/// curl -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d '{"body":"written over HTTP"}'
/// ```
Future<void> main() async {
  final directory = Directory.systemTemp.createTempSync('dust_notes');
  final database = openDatabase('${directory.path}/notes.db');

  final server = await serve(buildApp(database), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await database.close();
  await server.close(drain: const Duration(seconds: 5));
  directory.deleteSync(recursive: true);
}

/// Opens the database the handlers read, migrations and all.
///
/// WAL because a server has readers and a writer at once, and a busy timeout
/// so a moment of contention waits rather than failing the request.
Sqlite3Driver openDatabase(String path) {
  return Sqlite3Driver.open(
    path,
    options: const SqliteConnectOptions(
      journalMode: SqliteJournalMode.wal,
      busyTimeout: Duration(seconds: 5),
      foreignKeys: true,
    ),
    migrations: const <String, String>{
      '0001_create_notes.sql': '''
CREATE TABLE notes (
  id   INTEGER PRIMARY KEY,
  body TEXT NOT NULL
);
''',
    },
  );
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(Sqlite3Driver database) {
  return Router()
    ..route('/notes', get(listNotes).post(addNote, status: 201))
    ..withState(database);
}

/// `GET /notes` — every note.
Future<List<Map<String, Object?>>> listNotes(Request request) async {
  final database = await request.state<Sqlite3Driver>();

  final notes = await database.fetchAll<Map<String, Object?>>(
    'SELECT id, body FROM notes ORDER BY id',
    const <Object?>[],
    _asNote,
  );

  return notes.unwrapOrElse((error) => throw error);
}

/// `POST /notes` — write one, and read it back in the same statement.
Future<Map<String, Object?>> addNote(Request request) async {
  final database = await request.state<Sqlite3Driver>();
  final body = await request.body<Map<String, Object?>>((json) => json);

  final written = await database.fetchOne<Map<String, Object?>>(
    r'INSERT INTO notes (body) VALUES ($1) RETURNING id, body',
    <Object?>[body['body']],
    _asNote,
  );

  return written.unwrapOrElse((error) => throw error);
}

/// One `notes` row as JSON.
Map<String, Object?> _asNote(Row row) => <String, Object?>{
      'id': row.read<int>('id'),
      'body': row.read<String>('body'),
    };
