import 'package:dust_server/server.dart';
import 'package:shelf/shelf.dart' as shelf;
import 'package:shelf_router/shelf_router.dart' as shelf_router;
import 'package:test/test.dart';

import '../support.dart';

/// `mount` hands a subtree to somebody else's handler, the way
/// `shelf_router.mount` and axum's `nest_service` do.

Handler _echoPath() => (request) async => textResponse('[${request.url.path}]');

void main() {
  group('mount', () {
    test('serves everything below the prefix', () async {
      final app = Router()..mount('/assets', _echoPath());
      final handler = app.handler;

      expect((await handler(request('GET', '/assets/a.css'))).statusCode, 200);
      expect(
          (await handler(request('GET', '/assets/deep/b.js'))).statusCode, 200);
    });

    test('strips the prefix before the handler sees it', () async {
      final app = Router()..mount('/assets', _echoPath());

      expect(
        await (await app.handler(request('GET', '/assets/a.css')))
            .readAsString(),
        '[a.css]',
      );
    });

    test('hands the bare prefix over as an empty path', () async {
      final app = Router()..mount('/assets', _echoPath());

      expect(
        await (await app.handler(request('GET', '/assets'))).readAsString(),
        '[]',
      );
    });

    test('serves every method', () async {
      final app = Router()..mount('/assets', _echoPath());
      final handler = app.handler;

      expect((await handler(request('POST', '/assets/a'))).statusCode, 200);
      expect((await handler(request('DELETE', '/assets/a'))).statusCode, 200);
    });

    test('does not swallow a sibling path', () async {
      final app = Router()
        ..mount('/assets', _echoPath())
        ..route('/other', get(label('other')));
      final handler = app.handler;

      expect(
        await (await handler(request('GET', '/other'))).readAsString(),
        'other',
      );
      expect((await handler(request('GET', '/nope'))).statusCode, 404);
    });

    test('loses to an exact route declared first', () async {
      final app = Router()
        ..route('/assets/special', get(label('special')))
        ..mount('/assets', _echoPath());

      expect(
        await (await app.handler(request('GET', '/assets/special')))
            .readAsString(),
        'special',
      );
    });

    test('hosts a shelf_router as the mounted handler', () async {
      final inner = shelf_router.Router()
        ..get('/inner', (shelf.Request request) => shelf.Response.ok('inner'));
      final app = Router()..mount('/legacy', inner.call);

      expect(
        await (await app.handler(request('GET', '/legacy/inner')))
            .readAsString(),
        'inner',
      );
    });

    test('works under a nested router', () async {
      final api = Router()..mount('/assets', _echoPath());
      final app = Router()..nest('/api', api);

      expect(
        await (await app.handler(request('GET', '/api/assets/a.css')))
            .readAsString(),
        '[a.css]',
      );
    });

    test('refuses a mount after the handler was built', () {
      final app = Router()..route('/a', get(label('a')));
      app.handler;

      expect(() => app.mount('/late', _echoPath()), throwsStateError);
    });
  });
}
