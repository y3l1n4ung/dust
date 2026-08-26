import 'dart:io';

import 'package:dust_server/server.dart';

/// Reading headers, and the host the client claims to have asked for.
///
/// `header(name)` returns `String?` — a header is absent often enough that its
/// type says so, and an absent one is not an error.
///
/// `host()` is the part worth reading twice. The `Host` header is **client
/// input**. It reaches your process unverified, so it is fine to route on and
/// wrong to trust: building a password-reset link out of it hands an attacker
/// the link's domain. Compare it against a list you configured, and use that.
///
/// Run it with `dart run example/headers_and_host.dart`:
///
/// ```bash
/// curl -s localhost:8080/echo -H 'x-trace: abc'
/// curl -s localhost:8080/echo                       # absent reads as null
/// curl -s localhost:8080/all -H 'a: 1' -H 'b: 2'
/// curl -s localhost:8080/host
/// curl -i localhost:8080/host -H 'host: evil.example'  # 400, not on the list
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Hosts this application answers for.
const allowedHosts = {'localhost', '127.0.0.1'};

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() {
  return Router()
    ..route('/echo', get(echo))
    ..route('/all', get(all))
    ..route('/host', get(whichHost));
}

/// `GET /echo` — one header, or null.
Future<Map<String, Object?>> echo(Request request) async => {
      'trace': await request.header('x-trace'),
    };

/// `GET /all` — every header at once.
Future<Map<String, Object?>> all(Request request) async => {
      'headers': await request.extract(headers()),
    };

/// `GET /host` — checked against the list, never trusted as it arrives.
Future<Result<Map<String, Object?>, Rejection>> whichHost(
  Request request,
) async {
  final claimed = (await request.host()).split(':').first;
  if (!allowedHosts.contains(claimed)) {
    return const Err(Rejection.badRequest('unrecognised host'));
  }

  return Ok({'host': claimed});
}
