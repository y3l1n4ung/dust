import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../support.dart';

/// `layer` on a *nested* router covers everything that router answers, the
/// 404s and 405s inside its prefix included.
///
/// It used to be folded into each route's chain, which made it run only for
/// routes that matched — indistinguishable from `routeLayer`. The visible cost
/// was that a layer which rewrites a path so it *can* match never got the
/// chance: `NormalizePath` inside a `nest` silently did nothing, and the
/// request 404s with nothing to explain it.

/// Records that it ran and lets the request through.
Middleware _record(List<String> log, String name) {
  return (Handler inner) {
    return (Request request) async {
      log.add(name);
      return inner(request);
    };
  };
}

Map<String, Object?> _ok(Request request) => const {'ok': true};

void main() {
  group('a layer on a nested router', () {
    test('runs for a route inside its prefix', () async {
      final log = <String>[];
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(_record(log, 'api'))
            ..route('/notes', get(_ok)),
        );

      expect((await app.handler(request('GET', '/api/notes'))).statusCode, 200);
      expect(log, ['api']);
    });

    test('runs for a 404 inside its prefix', () async {
      final log = <String>[];
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(_record(log, 'api'))
            ..route('/notes', get(_ok)),
        );

      expect(
          (await app.handler(request('GET', '/api/nothing'))).statusCode, 404);
      expect(log, ['api']);
    });

    test('runs for a 405 inside its prefix', () async {
      final log = <String>[];
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(_record(log, 'api'))
            ..route('/notes', get(_ok)),
        );

      expect((await app.handler(request('PUT', '/api/notes'))).statusCode, 405);
      expect(log, ['api']);
    });

    test('does not run for a path outside its prefix', () async {
      final log = <String>[];
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(_record(log, 'api'))
            ..route('/notes', get(_ok)),
        )
        ..route('/health', get(_ok));

      expect((await app.handler(request('GET', '/health'))).statusCode, 200);
      expect(log, isEmpty);
    });

    test('treats the prefix as whole segments, so /apiary is outside /api',
        () async {
      final log = <String>[];
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(_record(log, 'api'))
            ..route('/notes', get(_ok)),
        )
        ..route('/apiary', get(_ok));

      expect((await app.handler(request('GET', '/apiary'))).statusCode, 200);
      expect(log, isEmpty);
    });

    test('runs once, not once per enclosing router', () async {
      final log = <String>[];
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(_record(log, 'api'))
            ..route('/notes', get(_ok)),
        );

      await app.handler(request('GET', '/api/notes'));

      expect(log, hasLength(1));
    });

    test('nests, outermost prefix outermost', () async {
      final log = <String>[];
      final app = Router()
        ..layer(_record(log, 'root'))
        ..nest(
          '/a',
          Router()
            ..layer(_record(log, 'a'))
            ..nest(
              '/b',
              Router()
                ..layer(_record(log, 'b'))
                ..route('/notes', get(_ok)),
            ),
        );

      await app.handler(request('GET', '/a/b/notes'));

      expect(log, ['root', 'a', 'b']);
    });

    test('a merged router covers everything, because merge is the same level',
        () async {
      // `merge` is `nest('')`, so its layer has no prefix to be scoped by.
      final log = <String>[];
      final app = Router()
        ..merge(Router()..layer(_record(log, 'merged')))
        ..route('/health', get(_ok));

      expect((await app.handler(request('GET', '/health'))).statusCode, 200);
      expect((await app.handler(request('GET', '/nothing'))).statusCode, 404);
      expect(log, ['merged', 'merged']);
    });

    test('a path-rewriting layer runs early enough for the path to match',
        () async {
      // The bug this fixes. `/api/notes/` matches nothing until the layer
      // rewrites it, and folding the layer into route chains meant it only ran
      // after a match — so it never ran at all.
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(const NormalizePath())
            ..route('/notes', get(_ok)),
        );

      expect(
          (await app.handler(request('GET', '/api/notes/'))).statusCode, 200);
      expect((await app.handler(request('GET', '/api/notes'))).statusCode, 200);
    });

    test('a redirecting rewrite keeps the prefix in the Location', () async {
      final app = Router()
        ..nest(
          '/api',
          Router()
            ..layer(const NormalizePath.redirecting())
            ..route('/notes', get(_ok)),
        );

      final response = await app.handler(request('GET', '/api/notes/'));

      expect(response.statusCode, 308);
      expect(response.headers['location'], '/api/notes');
    });

    test(
        'a guard on a nested router refuses its 404 too, which is why '
        'routeLayer exists', () async {
      // Not a defect — the documented difference. `layer` is enforcement over
      // everything below; `routeLayer` is enforcement over matched routes.
      final app = Router()
        ..nest(
          '/admin',
          Router()
            ..layer((inner) => (request) async =>
                const Rejection.unauthorized('no key').intoResponse())
            ..route('/orders', get(_ok)),
        );

      expect(
          (await app.handler(request('GET', '/admin/orders'))).statusCode, 401);
      expect(
          (await app.handler(request('GET', '/admin/typo'))).statusCode, 401);
    });
  });
}
