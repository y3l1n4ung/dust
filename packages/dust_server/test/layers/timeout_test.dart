import 'dart:async';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('RequestTimeout', () {
    test('lets a fast handler through untouched', () async {
      final app = Router()
        ..layer(const RequestTimeout(Duration(seconds: 5)))
        ..route('/a', get((request) async => textResponse('fast')));

      final response = await app.handler(request('GET', '/a'));

      expect(response.statusCode, 200);
      expect(await response.readAsString(), 'fast');
    });

    test('answers 503 when the budget runs out', () async {
      final app = Router()
        ..layer(const RequestTimeout(Duration(milliseconds: 20)))
        ..route('/slow', get((request) async {
          await Future<void>.delayed(const Duration(seconds: 5));
          return textResponse('too late');
        }));

      final response = await app.handler(request('GET', '/slow'));

      expect(response.statusCode, 503);
      expect(await response.readAsString(), contains('20ms'));
    });

    test('reports the timeout to the callback', () async {
      var reported = 0;
      final app = Router()
        ..layer(RequestTimeout(
          const Duration(milliseconds: 20),
          onTimeout: (request) => reported++,
        ))
        ..route('/slow', get((request) async {
          await Future<void>.delayed(const Duration(seconds: 5));
          return noContent();
        }));

      await app.handler(request('GET', '/slow'));

      expect(reported, 1);
    });

    test('does not fire for a handler that finishes in time', () async {
      var reported = 0;
      final app = Router()
        ..layer(RequestTimeout(
          const Duration(seconds: 5),
          onTimeout: (request) => reported++,
        ))
        ..route('/a', get((request) async => noContent()));

      await app.handler(request('GET', '/a'));

      expect(reported, 0);
    });

    test('covers a route that was never matched', () async {
      final app = Router()
        ..layer(const RequestTimeout(Duration(seconds: 5)))
        ..route('/a', get((request) async => noContent()));

      expect((await app.handler(request('GET', '/nope'))).statusCode, 404);
    });
  });
}
