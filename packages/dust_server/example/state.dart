import 'dart:io';

import 'package:dust_server/server.dart';

/// Passing a database, a client, or a config to a handler.
///
/// `withState(value)` attaches it; `request.state<T>()` reads it back. This is
/// how a pool arrives in axum and how a dependency arrives in FastAPI, and the
/// reason handlers here are top-level functions rather than methods on a
/// controller: there is no constructor to inject into.
///
/// State is keyed **by type**, which has two consequences worth knowing before
/// you hit them:
///
/// * Two `withState` calls with the same type — the second wins, silently. Wrap
///   each in its own type when you need two of something.
/// * A `state<T>()` for a type nothing attached is a **500**, not a 404. It is
///   a wiring mistake in your route table, not something a client did, and the
///   message says which type is missing.
///
/// Run it with `dart run example/state.dart`:
///
/// ```bash
/// curl -s localhost:8080/notes
/// curl -s -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d '{"title":"buy milk"}'
/// curl -s localhost:8080/config
/// curl -i localhost:8080/missing    # 500, nothing attached that type
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/notes', get(listNotes).post(addNote, status: 201))
    ..route('/config', get(readConfig))
    ..route('/missing', get(readMissing))
    // Two different types, so neither overwrites the other.
    ..withState(NoteStore())
    ..withState(const ShopConfig(currency: 'GBP'));
}

/// `GET /notes`
Future<List<String>> listNotes(Request request) async {
  final store = await request.state<NoteStore>();

  return store.titles;
}

/// `POST /notes`
Future<Map<String, Object?>> addNote(Request request) async {
  final store = await request.state<NoteStore>();
  final body = await request.body((json) => json['title']! as String);

  store.titles.add(body);
  return {'count': store.titles.length};
}

/// `GET /config`
Future<Map<String, Object?>> readConfig(Request request) async {
  final config = await request.state<ShopConfig>();

  return {'currency': config.currency};
}

/// `GET /missing` — nothing attached a [Uri], so this is a 500.
Future<Map<String, Object?>> readMissing(Request request) async {
  final absent = await request.state<Uri>();

  return {'unreachable': absent.toString()};
}

/// Stands in for a database.
///
/// Held for the life of the process, so state is the place for the things that
/// are expensive to build once and cheap to share — a pool, an HTTP client, a
/// template set. Anything per-request belongs in an extractor instead.
final class NoteStore {
  /// What has been written so far.
  final titles = <String>['first'];
}

/// Configuration read at startup.
final class ShopConfig {
  /// Creates a [ShopConfig].
  const ShopConfig({required this.currency});

  /// Which currency prices are in.
  final String currency;
}
