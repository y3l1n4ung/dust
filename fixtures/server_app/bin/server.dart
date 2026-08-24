import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:server_app/server_app.dart';

/// Serves the fixture.
///
/// The database path comes from the first argument, so the same entry point
/// serves a scratch file or a real one without an edit. `PORT` follows the
/// convention every host already expects.
///
/// `SIGINT` closes gracefully: the socket stops accepting, requests already in
/// flight get a few seconds to finish, and only then does the database close.
/// Killing the process instead would drop those responses on the floor.
Future<void> main(List<String> arguments) async {
  final path = arguments.isEmpty ? 'server_app.db' : arguments.first;
  final port = int.tryParse(Platform.environment['PORT'] ?? '') ?? 8080;

  final database = AppDatabase.open(path, options: appOptions);
  final server = await serveRouter(
    buildApp(database),
    InternetAddress.anyIPv4,
    port,
  );
  stdout.writeln('server_app listening on port ${server.port}, database $path');

  await ProcessSignal.sigint.watch().first;
  stdout.writeln('draining');
  await server.close(drain: const Duration(seconds: 5));
  await database.connection.close();
}
