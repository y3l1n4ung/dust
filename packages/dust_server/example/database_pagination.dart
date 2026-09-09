import 'dart:io';

import 'package:dust_dart/db.dart';
import 'package:dust_db_sqlite3/dust_db_sqlite3.dart';
import 'package:dust_server/server.dart';

/// Paging and sorting a list endpoint from the query string.
///
/// `?limit=` and `?offset=` are values, so they bind. `?sort=` is not — no
/// dialect binds a column name, and the temptation is to reach for
/// `'ORDER BY $sort'`, which is how a list endpoint becomes an injection.
///
/// A `switch` over an enum is the answer. Each arm is a complete, constant
/// statement: the SQL is fixed at compile time, an unknown sort is a 400
/// rather than a database error, and `dust db build` can validate every arm
/// because each one is a literal.
///
/// The `limit` is clamped rather than trusted. `?limit=1000000` is not a
/// client asking politely for a lot; it is one request that reads the table
/// into memory.
///
/// ```shell
/// dart run example/database_pagination.dart
///
/// curl 'localhost:8080/notes?limit=2'
/// curl 'localhost:8080/notes?limit=2&offset=2'
/// curl 'localhost:8080/notes?sort=body'
/// curl 'localhost:8080/notes?sort=; DROP TABLE notes'   # 400
/// ```
Future<void> main() async {
  final database = openDatabase();

  final server = await serve(buildApp(database), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await database.close();
  await server.close(drain: const Duration(seconds: 5));
}

/// Opens the database with a handful of notes to page through.
Sqlite3Driver openDatabase() {
  final database = Sqlite3Driver.connect(
    const SqliteConnectOptions.memory(),
    migrations: const <String, String>{
      '0001_create_notes.sql': '''
CREATE TABLE notes (
  id   INTEGER PRIMARY KEY,
  body TEXT NOT NULL
);
''',
    },
  );
  for (final body in const <String>['delta', 'alpha', 'charlie', 'bravo']) {
    database.execute(
      r'INSERT INTO notes (body) VALUES ($1)',
      <Object?>[body],
    );
  }
  return database;
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(Sqlite3Driver database) {
  return Router()
    ..route('/notes', get(listNotes))
    ..withState(database);
}

/// What a caller may sort by, and nothing else.
enum NoteSort {
  /// Insertion order.
  id,

  /// Alphabetical.
  body;

  /// Reads one from the query string, or null when it is not one of these.
  static NoteSort? parse(String? value) => switch (value) {
        null || 'id' => NoteSort.id,
        'body' => NoteSort.body,
        _ => null,
      };
}

/// `GET /notes?limit=&offset=&sort=` — one page.
Future<Result<List<Map<String, Object?>>, Rejection>> listNotes(
  Request request,
) async {
  final database = await request.state<Sqlite3Driver>();

  final sort = NoteSort.parse(await request.query<String?>('sort'));
  if (sort == null) {
    return const Err(Rejection.badRequest('sort must be id or body'));
  }

  // Clamped, not trusted. A page size is a value the server decides on.
  final limit = (await request.query<int?>('limit') ?? 20).clamp(1, 100);
  final offset = (await request.query<int?>('offset') ?? 0).clamp(0, 1 << 30);

  // One constant statement per arm. Nothing is concatenated, so there is
  // nothing for a caller to inject into.
  final page = await switch (sort) {
    NoteSort.id => database.fetchAll<Map<String, Object?>>(
        r'SELECT id, body FROM notes ORDER BY id LIMIT $1 OFFSET $2',
        <Object?>[limit, offset],
        _asNote,
      ),
    NoteSort.body => database.fetchAll<Map<String, Object?>>(
        r'SELECT id, body FROM notes ORDER BY body LIMIT $1 OFFSET $2',
        <Object?>[limit, offset],
        _asNote,
      ),
  };

  return switch (page) {
    Ok(:final value) => Ok(value),
    Err() => const Err(Rejection.internal()),
  };
}

/// One `notes` row as JSON.
Map<String, Object?> _asNote(Row row) => <String, Object?>{
      'id': row.read<int>('id'),
      'body': row.read<String>('body'),
    };
