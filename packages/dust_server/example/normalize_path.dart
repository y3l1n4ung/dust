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
/// A `layer` covers everything the router it is on answers, and on a **nested**
/// router that includes the 404s inside its prefix. So this works either at the
/// top level or inside a `nest`, and `buildScopedApp` below shows the second —
/// `/api` normalized, and nothing outside it touched.
///
/// Run it with `dart run example/normalize_path.dart`:
///
/// ```bash
/// curl -s  localhost:8080/notes          # rewritten
/// curl -s  localhost:8080/notes/         # the same answer, same URL
/// curl -s  localhost:8080/api/notes/     # nested, covered from above
/// curl -s  localhost:8080/
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  final api = Router()..route('/notes', get(listNotes));

  return Router()
    ..layer(const NormalizePath())
    ..route('/notes', get(listNotes))
    ..route('/', get(home))
    ..nest('/api', api);
}

/// The same layer scoped to one prefix rather than the whole application.
///
/// `/api/notes/` is normalized; `/shop/notes/` is not, because the layer covers
/// only what its own router answers. Scoping matters when one half of an
/// application has URLs you must not rewrite — a webhook whose signature is
/// computed over the exact path, say.
Router buildScopedApp() {
  final api = Router()
    ..layer(const NormalizePath())
    ..route('/notes', get(listNotes));

  final shop = Router()..route('/notes', get(listNotes));

  return Router()
    ..nest('/api', api)
    ..nest('/shop', shop);
}

/// `GET /notes`
List<String> listNotes(Request request) => const ['first'];

/// `GET /`
Map<String, Object?> home(Request request) => const {'root': true};
