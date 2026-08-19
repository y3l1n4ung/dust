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
  group('serveCluster', () {
    test('serves from one isolate when asked for one', () async {
      final cluster = await serveCluster(
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
      final cluster = await serveCluster(
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
      final cluster = await serveCluster(
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
      final cluster = await serveCluster(
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
      final cluster = await serveCluster(
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
        () => serveCluster(
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
