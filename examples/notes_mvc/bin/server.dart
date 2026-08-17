import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:notes_mvc/app.dart';

/// ```bash
/// dart run bin/server.dart          # in memory
/// dart run bin/server.dart notes.db # a file, migrated on startup
/// ```
Future<void> main(List<String> arguments) async {
  final database = NotesDatabase.open(
    arguments.isEmpty ? ':memory:' : arguments.first,
  );
  final app = buildApp(Notes(database.connection));

  final server = await serveRouter(app, InternetAddress.loopbackIPv4, 8083);
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigint.watch().first;
  await server.close(drain: const Duration(seconds: 5));
  await database.connection.close();
}
