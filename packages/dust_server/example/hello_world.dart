import 'dart:io';

import 'package:dust_server/server.dart';

/// The smallest server that answers.
///
/// Four things are worth noticing in this much code:
///
/// * A handler is a function of the request. There is no base class and no
///   annotation — `get` takes what you give it.
/// * The verb builder is generic over the return type and encodes it, so a
///   handler ends in the value it produced rather than in a call to an encoder.
/// * A `String` goes out as `text/plain`; everything else becomes **JSON**.
///   That is axum's rule. To send a JSON string specifically, wrap it —
///   `['Hello']` — or call `jsonResponse`.
/// * `close(drain:)` finishes the requests already accepted before the process
///   exits. Skipping it drops them.
///
/// Run it with `dart run example/hello_world.dart`:
///
/// ```bash
/// curl localhost:8080/          # Hello, world!
/// curl localhost:8080/hello/ada # Hello, ada!
/// curl localhost:8080/json      # {"greeting":"Hello, world!"}
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
    ..route('/', get(hello))
    ..route('/hello/{name}', get(greet))
    ..route('/json', get(greeting));
}

/// `GET /` — a String, so it goes out as text.
String hello(Request request) => 'Hello, world!';

/// `GET /hello/{name}` — the path segment, coerced and read.
Future<String> greet(Request request) async {
  final name = await request.path<String>('name');

  return 'Hello, $name!';
}

/// `GET /json` — a model, encoded without being asked.
Map<String, Object?> greeting(Request request) => {
      'greeting': 'Hello, world!',
    };
