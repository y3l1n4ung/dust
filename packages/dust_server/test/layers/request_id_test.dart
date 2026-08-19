import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('RequestId', () {
    test('assigns an id and echoes it', () async {
      final app = Router()
        ..layer(const RequestId())
        ..route('/a', get((request) async => noContent()));

      final response = await app.handler(request('GET', '/a'));

      expect(response.headers['x-request-id'], isNotEmpty);
    });

    test('keeps an id the caller supplied', () async {
      final app = Router()
        ..layer(const RequestId())
        ..route('/a', get((request) async => noContent()));

      final response = await app.handler(
        request('GET', '/a', headers: {'x-request-id': 'from-proxy'}),
      );

      expect(response.headers['x-request-id'], 'from-proxy');
    });

    test('makes the id readable inside the handler', () async {
      String? seen;
      final app = Router()
        ..layer(const RequestId())
        ..route('/a', get((request) async {
          seen = requestIdOf(request);
          return noContent();
        }));

      await app.handler(request('GET', '/a', headers: {'x-request-id': 'abc'}));

      expect(seen, 'abc');
    });

    test('reads a header name of its own', () async {
      final app = Router()
        ..layer(const RequestId(header: 'x-correlation-id'))
        ..route('/a', get((request) async => noContent()));

      final response = await app.handler(
        request('GET', '/a', headers: {'x-correlation-id': 'corr'}),
      );

      expect(response.headers['x-correlation-id'], 'corr');
    });

    test('uses a generator of its own', () async {
      final app = Router()
        ..layer(RequestId(generate: () => 'fixed'))
        ..route('/a', get((request) async => noContent()));

      expect(
        (await app.handler(request('GET', '/a'))).headers['x-request-id'],
        'fixed',
      );
    });

    test('gives different requests different ids', () async {
      final app = Router()
        ..layer(const RequestId())
        ..route('/a', get((request) async => noContent()));
      final handler = app.handler;

      final first =
          (await handler(request('GET', '/a'))).headers['x-request-id'];
      final second =
          (await handler(request('GET', '/a'))).headers['x-request-id'];

      expect(first, isNot(second));
    });

    test('reports null when no layer ran', () async {
      String? seen = 'unset';
      final app = Router()
        ..route('/a', get((request) async {
          seen = requestIdOf(request);
          return noContent();
        }));

      await app.handler(request('GET', '/a'));

      expect(seen, isNull);
    });
  });
}
