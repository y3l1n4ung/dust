import 'dart:io';

import 'package:dust_server/server.dart';

/// Paths, methods, nesting, and merging.
///
/// Four ways to put a route table together, and they are not interchangeable:
///
/// * `route` attaches one path to a `MethodRouter`. Two verbs on one path chain
///   onto each other — `get(...).post(...)` — so the path is written once.
/// * `nest` mounts a router under a prefix. The inner routes do not repeat it.
/// * `merge` folds another router in at the same level, for splitting a large
///   table across files without inventing a prefix for it.
/// * `fallback` answers whatever matched nothing. Without one, that is a bare
///   404.
///
/// A path that exists but not for this method answers **405** with `Allow`,
/// never 404 — the difference tells a client whether to fix the verb or the URL.
///
/// Run it with `dart run example/routing.dart`:
///
/// ```bash
/// curl -s localhost:8080/health
/// curl -s localhost:8080/api/notes
/// curl -s -X POST localhost:8080/api/notes
/// curl -i -X PUT localhost:8080/api/notes    # 405, Allow: GET, HEAD, POST
/// curl -s localhost:8080/nothing-here        # the fallback
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  // One path, two verbs, chained rather than declared twice.
  final notes = Router()
    ..route('/notes', get(listNotes).post(createNote, status: 201))
    ..route('/notes/{id}', get(readNote));

  // A second file's worth of routes, folded in at the top level.
  final operations = Router()..route('/health', get(health));

  return Router()
    ..nest('/api', notes)
    ..merge(operations)
    ..fallback(missing);
}

/// `GET /api/notes`
List<String> listNotes(Request request) => const ['first', 'second'];

/// `POST /api/notes`
Map<String, Object?> createNote(Request request) => const {'created': true};

/// `GET /api/notes/{id}`
Future<Map<String, Object?>> readNote(Request request) async => {
      'id': await request.path<String>('id'),
    };

/// `GET /health`
Map<String, Object?> health(Request request) => const {'status': 'ok'};

/// Whatever matched nothing.
Response missing(Request request) =>
    const Rejection.notFound('no such route').intoResponse();
