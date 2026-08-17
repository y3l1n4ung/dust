import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../support.dart';

/// `layer` wraps every answer; `routeLayer` wraps only the ones a route
/// produced. The distinction decides what a 404 looks like: an auth layer in
/// the wrong slot turns "no such route" into "not authorised", which reads as
/// a permissions bug and sends people looking in the wrong place.

/// Records that it ran and lets the request through.
Middleware _record(List<String> log, String name) {
  return (Handler inner) {
    return (Request request) async {
      log.add(name);
      return inner(request);
    };
  };
}

/// Refuses everything it is given, the way an auth layer would.
Middleware _refuse() {
  return (Handler inner) {
    return (Request request) async =>
        const Rejection.unauthorized('no key').intoResponse();
  };
}

void main() {
  group('a route layer', () {
    test('runs for a request that matched a route', () async {
      final log = <String>[];
      final app = Router()
        ..routeLayer(_record(log, 'route'))
        ..route('/todos', get((request) async => {'ok': true}));

      await app.handler(request('GET', '/todos'));

      expect(log, ['route']);
    });

    test('does not run for a path nothing serves', () async {
      final log = <String>[];
      final app = Router()
        ..routeLayer(_record(log, 'route'))
        ..route('/todos', get((request) async => {'ok': true}));

      final response = await app.handler(request('GET', '/nothing'));

      expect(response.statusCode, 404);
      expect(log, isEmpty);
    });

    test('does not run for a method the path does not serve', () async {
      final log = <String>[];
      final app = Router()
        ..routeLayer(_record(log, 'route'))
        ..route('/todos', get((request) async => {'ok': true}));

      final response = await app.handler(request('DELETE', '/todos'));

      expect(response.statusCode, 405);
      expect(log, isEmpty);
    });

    test('does not run for the fallback', () async {
      final log = <String>[];
      final app = Router()
        ..routeLayer(_record(log, 'route'))
        ..route('/todos', get((request) async => {'ok': true}))
        ..fallback((request) async => Response.ok('fallback'));

      final response = await app.handler(request('GET', '/elsewhere'));

      expect(await response.readAsString(), 'fallback');
      expect(log, isEmpty);
    });

    test('leaves a 404 answerable when it would have refused', () async {
      // The reason to reach for `routeLayer` at all.
      final app = Router()
        ..routeLayer(_refuse())
        ..route('/todos', get((request) async => {'ok': true}));

      expect((await app.handler(request('GET', '/todos'))).statusCode, 401);
      expect((await app.handler(request('GET', '/nothing'))).statusCode, 404);
    });
  });

  group('an ordinary layer', () {
    test('runs for a path nothing serves', () async {
      final log = <String>[];
      final app = Router()
        ..layer(_record(log, 'all'))
        ..route('/todos', get((request) async => {'ok': true}));

      await app.handler(request('GET', '/nothing'));

      expect(log, ['all']);
    });

    test('runs for the fallback too', () async {
      final log = <String>[];
      final app = Router()
        ..layer(_record(log, 'all'))
        ..route('/todos', get((request) async => {'ok': true}))
        ..fallback((request) async => Response.ok('fallback'));

      await app.handler(request('GET', '/elsewhere'));

      expect(log, ['all']);
    });

    test('refuses a 404 as readily as a route, which is the trap', () async {
      final app = Router()
        ..layer(_refuse())
        ..route('/todos', get((request) async => {'ok': true}));

      // Both answer 401: the missing route is indistinguishable from a
      // forbidden one.
      expect((await app.handler(request('GET', '/todos'))).statusCode, 401);
      expect((await app.handler(request('GET', '/nothing'))).statusCode, 401);
    });
  });

  group('the two together', () {
    test('run outermost layer first, then the route layer', () async {
      final log = <String>[];
      final app = Router()
        ..layer(_record(log, 'outer'))
        ..routeLayer(_record(log, 'route'))
        ..route('/todos', get((request) async => {'ok': true}));

      await app.handler(request('GET', '/todos'));

      expect(log, ['outer', 'route']);
    });

    test('leave only the outer one on a miss', () async {
      final log = <String>[];
      final app = Router()
        ..layer(_record(log, 'outer'))
        ..routeLayer(_record(log, 'route'))
        ..route('/todos', get((request) async => {'ok': true}));

      await app.handler(request('GET', '/nothing'));

      expect(log, ['outer']);
    });
  });

  group('nesting', () {
    test('a route layer on a nested router covers its own routes', () async {
      final log = <String>[];
      final inner = Router()
        ..routeLayer(_record(log, 'inner'))
        ..route('/{id}', get((request) async => {'ok': true}));

      final app = Router()
        ..nest('/todos', inner)
        ..route('/health', get((request) async => {'ok': true}));

      await app.handler(request('GET', '/todos/7'));
      expect(log, ['inner']);

      log.clear();
      await app.handler(request('GET', '/health'));
      expect(log, isEmpty);
    });

    test('applies in declaration order down the tree', () async {
      final log = <String>[];
      final inner = Router()
        ..routeLayer(_record(log, 'inner'))
        ..route('/{id}', get((request) async => {'ok': true}));

      final app = Router()
        ..routeLayer(_record(log, 'outer'))
        ..nest('/todos', inner);

      await app.handler(request('GET', '/todos/7'));

      expect(log, ['outer', 'inner']);
    });

    test('takes a Layer as readily as a Middleware', () async {
      final app = Router()
        ..routeLayer(const RequestId())
        ..route('/todos', get((request) async => {'ok': true}));

      final matched = await app.handler(request('GET', '/todos'));
      final missed = await app.handler(request('GET', '/nothing'));

      expect(matched.headers['x-request-id'], isNotNull);
      expect(missed.headers, isNot(contains('x-request-id')));
    });
  });

  group('sealing', () {
    test('refuses a route layer after the handler was built', () {
      final app = Router()
        ..route('/todos', get((request) async => {'ok': true}));
      app.handler;

      expect(() => app.routeLayer(const RequestId()), throwsStateError);
    });
  });
}
