import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Running an extractor as a layer, so one guard covers a whole module.
///
/// A layer can pass a request on or answer it — it cannot hand a value to a
/// handler. `fromExtractor` puts what it produced in the request context and
/// `Extension` reads it back, which is how axum's `from_extractor` and
/// `Extension<T>` fit together.

final class Caller {
  const Caller(this.id);

  final String id;
}

/// Accepts `Bearer <id>`, and counts how often it ran.
final class CountingAuth implements FromRequestParts<Caller> {
  CountingAuth();

  int runs = 0;

  @override
  Future<Result<Caller, Rejection>> extract(Request request) async {
    runs++;
    final header = request.headers['authorization'];
    if (header == null || !header.startsWith('Bearer ')) {
      return const Err(Rejection.unauthorized('no credential'));
    }
    if (header == 'Bearer intern') {
      return const Err(Rejection.forbidden('not allowed here'));
    }
    return Ok(Caller(header.substring(7)));
  }
}

Future<Map<String, Object?>> _whoAmI(Request request) async {
  final caller = await request.extract(const Extension<Caller>());

  return {'id': caller.id};
}

void main() {
  group('fromExtractor', () {
    test('passes the value to the handler', () async {
      final app = Router()
        ..routeLayer(fromExtractor(CountingAuth()))
        ..route('/me', get(_whoAmI));

      final response = await app.handler(
        request('GET', '/me', headers: {'authorization': 'Bearer ada'}),
      );

      expect(response.statusCode, 200);
      expect(await response.readAsString(), '{"id":"ada"}');
    });

    test('answers the rejection without reaching the handler', () async {
      var reached = false;
      final app = Router()
        ..routeLayer(fromExtractor(CountingAuth()))
        ..route('/me', get((request) async {
          reached = true;
          return const {'ok': true};
        }));

      final response = await app.handler(request('GET', '/me'));

      expect(response.statusCode, 401);
      expect(reached, isFalse);
    });

    test('keeps the extractor own status, so 403 stays 403', () async {
      final app = Router()
        ..routeLayer(fromExtractor(CountingAuth()))
        ..route('/me', get(_whoAmI));

      final response = await app.handler(
        request('GET', '/me', headers: {'authorization': 'Bearer intern'}),
      );

      expect(response.statusCode, 403);
    });

    test('runs once per request, not once per handler that wants it', () async {
      // The reason to hoist it: two routes, one extraction each request.
      final auth = CountingAuth();
      final app = Router()
        ..routeLayer(fromExtractor(auth))
        ..route('/me', get(_whoAmI))
        ..route('/also', get(_whoAmI));

      await app.handler(
        request('GET', '/me', headers: {'authorization': 'Bearer ada'}),
      );
      await app.handler(
        request('GET', '/also', headers: {'authorization': 'Bearer ada'}),
      );

      expect(auth.runs, 2, reason: 'once per request, not twice per request');
    });

    test('covers a route added later, which is the point', () async {
      final app = Router()
        ..routeLayer(fromExtractor(CountingAuth()))
        ..route('/me', get(_whoAmI))
        ..route('/added-later', get(_whoAmI));

      expect(
        (await app.handler(request('GET', '/added-later'))).statusCode,
        401,
      );
    });

    test('two types coexist, each read by its own Extension', () async {
      final app = Router()
        ..routeLayer(fromExtractor(CountingAuth()))
        ..routeLayer(fromExtractor(const _Tenant('acme')))
        ..route('/me', get((request) async {
          final caller = await request.extract(const Extension<Caller>());
          final tenant = await request.extract(const Extension<_TenantId>());
          return {'id': caller.id, 'tenant': tenant.value};
        }));

      final response = await app.handler(
        request('GET', '/me', headers: {'authorization': 'Bearer ada'}),
      );

      expect(await response.readAsString(), '{"id":"ada","tenant":"acme"}');
    });
  });

  group('as a routeLayer', () {
    test('an unmatched path is 404, not 401', () async {
      // A guard that answered 401 for a path that does not exist hides a typo
      // in your own route table behind a credential error.
      final app = Router()
        ..routeLayer(fromExtractor(CountingAuth()))
        ..route('/me', get(_whoAmI));

      expect((await app.handler(request('GET', '/typo'))).statusCode, 404);
    });
  });

  group('Extension', () {
    test('is a 500 when no layer produced the value', () async {
      // A wiring mistake, not a client error. A 401 would send whoever is
      // debugging it after a credential that was never the problem.
      final app = Router()..route('/me', get(_whoAmI));

      final response = await app.handler(request('GET', '/me'));

      expect(response.statusCode, 500);
    });

    test('names the missing type in the message', () async {
      final outcome =
          await const Extension<Caller>().extract(request('GET', '/me'));

      expect(expectStatus(outcome, 500).message, contains('Caller'));
      expect(expectStatus(outcome, 500).message, contains('fromExtractor'));
    });

    test('keys by type, so the wrong type is not returned', () async {
      final app = Router()
        ..routeLayer(fromExtractor(CountingAuth()))
        ..route('/me', get((request) async {
          final wrong = await request.extract(const Extension<_TenantId>());
          return {'value': wrong.value};
        }));

      final response = await app.handler(
        request('GET', '/me', headers: {'authorization': 'Bearer ada'}),
      );

      expect(response.statusCode, 500);
    });
  });
}

final class _TenantId {
  const _TenantId(this.value);

  final String value;
}

final class _Tenant implements FromRequestParts<_TenantId> {
  const _Tenant(this.value);

  final String value;

  @override
  Future<Result<_TenantId, Rejection>> extract(Request request) async =>
      Ok(_TenantId(value));
}
