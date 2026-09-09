import 'dart:io';

import 'package:dust_db_postgres/dust_db_postgres.dart';
import 'package:dust_server/server.dart';

/// Serving queries from PostgreSQL.
///
/// `state.dart` shows how a pool reaches a handler. This shows what to put in
/// it, and the two things PostgreSQL asks for that SQLite does not.
///
/// **`migrate()` is a separate call.** SQLite applies migrations while opening
/// the file; PostgreSQL is reached over a socket and cannot, so opening returns
/// immediately and the migrations run when you await them. Doing it before
/// `serve` is what keeps the first request from racing the schema.
///
/// **A transaction holds one pooled connection** for its whole closure. Two
/// statements outside one may land on different backends, so a read that must
/// see the write above it belongs inside a transaction rather than after it.
///
/// The driver is the only PostgreSQL-shaped thing here. A handler is given an
/// `Executor` and cannot tell which database answered — the same handler runs
/// over `dust_db_sqlite3` with a different two lines in `main`.
///
/// ```shell
/// DUST_DATABASE_URL='postgres://user:pw@localhost:5432/app?sslmode=disable' \
///   dart run example/postgres_database.dart
///
/// curl localhost:8080/notes
/// curl -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d '{"body":"written over HTTP"}'
/// ```
Future<void> main() async {
  final url = Platform.environment['DUST_DATABASE_URL'];
  if (url == null) {
    stdout.writeln('Set DUST_DATABASE_URL to a PostgreSQL database.');
    return;
  }

  final database = PostgresDriver.connect(
    url,
    migrations: const <String, String>{
      '0001_create_notes.sql': '''
CREATE TABLE IF NOT EXISTS example_notes (
  id   BIGSERIAL PRIMARY KEY,
  body TEXT NOT NULL
);
''',
    },
  );
  // Before serving: a request arriving against a schema that is still being
  // applied is a race you only lose in production.
  (await database.migrate()).unwrapOrElse((error) => throw error);

  final server = await serve(buildApp(database), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await database.close();
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(PostgresDriver database) {
  return Router()
    ..route('/notes', get(listNotes).post(addNote, status: 201))
    ..withState(database);
}

/// `GET /notes` — every note.
Future<List<Map<String, Object?>>> listNotes(Request request) async {
  final database = await request.state<PostgresDriver>();

  final notes = await database.fetchAll<Map<String, Object?>>(
    'SELECT id, body FROM example_notes ORDER BY id',
    const <Object?>[],
    (row) => <String, Object?>{
      'id': row.read<int>('id'),
      'body': row.read<String>('body'),
    },
  );

  return notes.unwrapOrElse((error) => throw error);
}

/// `POST /notes` — write one, and read it back in the same statement.
///
/// `RETURNING` rather than a second query: PostgreSQL has no `lastInsertId`,
/// and asking for the row costs nothing extra here.
Future<Map<String, Object?>> addNote(Request request) async {
  final database = await request.state<PostgresDriver>();
  final body = await request.body<Map<String, Object?>>((json) => json);

  final written = await database.transaction<Map<String, Object?>>((tx) async {
    return tx.fetchOne<Map<String, Object?>>(
      r'INSERT INTO example_notes (body) VALUES ($1) RETURNING id, body',
      <Object?>[body['body']],
      (row) => <String, Object?>{
        'id': row.read<int>('id'),
        'body': row.read<String>('body'),
      },
    );
  });

  return written.unwrapOrElse((error) => throw error);
}
