import 'dart:async';
import 'dart:io';
import 'dart:isolate';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

/// Every isolate builds its own router, so the factory has to be top-level.
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

void main() {
  _deathTests();
  group('serveIsolates', () {
    test('serves from one isolate when asked for one', () async {
      final cluster = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 1,
      );

      final response = await http.get(
        Uri.parse('http://${cluster.address.host}:${cluster.port}/health'),
      );

      expect(response.statusCode, 204);
      expect(cluster.size, 1);

      await cluster.close(drain: const Duration(seconds: 2));
    });

    test('shares one port across isolates', () async {
      final cluster = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 3,
      );

      expect(cluster.size, 3);
      expect(cluster.port, greaterThan(0));

      final responses = await Future.wait([
        for (var i = 0; i < 30; i++)
          http.get(
            Uri.parse('http://${cluster.address.host}:${cluster.port}/who'),
          ),
      ]);

      expect(
        responses.every((response) => response.statusCode == 200),
        isTrue,
      );

      await cluster.close(drain: const Duration(seconds: 2));
    });

    test('spreads work across more than one isolate', () async {
      final cluster = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 4,
      );

      final origin = 'http://${cluster.address.host}:${cluster.port}';
      final servers = <String>{};
      for (var round = 0; round < 12; round++) {
        final responses = await Future.wait([
          for (var i = 0; i < 8; i++) http.get(Uri.parse('$origin/who')),
        ]);
        for (final response in responses) {
          servers.add(response.body);
        }
        if (servers.length > 1) break;
      }

      // The operating system decides which socket accepts, so this asserts that
      // more than one can, not that any particular one does.
      expect(servers.length, greaterThan(1));

      await cluster.close(drain: const Duration(seconds: 2));
    });

    test('builds separate state in each isolate', () async {
      final cluster = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 4,
      );

      final origin = 'http://${cluster.address.host}:${cluster.port}';
      final startedAtOne = <String>{};

      // Which isolate accepts is the operating system's call, so this keeps
      // asking until two of them have answered. Each one counting its own
      // first request as 1 is what proves the state is not shared.
      for (var round = 0; round < 20 && startedAtOne.length < 2; round++) {
        final responses = await Future.wait([
          for (var i = 0; i < 8; i++) http.get(Uri.parse('$origin/count')),
        ]);
        for (final response in responses) {
          if (countOf(response.body) == 1) {
            startedAtOne.add(isolateOf(response.body));
          }
        }
      }

      expect(startedAtOne.length, greaterThanOrEqualTo(2));

      await cluster.close(drain: const Duration(seconds: 2));
    });

    test('stops serving once closed', () async {
      final cluster = await serveIsolates(
        buildClusterApp,
        InternetAddress.loopbackIPv4,
        0,
        isolates: 2,
      );
      final origin = 'http://${cluster.address.host}:${cluster.port}';
      await cluster.close(drain: const Duration(seconds: 2));

      await expectLater(
        http.get(Uri.parse('$origin/health')),
        throwsA(isA<Exception>()),
      );
    });

    test('refuses a cluster of no isolates', () {
      expect(
        () => serveIsolates(
          buildClusterApp,
          InternetAddress.loopbackIPv4,
          0,
          isolates: 0,
        ),
        throwsArgumentError,
      );
    });
  });
}

/// The `served` counter out of a `/count` body.
int countOf(String body) =>
    int.parse(RegExp(r'"served":(\d+)').firstMatch(body)!.group(1)!);

/// The isolate name out of a `/count` body.
String isolateOf(String body) =>
    RegExp(r'"isolate":"([^"]+)"').firstMatch(body)!.group(1)!;

/// Succeeds everywhere except the second spawned isolate.
///
/// Exercises the cleanup of isolates spawned *before* the one that failed:
/// with a factory that fails on the first spawn there are none to clean up,
/// which is how those lines stayed unreached.
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

void _deathTests() {
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
