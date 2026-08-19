import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Adversarial and pathological input. The route table is written by the
/// application author, but the request path, method, and headers are not.

Handler _label(String label) => (request) async => textResponse(label);

const _routes = <String>[
  '/',
  '/a',
  '/a/{id}',
  '/a/{id}/b',
  '/a/{id}/b/{other}',
  '/x/y/z',
];

Router _ours() {
  final app = Router();
  for (final route in _routes) {
    app.route(route, get(_label(route)));
  }
  return app;
}

void main() {
  group('pathological input', () {
    test('a very long path does not hang', () async {
      final app = _ours().handler;
      final path = '/${'a/' * 5000}b';

      final stopwatch = Stopwatch()..start();
      final response = await app(request('GET', path));
      stopwatch.stop();

      expect(response.statusCode, 404);
      expect(stopwatch.elapsedMilliseconds, lessThan(1000));
    });

    test('a very long single segment does not hang', () async {
      final app = _ours().handler;

      final stopwatch = Stopwatch()..start();
      final response = await app(request('GET', '/a/${'x' * 100000}'));
      stopwatch.stop();

      expect(response.statusCode, 200);
      expect(stopwatch.elapsedMilliseconds, lessThan(1000));
    });

    test('two hundred routes stay responsive', () async {
      final app = Router();
      for (var i = 0; i < 200; i++) {
        app.route('/route$i/{id}', get(_label('route$i')));
      }
      final handler = app.handler;

      final stopwatch = Stopwatch()..start();
      for (var i = 0; i < 1000; i++) {
        await handler(request('GET', '/route199/7'));
      }
      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(2000));
    });
  });
}
