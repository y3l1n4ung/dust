import 'dart:io';

import 'package:dust_server/server.dart';

/// Testing an application built on this runtime.
///
/// Two ways in, and the choice is not stylistic:
///
/// * **`router.handler`** takes a `Request` and returns a `Response`, in
///   process, no socket. Fast enough to run thousands of, and right for
///   asserting statuses, bodies, and headers.
/// * **`serveRouter` on port 0** goes over a real socket with a real client.
///   Slower, and the only way to catch what the wire does: gzip, chunked
///   bodies, `HEAD` dropping a body, a WebSocket upgrade, connection reuse.
///
/// Start with the first. Reach for the second when the thing you are asserting
/// only happens on a socket.
///
/// > **Port 0, never a fixed port.** The OS assigns a free one. A suite pinned to
/// > 8080 fails when anything else holds it, and worse, it can *pass* against
/// > another process that happens to be listening — measuring something that is
/// > not your code at all.
///
/// The other rule worth stating: `buildApp` takes its dependencies, so a test
/// substitutes a store or a clock without a global. That is the only reason every
/// example in this directory splits `buildApp` out of `main`.
///
/// Run it with `dart run example/testing.dart`, or read
/// `test/example/examples_test.dart`, which drives all 34 examples this way.
Future<void> main() async {
  final server = await serveRouter(
    buildApp(NoteStore()),
    InternetAddress.anyIPv4,
    8080,
  );
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application over [store].
///
/// The dependency is a parameter rather than a global, which is what lets a test
/// hand in an empty store and assert on it afterwards.
Router buildApp(NoteStore store) {
  return Router()
    ..layer(const Compression())
    ..route('/notes', get(listNotes).post(addNote, status: 201))
    ..route('/notes/{id}', get(readNote))
    ..withState(store);
}

/// `GET /notes`
Future<List<String>> listNotes(Request request) async {
  final store = await request.state<NoteStore>();

  return store.titles;
}

/// `POST /notes`
Future<Map<String, Object?>> addNote(Request request) async {
  final store = await request.state<NoteStore>();
  final title = await request.body((json) => json['title']! as String);

  store.titles.add(title);
  return {'id': store.titles.length};
}

/// `GET /notes/{id}`
Future<Result<Map<String, Object?>, Rejection>> readNote(
  Request request,
) async {
  final id = await request.path<int>('id');
  final store = await request.state<NoteStore>();

  if (id < 1 || id > store.titles.length) {
    return const Err(Rejection.notFound('no such note'));
  }

  return Ok({'id': id, 'title': store.titles[id - 1]});
}

/// What the application keeps, injected so a test can inspect it.
final class NoteStore {
  /// Creates a store holding [titles].
  NoteStore([List<String>? titles]) : titles = titles ?? ['first'];

  /// The notes, in order.
  final List<String> titles;
}
