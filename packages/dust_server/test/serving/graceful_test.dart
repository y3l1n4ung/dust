import 'dart:async';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

void main() {
  group('serve', () {
    test('serves over a real socket', () async {
      final app = Router()
        ..route('/hello', get((request) async => textResponse('hello')));
      final server = await serve(app, InternetAddress.loopbackIPv4, 0);

      final response = await http.get(
        Uri.parse('http://${server.address.host}:${server.port}/hello'),
      );

      expect(response.body, 'hello');
      await server.close();
    });

    test('reports nothing in flight when idle', () async {
      final app = Router()..route('/a', get((request) async => noContent()));
      final server = await serve(app, InternetAddress.loopbackIPv4, 0);

      expect(server.inFlight, 0);
      await server.close();
    });

    test('counts a request while it is being handled', () async {
      final started = Completer<void>();
      final release = Completer<void>();
      final app = Router()
        ..route('/slow', get((request) async {
          started.complete();
          await release.future;
          return noContent();
        }));
      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      final origin = 'http://${server.address.host}:${server.port}';

      final pending = http.get(Uri.parse('$origin/slow'));
      await started.future;

      expect(server.inFlight, 1);

      release.complete();
      await pending;
      await server.close();
    });

    test('waits for an in-flight request before reporting closed', () async {
      final started = Completer<void>();
      final release = Completer<void>();
      var finished = false;
      final app = Router()
        ..route('/slow', get((request) async {
          started.complete();
          await release.future;
          finished = true;
          return noContent();
        }));
      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      final origin = 'http://${server.address.host}:${server.port}';

      final pending = http.get(Uri.parse('$origin/slow'));
      await started.future;

      final closing = server.close(drain: const Duration(seconds: 5));
      await Future<void>.delayed(const Duration(milliseconds: 20));
      expect(finished, isFalse, reason: 'close must not abandon the request');

      release.complete();
      expect(await closing, isTrue);
      expect(finished, isTrue);
      await pending;
    });

    test('reports false when the drain deadline passes', () async {
      final started = Completer<void>();
      final release = Completer<void>();
      final app = Router()
        ..route('/slow', get((request) async {
          started.complete();
          await release.future;
          return noContent();
        }));
      final server = await serve(app, InternetAddress.loopbackIPv4, 0);
      final origin = 'http://${server.address.host}:${server.port}';

      final pending = http.get(Uri.parse('$origin/slow'));
      await started.future;

      expect(
        await server.close(drain: const Duration(milliseconds: 50)),
        isFalse,
      );

      release.complete();
      await pending;
    });

    test('returns immediately when nothing is in flight', () async {
      final app = Router()..route('/a', get((request) async => noContent()));
      final server = await serve(app, InternetAddress.loopbackIPv4, 0);

      expect(await server.close(drain: const Duration(seconds: 5)), isTrue);
    });
  });
}
