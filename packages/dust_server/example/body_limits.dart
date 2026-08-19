import 'dart:io';

import 'package:dust_server/server.dart';

/// Refusing a body that is too big.
///
/// Without a ceiling, one request can allocate until the process dies. It costs
/// an attacker a single connection, which is why every body extractor has a
/// limit and why the default is small — **1 MiB** — rather than generous.
///
/// Two places set one, and the **stricter** of the two wins:
///
/// * `Router(bodyLimit:)` bounds the whole application, and reaches generated
///   code whose own limit was fixed at build time.
/// * A limit on the extractor bounds one route.
///
/// Taking the minimum means adding either can only ever refuse more, so the
/// router limit is a **ceiling**: set it to the largest body the application
/// should ever accept, and tighten individual routes below it. A route cannot
/// opt into more.
///
/// The check happens twice, on purpose. `content-length` is compared **before**
/// the body is read, so an oversized upload is refused without transferring it.
/// A body arriving without a length — chunked, which the client chooses — is
/// counted as it flows.
///
/// > **A declared `content-length` is a claim, not a fact.** The second check is
/// > what makes the first safe to trust: a client that lies is still stopped,
/// > just later.
///
/// One operational surprise: a client that is still uploading when the limit is
/// hit may see the connection reset rather than read the 413. The server has
/// stopped reading, and the response cannot get past the bytes still in flight.
/// The refusal is correct either way, but a client that reports "connection
/// reset" for a large upload is usually being told its body was too big.
///
/// Run it with `dart run example/body_limits.dart`:
///
/// ```bash
/// curl -s  -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d '{"title":"small"}'
///
/// head -c 100000 /dev/zero | tr '\0' 'a' > big.txt
/// curl -si -X POST localhost:8080/notes -H 'content-type: application/json' \
///   --data-binary @big.txt                    # 413: over this route's 16 KB
/// curl -s  -X POST localhost:8080/avatar --data-binary @big.txt   # 200
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
///
/// 5 MB is the ceiling for the whole application, because one route genuinely
/// needs it. Everything else is tightened below that rather than left at the
/// ceiling.
Router buildApp() {
  return Router(bodyLimit: 5 * 1024 * 1024)
    ..route('/notes', post(createNote, status: 201))
    ..route('/avatar', post(uploadAvatar, status: 201));
}

/// `POST /notes` — 16 KB, which is far more than a title needs.
///
/// Tightened here rather than left at the application ceiling. A JSON endpoint
/// that accepts five megabytes is a place to send five megabytes.
Future<Map<String, Object?>> createNote(Request request) async {
  final json = await request.extract(
    const JsonExtractable<Map<String, Object?>>(_identity, limit: 16 * 1024),
  );

  return {'title': json['title']};
}

/// `POST /avatar` — the route the ceiling was raised for.
Future<Map<String, Object?>> uploadAvatar(Request request) async {
  final bytes = await request.extract(
    const RawBodyExtractable(limit: 5 * 1024 * 1024),
  );

  return {'bytes': bytes.length};
}

/// Hands back the decoded object unchanged.
Map<String, Object?> _identity(Map<String, Object?> json) => json;
