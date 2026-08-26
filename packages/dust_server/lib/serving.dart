/// Running a router as a server, including shutting one down.
///
/// ```dart
/// import 'package:dust_server/serving.dart';
///
/// final server = await serve(app, InternetAddress.anyIPv4, 8080);
/// await ProcessSignal.sigterm.watch().first;
/// await server.close(drain: const Duration(seconds: 15));
/// ```
///
/// Everything here is also exported from `package:dust_server/server.dart`.
library;

export 'src/serving/background.dart' hide backgroundTasksIn, disposableLayersIn;
export 'src/serving/serving.dart';
