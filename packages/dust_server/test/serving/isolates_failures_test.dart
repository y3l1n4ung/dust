import 'dart:async';
import 'dart:io';
import 'dart:isolate';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

/// Every isolate builds its own router, so the factory has to be top-level.
Router failsOnTheSecondSpawn() {
  if ((Isolate.current.debugName ?? '').contains('isolate 2')) {
    throw StateError('cannot build here either');
  }
  return Router()..route('/', get((request) async => 'ok'));
}

/// Serves, then dies of an uncaught asynchronous error a moment later.
///
/// An isolate error arrives as `[error, stackTrace]`, which is the shape the
/// death callback reads; killing an isolate instead reports an exit with no
/// error and takes a different path.
Router diesShortlyAfterStarting() {
  // Only in a spawned isolate. `serveIsolates` calls the factory in the calling
  // isolate too, and a timer that throws there takes the test process with it.
  if ((Isolate.current.debugName ?? '').contains('dust_server isolate')) {
    Timer(const Duration(milliseconds: 150), () {
      throw StateError('died while serving');
    });
  }
  return Router()..route('/', get((request) async => 'ok'));
}

/// Succeeds in the isolate that calls it and fails in every spawned one.
///
/// The realistic shape of a startup failure: a file the parent already holds
/// open, an environment variable set for the process but read per isolate, a
/// port bound before the fork. The parent builds fine and the child does not.
Router failsOnlyWhenSpawned() {
  if ((Isolate.current.debugName ?? '').contains('dust_server isolate')) {
    throw StateError('cannot build here');
  }
  return Router()..route('/', get((request) async => 'ok'));
}

Router buildClusterApp() {
  final servedBy = Isolate.current.debugName ?? 'main';
  var served = 0;

  return Router()
    ..route('/who', get((request) async => jsonResponse({'isolate': servedBy})))
    ..route(
      '/count',
      get(
        (request) async =>
            jsonResponse({'isolate': servedBy, 'served': ++served}),
      ),
    )
    ..route('/health', get((request) async => noContent()));
}

/// A dead isolate, and what startup does when one will not come up.
void main() {
  group('a dead isolate', () {
    test('is reported by alive against size', () async {
      final servers = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 2,
      );
      addTearDown(() => servers.close(drain: const Duration(seconds: 1)));

      expect(servers.size, 2);
      expect(servers.alive, 2, reason: 'nothing has died yet');
    });
  });

  group('startup', () {
    test('a factory that throws only in a spawn fails instead of hanging',
        () async {
      // Waiting on the ready port alone waits forever: the isolate sends it
      // after the factory has already thrown, so nothing ever arrives.
      await expectLater(
        serveIsolates(
          failsOnlyWhenSpawned,
          InternetAddress.loopbackIPv4,
          0,
          isolates: 3,
        ).timeout(const Duration(seconds: 10)),
        throwsA(
          isA<StateError>().having(
            (e) => e.message,
            'message',
            allOf(contains('before it began serving'), contains('isolate 1')),
          ),
        ),
      );
    });

    test('a failed start leaves no port bound', () async {
      final server = await serve(
        Router()..route('/', get((request) async => 'free')),
        InternetAddress.loopbackIPv4,
        0,
      );
      final port = server.port;
      await server.close(drain: Duration.zero);

      await expectLater(
        serveIsolates(
          failsOnlyWhenSpawned,
          InternetAddress.loopbackIPv4,
          port,
          isolates: 2,
        ).timeout(const Duration(seconds: 10)),
        throwsA(isA<StateError>()),
      );

      // If the local server had been left bound, this would fail.
      final rebound = await serve(
        Router()..route('/', get((request) async => 'rebound')),
        InternetAddress.loopbackIPv4,
        port,
      );
      addTearDown(() => rebound.close(drain: Duration.zero));
      expect(rebound.port, port);
    });

    test('a failure on the second spawn tears down the first', () async {
      // The first isolate is already running when the second fails. Leaving it
      // alive would leak a heap and a share of the port for a server that
      // never started.
      await expectLater(
        serveIsolates(
          failsOnTheSecondSpawn,
          InternetAddress.loopbackIPv4,
          0,
          isolates: 3,
        ).timeout(const Duration(seconds: 10)),
        throwsA(
          isA<StateError>()
              .having((e) => e.message, 'message', contains('isolate 2')),
        ),
      );
    });

    test('an isolate that dies while serving reaches onIsolateError', () async {
      final failures = <Object?>[];
      final servers = await serveIsolates(
        diesShortlyAfterStarting,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 2,
        onIsolateError: (error, _) => failures.add(error),
      );
      addTearDown(() => servers.close(drain: const Duration(seconds: 1)));

      expect(servers.alive, 2, reason: 'both are up at first');

      await Future<void>.delayed(const Duration(seconds: 1));

      expect(failures, isNotEmpty, reason: 'the death has to be reported');
      expect(failures.first.toString(), contains('died while serving'));
      expect(servers.alive, lessThan(servers.size));
    });

    test('one isolate serves without spawning any', () async {
      final servers = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 1,
      );
      addTearDown(() => servers.close(drain: const Duration(seconds: 1)));

      expect(servers.size, 1);
      expect(servers.alive, 1);
    });

    test('closing twice is not an error', () async {
      final servers = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 2,
      );

      await servers.close(drain: const Duration(seconds: 1));
      await expectLater(
        servers.close(drain: const Duration(seconds: 1)),
        completes,
      );
    });
  });
}
