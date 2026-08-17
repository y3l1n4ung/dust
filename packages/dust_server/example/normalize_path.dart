import 'dart:io';

import 'package:dust_server/server.dart';

/// Making `/notes` and `/notes/` the same route.
///
/// The router treats them as different paths on purpose — a table that silently
/// matched both would hide a duplicate route. This layer settles the question
/// once, at the edge, instead of every route declaring itself twice.
///
/// Two ways to settle it, and the choice is not cosmetic:
///
/// * **Rewrite** (`NormalizePath()`) serves the normalized path directly. One
///   request, one response, and the address bar is left alone.
/// * **Redirect** (`NormalizePath.redirecting()`) answers **308** and sends the
///   client to the canonical form. It costs a round trip, and it collapses two
///   URLs into one for caches, logs, and search engines — which is the reason to
///   pay for it.
///
/// 308 rather than 301, because 308 keeps the method: a `POST` to `/notes/`
/// arrives at `/notes` as a `POST`. A 301 would turn it into a `GET` and lose the
/// body.
///
/// `/` is never touched. It is already canonical, and stripping its slash would
/// leave an empty path nothing can match.
///
/// > **It has to be on the top-level router.** A `layer` on a *nested* router
/// > runs only after one of that router's routes has matched — and this layer
/// > exists to run **before** matching. Put it inside a `nest` and it silently
/// > does nothing: `/api/notes/` matches nothing, so nothing normalizes it, and
/// > the request 404s with no hint as to why. Above the `nest` it covers every
/// > route below. `buildMisplacedApp` in this file is that mistake, kept so it
/// > can be seen rather than described.
///
/// Run it with `dart run example/normalize_path.dart`:
///
/// ```bash
/// curl -s  localhost:8080/notes          # rewritten
/// curl -s  localhost:8080/notes/         # the same answer, same URL
/// curl -s  localhost:8080/api/notes/     # nested, and covered from above
/// curl -s  localhost:8080/
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  final api = Router()..route('/notes', get(listNotes));

  return Router()
    // Above the nest, so every route below is covered — including `/api`.
    ..layer(const NormalizePath())
    ..route('/notes', get(listNotes))
    ..route('/', get(home))
    ..nest('/api', api);
}

/// The same layer in the wrong place, kept so the mistake can be seen.
///
/// Nothing here normalizes `/api/notes/`: the nested router's layer runs only
/// once one of its routes has matched, and `/api/notes/` matches none of them.
/// The request 404s, and the layer that was added to prevent exactly that never
/// runs.
Router buildMisplacedApp() {
  final api = Router()
    ..layer(const NormalizePath())
    ..route('/notes', get(listNotes));

  return Router()..nest('/api', api);
}

/// `GET /notes`
List<String> listNotes(Request request) => const ['first'];

/// `GET /`
Map<String, Object?> home(Request request) => const {'root': true};
