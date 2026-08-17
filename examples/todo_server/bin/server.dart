import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:todo_server/todo_server.dart';

/// Serves the example on port 8080 until interrupted.
///
/// ```bash
/// dart run bin/server.dart
/// curl -H 'authorization: Bearer todos:read' localhost:8080/api/v1/todos
/// ```
/// Pass a database path to persist; the default keeps everything in memory.
///
/// ```bash
/// dart run bin/server.dart            # in-memory
/// dart run bin/server.dart todos.db   # a file, migrated on startup
/// ```
Future<void> main(List<String> arguments) async {
  final store = arguments.isEmpty
      ? (TodoRepository()..seed())
      : SqliteTodoStore.open(arguments.first);

  final app = buildApp(store);

  final server = await serveRouter(app, InternetAddress.loopbackIPv4, 8080);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  stdout.writeln('draining ${server.inFlight} in-flight requests');
  await server.close(drain: const Duration(seconds: 10));
  await store.close();
}
