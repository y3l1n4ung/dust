import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../support.dart';

/// axum spells a per-path fallback `MethodRouter::fallback`; here it is `any`,
/// which is defined as "every method no other handler on this path claims".
/// Same behaviour, and these pin it so the equivalence is not just a claim in
/// a comparison table.

void main() {
  group('any as a per-path fallback', () {
    test('answers a method no other handler claimed', () async {
      final app = Router()
        ..route(
          '/thing',
          get((request) async => {'verb': 'get'})
              .any((request) async => {'verb': 'other'}),
        );

      final response = await app.handler(request('DELETE', '/thing'));

      expect(response.statusCode, 200);
    });

    test('loses to a method that has its own handler', () async {
      final app = Router()
        ..route(
          '/thing',
          get((request) async => {'verb': 'get'})
              .any((request) async => {'verb': 'other'}),
        );

      final response = await app.handler(request('GET', '/thing'));

      expect(await response.readAsString(), contains('get'));
    });

    test('replaces the 405 that path would otherwise answer', () async {
      final withFallback = Router()
        ..route(
          '/thing',
          get((request) async => 1).any((request) async => 2),
        );
      final without = Router()..route('/thing', get((request) async => 1));

      expect((await withFallback.handler(request('PUT', '/thing'))).statusCode,
          200);
      expect((await without.handler(request('PUT', '/thing'))).statusCode, 405);
    });

    test('does not reach a different path', () async {
      final app = Router()
        ..route('/thing', get((request) async => 1).any((request) async => 2))
        ..route('/other', get((request) async => 3));

      expect((await app.handler(request('PUT', '/other'))).statusCode, 405);
    });

    test('leaves an unmatched path to the router fallback', () async {
      final app = Router()
        ..route('/thing', any((request) async => 1))
        ..fallback((request) async => Response.ok('router fallback'));

      final response = await app.handler(request('GET', '/elsewhere'));

      expect(await response.readAsString(), 'router fallback');
    });

    test('serves every method on its own', () async {
      final app = Router()
        ..route('/thing', any((request) async => {'ok': true}));

      for (final method in ['GET', 'POST', 'PUT', 'PATCH', 'DELETE']) {
        expect(
          (await app.handler(request(method, '/thing'))).statusCode,
          200,
          reason: method,
        );
      }
    });

    test('can be declared before the verb it defers to', () async {
      // Declaration order decides ties, and a specific method still wins.
      final app = Router()
        ..route(
          '/thing',
          any((request) async => {'verb': 'other'})
              .get((request) async => {'verb': 'get'}),
        );

      expect(
        await (await app.handler(request('GET', '/thing'))).readAsString(),
        contains('other'),
      );
    });
  });
}
