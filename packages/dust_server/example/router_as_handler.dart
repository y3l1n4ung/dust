import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:shelf/shelf.dart' as shelf;

/// Using a router where a `shelf` handler is wanted.
///
/// `Router` implements `Service`, whose whole contract is `call(Request)`. Dart
/// treats an object with a `call` method as assignable to a matching function
/// type, so a router **is** a shelf `Handler` — no adapter, no `.handler`
/// getter at the call site.
///
/// That is the same reason `axum::serve(listener, app)` takes a `Router`
/// directly: axum's `Router` implements tower's `Service`, and `serve` is
/// generic over it.
///
/// Why it matters: `dust_server` composes with the shelf ecosystem rather than
/// replacing it. Anything that takes a `Handler` — an existing pipeline, a
/// third-party middleware, `shelf_io` itself — takes a Dust router unchanged.
///
/// One wrinkle: `very_good_analysis` enables `implicit_call_tearoffs`, so the
/// implicit form is linted even though it compiles. Write `app.call` where a
/// `Handler` is expected and the lint is satisfied without an adapter.
///
/// Run it with `dart run example/router_as_handler.dart`:
///
/// ```bash
/// curl -i localhost:8080/        # served through a shelf Pipeline
/// ```
Future<void> main() async {
  final server = await serve(buildApp(), InternetAddress.anyIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
}

/// Assembles the application, kept apart from `main` so tests can serve it.
///
/// The inner router is handed to a plain shelf `Pipeline`, which knows nothing
/// about Dust, and the result is mounted back into an outer router.
Router buildApp() {
  final inner = Router()
    ..route('/', get((request) async => 'from the inner router'))
    ..route('/who', get((request) async => {'router': 'inner'}));

  // A shelf Pipeline takes a Handler. The router goes in directly.
  final piped =
      const shelf.Pipeline().addMiddleware(_stamp).addHandler(inner.call);

  return Router()..mount('/', piped);
}

/// Ordinary shelf middleware, written without reference to Dust.
shelf.Handler _stamp(shelf.Handler inner) {
  return (request) async {
    final response = await inner(request);
    return response.change(headers: {'x-served-through': 'shelf-pipeline'});
  };
}
