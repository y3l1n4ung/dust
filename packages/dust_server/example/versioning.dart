import 'dart:io';

import 'package:dust_server/server.dart';

/// Serving two versions of the same API.
///
/// Two ways to do it, and they are different promises:
///
/// * **In the path** — `/v1/notes`, `/v2/notes`. Visible, cacheable, trivial to
///   route, and impossible to get wrong by accident. Both versions are separate
///   routers, so v1 keeps working while v2 changes.
/// * **In a header** — `Accept: application/vnd.notes.v2+json`. Keeps one URL per
///   resource, which REST purists prefer, at the cost of a version that is
///   invisible in a log and easy for a client to omit. An absent header has to
///   mean something, and whatever you pick is a compatibility decision.
///
/// This example does both, because a real service usually ends up with both: the
/// path for the big break, the header for a field-level change.
///
/// > **Default to the oldest version, not the newest.** A client that sends no
/// > version wrote its code against whatever was current then. Defaulting to
/// > `latest` breaks it the day you ship v3 — silently, in production, with no
/// > deploy of theirs to blame.
///
/// Run it with `dart run example/versioning.dart`:
///
/// ```bash
/// curl -s localhost:8080/v1/notes
/// curl -s localhost:8080/v2/notes                    # a different shape
/// curl -s localhost:8080/notes                       # no header: v1
/// curl -s localhost:8080/notes -H 'accept: application/vnd.notes.v2+json'
/// curl -si localhost:8080/notes -H 'accept: application/vnd.notes.v9+json'
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  final v1 = Router()..route('/notes', get(listV1));
  final v2 = Router()..route('/notes', get(listV2));

  return Router()
    ..nest('/v1', v1)
    ..nest('/v2', v2)
    ..route('/notes', get(listNegotiated));
}

/// `GET /v1/notes` — the original shape: titles only.
List<String> listV1(Request request) => const ['first', 'second'];

/// `GET /v2/notes` — objects, so a field can be added later without another
/// version.
List<Map<String, Object?>> listV2(Request request) => const [
      {'id': 1, 'title': 'first'},
      {'id': 2, 'title': 'second'},
    ];

/// `GET /notes` — the version comes from `Accept`.
Future<Result<Object, Rejection>> listNegotiated(Request request) async {
  final accept = await request.header('accept') ?? '';
  final version = RegExp(r'application/vnd\.notes\.v(\d+)\+json')
      .firstMatch(accept)
      ?.group(1);

  return switch (version) {
    // Absent means v1, the oldest, not the newest. A client that sent nothing
    // wrote its code against what was current then.
    null || '1' => Ok(listV1(request)),
    '2' => Ok(listV2(request)),
    _ => Err(
        Rejection.status(406, 'unknown version: v$version, try v1 or v2'),
      ),
  };
}
