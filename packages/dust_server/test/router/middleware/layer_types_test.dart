import 'package:dust_server/server.dart';
import 'package:shelf/shelf.dart' as shelf;
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('layer types', () {
    test('accepts a const Layer', () async {
      final app = Router()
        ..layer(const HeaderLayer('x-const', 'yes'))
        ..route('/a', get(label('a')));

      expect(
        (await app.handler(request('GET', '/a'))).headers['x-const'],
        'yes',
      );
    });

    test('accepts a bare shelf Middleware', () async {
      final app = Router()
        ..layer(
          shelf.createMiddleware(
            responseHandler: (response) =>
                response.change(headers: {'x-shelf': 'yes'}),
          ),
        )
        ..route('/a', get(label('a')));

      expect(
        (await app.handler(request('GET', '/a'))).headers['x-shelf'],
        'yes',
      );
    });

    test('accepts both on one router', () async {
      final app = Router()
        ..layer(const HeaderLayer('x-const', 'yes'))
        ..layer(
          shelf.createMiddleware(
            responseHandler: (response) =>
                response.change(headers: {'x-shelf': 'yes'}),
          ),
        )
        ..route('/a', get(label('a')));

      final response = await app.handler(request('GET', '/a'));

      expect(response.headers['x-const'], 'yes');
      expect(response.headers['x-shelf'], 'yes');
    });

    test('rejects anything else when the handler is built', () {
      final app = Router()
        ..layer('nonsense')
        ..route('/a', get(label('a')));

      expect(() => app.handler, throwsArgumentError);
    });

    test('names the offending type in the error', () {
      final app = Router()
        ..layer(42)
        ..route('/a', get(label('a')));

      expect(
        () => app.handler,
        throwsA(
          isA<ArgumentError>().having(
            (error) => error.message,
            'message',
            contains('int'),
          ),
        ),
      );
    });
  });
}
