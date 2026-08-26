import 'dart:async';
import 'dart:io';
import 'dart:isolate';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// A cluster shuts down by asking each isolate to drain and waiting for it to
/// answer. An isolate that never answers — wedged on a synchronous loop, out
/// of memory, blocked on a syscall — must not hold the shutdown open forever,
/// because a deploy that cannot finish is an outage.
///
/// This wedges a worker on purpose and asserts that `close` still returns, and
/// that the isolate is gone afterwards.

/// Built inside every isolate, so the check runs where the router is built.
///
/// The worker wedges shortly after it starts serving. The loop allocates, so
/// it reaches a safepoint and `Isolate.kill` can still stop it; a loop that
/// allocates nothing may never be interruptible.
Router buildWedgingApp() {
  final isWorker =
      Isolate.current.debugName?.startsWith('dust_server') ?? false;
  if (isWorker) {
    Timer(const Duration(milliseconds: 200), () {
      while (true) {
        List<int>.filled(1, 0);
      }
    });
  }

  return Router()..route('/health', get((request) async => null));
}

void main() {
  group('closing a cluster with a wedged isolate', () {
    test('returns instead of waiting forever', () async {
      final cluster = await serveIsolates(
        buildWedgingApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 2,
      );

      // Give the worker time to reach the wedge.
      await Future<void>.delayed(const Duration(milliseconds: 400));

      await expectLater(
        cluster.close(drain: Duration.zero),
        completes,
      );
    }, timeout: const Timeout(Duration(seconds: 40)));

    test('gives up after the drain budget plus its grace period', () async {
      final cluster = await serveIsolates(
        buildWedgingApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 2,
      );

      await Future<void>.delayed(const Duration(milliseconds: 400));

      final started = DateTime.now();
      await cluster.close(drain: Duration.zero);
      final waited = DateTime.now().difference(started);

      // The budget is `drain + 5s`; anything far past that means the timeout
      // did not fire and something else released the wait.
      expect(waited, greaterThanOrEqualTo(const Duration(seconds: 5)));
      expect(waited, lessThan(const Duration(seconds: 20)));
    }, timeout: const Timeout(Duration(seconds: 40)));

    test('leaves the killed isolate holding its share of the port', () async {
      final cluster = await serveIsolates(
        buildWedgingApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 2,
      );
      final port = cluster.port;

      await Future<void>.delayed(const Duration(milliseconds: 400));
      await cluster.close(drain: Duration.zero);

      // `Isolate.kill` stops the isolate running but does not close the socket
      // it bound, and a shared bind is only released when the process exits.
      // An exclusive rebind therefore still fails. This is pinned rather than
      // wished away: a supervisor restarting a wedged worker in-process will
      // not get the port back, and has to replace the process instead.
      await expectLater(
        ServerSocket.bind(InternetAddress.loopbackIPv4, port),
        throwsA(isA<SocketException>()),
      );

      // Rebinding the way the cluster itself does still works, so a restart
      // that also passes `shared: true` comes up.
      final rebound = await ServerSocket.bind(
        InternetAddress.loopbackIPv4,
        port,
        shared: true,
      );
      addTearDown(rebound.close);

      expect(rebound.port, port);
    }, timeout: const Timeout(Duration(seconds: 40)));
  });
}
