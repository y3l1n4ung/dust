// Serves the example for an external load generator.
//
//   dart run tool/bench_server.dart <isolates> [port] [dbPath]
//
// Pass port 0 — the default — to let the OS pick a free one; the chosen port
// is printed on the READY line. A fixed port silently collides with whatever
// else is listening, and `shared: true` means the collision does not even
// fail: the load generator just measures the other process.
//
// With no path each isolate keeps its own in-memory store, which measures the
// server — routing, extraction, encoding. With a path every isolate opens the
// *same* SQLite file in WAL mode, which measures what a shared database does
// to those numbers.
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:todo_server/todo_server.dart';

/// Set before the isolates are spawned, and copied into each of them.
String? benchDatabasePath;

Router buildBenchApp() => buildApp(
      benchDatabasePath == null
          ? (TodoRepository()..seed())
          : SqliteTodoStore.open(
              benchDatabasePath!,
              SqliteTodoStore.sharedFile,
            ),
      log: (_) {},
      onError: (error, stack) {},
    );

Future<void> main(List<String> arguments) async {
  final isolates = arguments.isEmpty ? 1 : int.parse(arguments.first);
  final port = arguments.length > 1 ? int.parse(arguments[1]) : 0;
  final path = arguments.length > 2 ? arguments[2] : null;

  if (path != null) {
    // Migrate and seed once, here, so the isolates never race to create the
    // schema. Each of them then opens the finished file.
    final store = SqliteTodoStore.open(path, SqliteTodoStore.sharedFile);
    if ((await store.all()).isEmpty) {
      await store.add(
        const CreateTodo(title: 'benchmark row', assignTo: 'ada@dust.test'),
      );
    }
    await store.close();
    benchDatabasePath = path;
  }

  final cluster = await serveCluster(
    buildBenchApp,
    InternetAddress.loopbackIPv4,
    port,
    isolates: isolates,
  );

  stdout.writeln('READY isolates=${cluster.size} port=${cluster.port}');
  await ProcessSignal.sigint.watch().first;
  await cluster.close(drain: const Duration(seconds: 2));
}
