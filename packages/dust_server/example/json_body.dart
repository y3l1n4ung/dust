import 'dart:io';

import 'package:dust_dart/serde.dart';
import 'package:dust_server/server.dart';

/// Decoding a JSON body into a model.
///
/// `body(Model.deserialize)` takes the function that builds your type. A
/// generated model supplies it; this one is hand-written, so the example runs
/// without a codegen step.
///
/// Four failures, four statuses, and none of them reach the handler:
///
/// | Sent | Answer |
/// | :--- | :--- |
/// | no `content-type: application/json` | 415 |
/// | bytes that are not JSON | 400 |
/// | JSON of the wrong shape — an array, a number | 422 |
/// | an object missing a required field | 422 |
///
/// Run it with `dart run example/json_body.dart`:
///
/// ```bash
/// curl -s -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d '{"title":"buy milk","body":"two litres"}'
/// curl -i -X POST localhost:8080/notes -d 'title=x'      # 415
/// curl -i -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d 'not json'                                        # 400
/// curl -i -X POST localhost:8080/notes -H 'content-type: application/json' \
///   -d '{"body":"no title"}'                             # 422
/// ```
Future<void> main() async {
  final server = await serveRouter(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp() => Router()..route('/notes', post(createNote, status: 201));

/// `POST /notes`
Future<Note> createNote(Request request) async {
  final note = await request.body(Note.deserialize);

  return note;
}

/// What the endpoint accepts, and answers with.
///
/// [Serializable] comes from `package:dust_dart/serde.dart`, which
/// `server.dart` does not re-export — the runtime re-exports `fp.dart` for
/// `Result` and stops there. Declaring `serialize()` without the interface
/// compiles and then answers 500, because the encoder looks for the interface
/// or a `toJson`, so the import is not optional.
final class Note implements Serializable {
  /// Creates a [Note].
  const Note({required this.title, required this.body});

  /// Reads a [Note] from decoded JSON.
  ///
  /// The throw is what produces the 422, and **what it says reaches the
  /// client**. `json['title']! as String` throws "Null check operator used on a
  /// null value" — a Dart implementation detail that names no field and tells a
  /// caller nothing. So the field is named here instead.
  ///
  /// A generated `Deserialize()` does this for you. Hand-written, it is worth
  /// the three lines.
  static Note deserialize(Map<String, Object?> json) => Note(
        title: switch (json['title']) {
          final String title => title,
          null => throw const FormatException('title is required'),
          _ => throw const FormatException('title must be a string'),
        },
        body: switch (json['body']) {
          final String body => body,
          null => '',
          _ => throw const FormatException('body must be a string'),
        },
      );

  /// The heading.
  final String title;

  /// The text.
  final String body;

  @override
  Map<String, Object?> serialize() => {'title': title, 'body': body};

  @override
  Map<String, Object?> toJson() => serialize();
}
