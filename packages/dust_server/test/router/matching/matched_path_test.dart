import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../support.dart';

/// The route that matched, not the path that was asked for. Everything that
/// labels a request — a span name, a metric, a log line — wants the pattern:
/// `/todos/{id}` is one operation, and `/todos/7` is one request.
///
/// Getting this wrong is not visible in a response, only in a dashboard with a
/// million distinct operations, so it is pinned here.

void main() {
  group('a matched route', () {
    test('is the pattern, not the path', () async {
      String? seen;
      final app = Router()
        ..route('/todos/{id}', get((request) async {
          seen = matchedPathOf(request);
          return {'ok': true};
        }));

      await app.handler(request('GET', '/todos/7'));

      expect(seen, '/todos/{id}');
    });

    test('carries the whole nested prefix', () async {
      String? seen;
      final app = Router()
        ..nest(
          '/api/v1',
          Router()
            ..nest(
              '/todos',
              Router()
                ..route('/{id}', get((request) async {
                  seen = matchedPathOf(request);
                  return {'ok': true};
                })),
            ),
        );

      await app.handler(request('GET', '/api/v1/todos/7'));

      expect(seen, '/api/v1/todos/{id}');
    });

    test('is the literal path when the route has no parameters', () async {
      String? seen;
      final app = Router()
        ..route('/health', get((request) async {
          seen = matchedPathOf(request);
          return {'ok': true};
        }));

      await app.handler(request('GET', '/health'));

      expect(seen, '/health');
    });

    test('keeps a constraint in the pattern', () async {
      String? seen;
      final app = Router()
        ..route(r'/files/{name|.+}', get((request) async {
          seen = matchedPathOf(request);
          return {'ok': true};
        }));

      await app.handler(request('GET', '/files/a/b/c'));

      expect(seen, r'/files/{name|.+}');
    });

    test('keeps a wildcard in the pattern', () async {
      String? seen;
      final app = Router()
        ..route('/assets/{*rest}', get((request) async {
          seen = matchedPathOf(request);
          return {'ok': true};
        }));

      await app.handler(request('GET', '/assets/css/app.css'));

      expect(seen, '/assets/{*rest}');
    });

    test('is the same for two different requests to one route', () async {
      final seen = <String?>[];
      final app = Router()
        ..route('/todos/{id}', get((request) async {
          seen.add(matchedPathOf(request));
          return {'ok': true};
        }));

      await app.handler(request('GET', '/todos/1'));
      await app.handler(request('GET', '/todos/99999'));

      expect(seen, ['/todos/{id}', '/todos/{id}']);
      expect(seen.toSet(), hasLength(1));
    });

    test('distinguishes two routes that share a prefix', () async {
      final seen = <String?>[];
      Future<Object?> record(Request request) async {
        seen.add(matchedPathOf(request));
        return {'ok': true};
      }

      final app = Router()
        ..route('/todos/{id}', get(record))
        ..route('/todos/count', get(record));

      await app.handler(request('GET', '/todos/count'));
      await app.handler(request('GET', '/todos/7'));

      expect(seen, ['/todos/{id}', '/todos/{id}']);
    });
  });

  group('when nothing matched', () {
    test('a fallback sees no route, because none claimed the request',
        () async {
      String? seen = 'unset';
      final app = Router()
        ..route('/todos', get((request) async => {'ok': true}))
        ..fallback((request) async {
          seen = matchedPathOf(request);
          return Response.notFound('gone');
        });

      await app.handler(request('GET', '/nothing'));

      expect(seen, isNull);
    });

    test('a request built without a router sees none either', () {
      expect(matchedPathOf(request('GET', '/todos/7')), isNull);
    });
  });

  group('from a request extension', () {
    test('is reachable the same way as anything else', () async {
      String? seen;
      final app = Router()
        ..route('/todos/{id}', get((request) async {
          seen = matchedPathOf(request);
          final id = await request.path<int>('id');
          return {'id': id};
        }));

      await app.handler(request('GET', '/todos/7'));

      expect(seen, '/todos/{id}');
    });
  });

  group('a mounted handler', () {
    test('sees the mount point as the route', () async {
      String? seen;
      final app = Router()
        ..mount('/legacy', (request) async {
          seen = matchedPathOf(request);
          return Response.ok('inner');
        });

      await app.handler(request('GET', '/legacy/deep/path'));

      expect(seen, '/legacy');
    });
  });
}
