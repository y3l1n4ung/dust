import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';

/// Work that outlives the response that started it.
///
/// Sending a receipt, warming a cache, writing an audit row: a caller should not
/// wait for any of it, and it still has to finish.
///
/// > **`unawaited(...)` is the trap this replaces.** Shutdown counts
/// > **requests**, so work spawned outside one is invisible to it. On every
/// > deploy that work is killed mid-flight and nothing logs it — a customer gets
/// > their 201 and never gets their email, with no trace of why. A task
/// > registered here is drained with everything else, and `close` reports
/// > `false` if it did not finish.
///
/// The registry can also be attached with `withState` alone — `serve`
/// finds it there when no `background:` is passed. That is what lets a
/// **clustered** server drain its tasks: each isolate builds its own registry
/// inside the factory, so nothing outside can hand one in.
///
/// A task does **not** inherit the request's tracing span. The span ends when
/// the response goes out, so a task that kept it would write attributes onto a
/// finished, already-exported span. Work that wants a trace starts its own.
///
/// Two things to be clear about before reaching for it:
///
/// * **It is in-process and unpersisted.** A task lost to a crash is gone, and
///   nothing retries it. That is the right shape for work that is *nice* to
///   finish and the wrong shape for work that *must* happen — an outbox table or
///   a real queue is the answer there, and this is not a substitute.
/// * **A task that throws is reported, not raised.** An unhandled asynchronous
///   error would otherwise take the isolate down. The report names the task, so
///   a failure at three in the morning can be placed.
///
/// Run it with `dart run example/background_tasks.dart`:
///
/// ```bash
/// curl -s -X POST localhost:8080/orders -H 'content-type: application/json' \
///   -d '{"email":"ada@example.com"}'   # returns at once
/// curl -s localhost:8080/receipts      # the task has finished by now
/// ```
Future<void> main() async {
  final tasks = BackgroundTasks();
  final server = await serve(
    buildApp(tasks),
    InternetAddress.anyIPv4,
    8080,
    // Without this, shutdown drains requests and abandons everything else.
    background: tasks,
  );
  stdout.writeln('listening on http://${server.address.host}:${server.port}');

  await ProcessSignal.sigterm.watch().first;
  stdout.writeln('draining ${server.inFlight} request(s), '
      '${server.pendingTasks} task(s)');

  if (!await server.close(drain: const Duration(seconds: 15))) {
    stderr.writeln('gave up with work still running');
    exitCode = 1;
  }
}

/// Assembles the application, kept apart from `main` so tests can serve it.
Router buildApp(BackgroundTasks tasks) {
  return Router()
    ..route('/orders', post(placeOrder, status: 201))
    ..route('/receipts', get(sentReceipts))
    ..withState(tasks)
    ..withState(Receipts());
}

/// `POST /orders` — answers immediately, sends the receipt afterwards.
Future<Map<String, Object?>> placeOrder(Request request) async {
  final tasks = await request.state<BackgroundTasks>();
  final receipts = await request.state<Receipts>();
  final email = await request.body((json) => json['email']! as String);

  // The caller is not kept waiting for a mail server. `run` returns false when
  // the registry is draining, which is worth acting on: the order was placed and
  // the receipt will not be sent, so it belongs in an outbox rather than lost.
  final accepted = tasks.run('receipt', () async {
    await Future<void>.delayed(const Duration(milliseconds: 20));
    receipts.sent.add(email);
  });

  return {'placed': true, 'receiptQueued': accepted};
}

/// `GET /receipts` — what the background work has done so far.
Future<Map<String, Object?>> sentReceipts(Request request) async {
  final receipts = await request.state<Receipts>();

  return {'sent': receipts.sent};
}

/// Stands in for a mail server's outbox.
final class Receipts {
  /// Who has been sent one.
  final sent = <String>[];
}
