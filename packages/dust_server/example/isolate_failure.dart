import 'dart:io';

import 'package:dust_server/server.dart';

/// Knowing when an isolate dies, and why it cannot be replaced.
///
/// A serving isolate that dies is otherwise invisible. The port stays bound by
/// the survivors, so traffic keeps flowing — at a fraction of the capacity you
/// asked for, with nothing in the log to say so. The throughput ceiling then
/// looks like the application's rather than the process's, which is a bad thing
/// to debug at three in the morning.
///
/// `ServerIsolates.alive` reports how many are still running against `size`,
/// and `onIsolateError` fires with whatever killed one.
///
/// **It cannot be restarted.** Killing an isolate does not release the socket
/// it bound, so a replacement cannot take the port back. This is the one place
/// `uvicorn --workers` genuinely does not port: an OS worker takes its socket
/// with it when it dies, and an isolate does not. Recovery is replacing the
/// process — which is what a supervisor, a systemd unit, or Kubernetes already
/// does, and the reason `isolates: 1` plus more replicas is usually the better
/// shape in a container.
///
/// So this is a health signal, not a self-heal. Report it, alert on it, and let
/// whatever supervises the process do the replacing.
///
/// Run it with `dart run example/isolate_failure.dart`:
///
/// ```bash
/// curl localhost:8080/health   # {"size":4,"alive":4}
/// ```
Future<void> main() async {
  final servers = await serveIsolates(
    buildApp,
    InternetAddress.anyIPv4,
    8080,
    isolates: 4,
    onIsolateError: (error, stackTrace) {
      // Nothing here can bring it back. Say so loudly instead.
      stderr.writeln('a serving isolate died: $error');
    },
  );

  stdout.writeln('serving on ${servers.port} across ${servers.size} isolates');

  await ProcessSignal.sigint.watch().first;
  await servers.close(drain: const Duration(seconds: 5));
}

/// Builds the application inside each isolate.
///
/// Top-level, because it is sent across an isolate boundary and a closure
/// capturing local state cannot be. Each isolate gets its own of everything the
/// factory builds, which is why a counter here would count a quarter of the
/// traffic.
Router buildApp() {
  return Router()
    ..route('/', get((request) async => 'ok'))
    ..route('/health', get((request) async => {'pid': pid}));
}
