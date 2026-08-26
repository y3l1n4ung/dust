import 'dart:io';

import 'package:dust_server/server.dart';

/// Accepting the same endpoint as JSON or as a form.
///
/// One endpoint, two wire formats: JSON for a client, urlencoded for a
/// `<form method="post">` with no JavaScript. The decision belongs in the
/// handler, because only it knows the two decoders produce the same value.
///
/// The extractors each answer **415** for a `content-type` they do not handle,
/// so the check happens before either runs rather than by catching a rejection.
/// A type neither handles is a 415 you raise yourself, naming what you accept —
/// which is more useful to a client than whichever extractor happened to run
/// last.
///
/// Run it with `dart run example/parse_body_by_content_type.dart`:
///
/// ```bash
/// curl -s -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d '{"title":"from json"}'
/// curl -s -X POST localhost:8080/notes -d 'title=from a form'
/// curl -i -X POST localhost:8080/notes -H 'content-type: text/xml' -d '<x/>'
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() => Router()..route('/notes', post(createNote, status: 201));

/// `POST /notes`
Future<Result<Map<String, Object?>, Rejection>> createNote(
  Request request,
) async {
  // The parameters after `;` are the charset and the boundary. Comparing the
  // whole header would fail on `application/json; charset=utf-8`, which is a
  // perfectly ordinary thing for a client to send.
  final type = (request.headers['content-type'] ?? '').split(';').first.trim();

  return switch (type) {
    'application/json' => Ok({
        'from': 'json',
        'title': (await request.body(_title)).trim(),
      }),
    'application/x-www-form-urlencoded' => Ok({
        'from': 'form',
        'title': switch ((await request.form()).field<String>('title')) {
          Ok(:final value) => value.trim(),
          Err() => '',
        },
      }),
    _ => Err(
        Rejection.unsupportedMediaType(
          'send application/json or application/x-www-form-urlencoded',
        ),
      ),
  };
}

/// Pulls the one field this example cares about out of decoded JSON.
String _title(Map<String, Object?> json) => json['title']! as String;
