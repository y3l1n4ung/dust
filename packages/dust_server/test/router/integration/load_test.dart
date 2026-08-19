import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:test/test.dart';

import '../support.dart';

/// Concurrency and throughput. The router is built once and then read by every
/// request, so the thing to prove is that nothing it holds is mutated per
/// request.

final class _Counter {
  int value = 0;
}

void main() {
  group('under concurrent load', () {
    late HttpServer server;
    late String origin;
    late _Counter counter;

    setUp(() async {
      counter = _Counter();
      final app = Router()
        ..route(
          '/count/{id}',
          get((request) async {
            counter.value++;
            final id =
                await const PathExtractable<String>('id').extract(request);
            return switch (id) {
              Ok(:final value) => textResponse(value),
              Err(:final error) => error.intoResponse(),
            };
          }),
        )
        ..withState(counter);

      server = await shelf_io.serve(
        app.handler,
        InternetAddress.loopbackIPv4,
        0,
      );
      origin = 'http://${server.address.host}:${server.port}';
    });

    tearDown(() => server.close(force: true));

    test('serves 200 concurrent requests without crossing parameters',
        () async {
      final responses = await Future.wait([
        for (var i = 0; i < 200; i++) http.get(Uri.parse('$origin/count/$i')),
      ]);

      for (var i = 0; i < responses.length; i++) {
        expect(responses[i].statusCode, 200);
        expect(responses[i].body, '$i');
      }
      expect(counter.value, 200);
    });

    test('keeps state readable from every concurrent request', () async {
      final responses = await Future.wait([
        for (var i = 0; i < 50; i++) http.get(Uri.parse('$origin/count/$i')),
      ]);

      expect(responses.every((response) => response.statusCode == 200), isTrue);
    });
  });

  group('throughput', () {
    test('matches a hundred-route table at a sane rate', () async {
      final app = Router();
      for (var i = 0; i < 100; i++) {
        app.route('/resource$i/{id}', get((request) async => noContent()));
      }
      final handler = app.handler;

      // Warm up, so the measurement is not dominated by first-call costs.
      for (var i = 0; i < 200; i++) {
        await handler(request('GET', '/resource50/7'));
      }

      const iterations = 2000;
      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < iterations; i++) {
        await handler(request('GET', '/resource99/7'));
      }
      stopwatch.stop();

      final perRequest = stopwatch.elapsedMicroseconds / iterations;
      // ignore: avoid_print
      print('worst-case match over 100 routes: '
          '${perRequest.toStringAsFixed(1)}us/request');

      expect(perRequest, lessThan(500));
    });

    test('scales to a thousand routes', () async {
      final app = Router();
      for (var i = 0; i < 1000; i++) {
        app.route('/resource$i/{id}', get((request) async => noContent()));
      }
      final handler = app.handler;

      for (var i = 0; i < 200; i++) {
        await handler(request('GET', '/resource500/7'));
      }

      const iterations = 2000;
      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < iterations; i++) {
        await handler(request('GET', '/resource999/7'));
      }
      stopwatch.stop();

      final perRequest = stopwatch.elapsedMicroseconds / iterations;
      // ignore: avoid_print
      print('worst-case match over 1000 routes: '
          '${perRequest.toStringAsFixed(1)}us/request');

      // Bucketing by first segment means the table size barely shows up.
      expect(perRequest, lessThan(500));
    });

    test('scales when every route shares a prefix', () async {
      // The realistic shape: one API, everything under it. Bucketing by the
      // whole literal prefix rather than the first segment is what keeps this
      // from degrading into a scan of the entire table.
      final app = Router();
      for (var i = 0; i < 500; i++) {
        app.route('/api/resource$i/{id}', get((request) async => noContent()));
      }
      final handler = app.handler;

      for (var i = 0; i < 200; i++) {
        await handler(request('GET', '/api/resource499/7'));
      }

      const iterations = 2000;
      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < iterations; i++) {
        await handler(request('GET', '/api/resource499/7'));
      }
      stopwatch.stop();

      final perRequest = stopwatch.elapsedMicroseconds / iterations;
      // ignore: avoid_print
      print('500 routes under one prefix: '
          '${perRequest.toStringAsFixed(1)}us/request');

      expect(perRequest, lessThan(500));
    });

    test('serves a static route without touching a pattern', () async {
      final app = Router();
      for (var i = 0; i < 500; i++) {
        app.route('/static$i', get((request) async => noContent()));
      }
      final handler = app.handler;

      for (var i = 0; i < 200; i++) {
        await handler(request('GET', '/static499'));
      }

      const iterations = 2000;
      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < iterations; i++) {
        await handler(request('GET', '/static499'));
      }
      stopwatch.stop();

      final perRequest = stopwatch.elapsedMicroseconds / iterations;
      // ignore: avoid_print
      print('static match over 500 routes: '
          '${perRequest.toStringAsFixed(1)}us/request');

      expect(perRequest, lessThan(500));
    });
  });
}
