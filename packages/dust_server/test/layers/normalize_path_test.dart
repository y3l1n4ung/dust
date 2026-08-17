import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// `/todos` and `/todos/` are different paths to the router, on purpose — a
/// table that matched both would hide a duplicate. This layer settles the
/// question once at the edge, and the root has to survive it.

void main() {
  Router routes() => Router()
    ..route('/', get((request) async => {'at': 'root'}))
    ..route('/todos', get((request) async => {'at': 'todos'}))
    ..route('/todos/{id}', get((request) async => {'at': 'one'}));

  Future<Response> send(Layer layer, String path, {String method = 'GET'}) {
    final app = Router()
      ..layer(layer)
      ..merge(routes());
    return Future.sync(() => app.handler(request(method, path)));
  }

  group('rewriting, stripping the slash', () {
    const layer = NormalizePath();

    test('serves a path that carries one', () async {
      final response = await send(layer, '/todos/');

      expect(response.statusCode, 200);
      expect(await response.readAsString(), contains('todos'));
    });

    test('leaves a path that does not', () async {
      expect((await send(layer, '/todos')).statusCode, 200);
    });

    test('leaves the root alone, which is already canonical', () async {
      // Stripping the root's slash would leave an empty path nothing matches.
      final response = await send(layer, '/');

      expect(response.statusCode, 200);
      expect(await response.readAsString(), contains('root'));
    });

    test('keeps the path parameters of the route it lands on', () async {
      final response = await send(layer, '/todos/7/');

      expect(await response.readAsString(), contains('one'));
    });

    test('still 404s a path nothing serves', () async {
      expect((await send(layer, '/nothing/')).statusCode, 404);
    });

    test('does not redirect, so there is one round trip', () async {
      expect((await send(layer, '/todos/')).statusCode, 200);
    });
  });

  group('rewriting, appending the slash', () {
    const layer = NormalizePath(slash: TrailingSlash.append);

    test('serves a path that lacks one', () async {
      final app = Router()
        ..layer(layer)
        ..route('/todos/', get((request) async => {'at': 'todos'}));

      final response = await Future.sync(
        () => app.handler(request('GET', '/todos')),
      );

      expect(response.statusCode, 200);
    });

    test('leaves the root alone', () async {
      final app = Router()
        ..layer(layer)
        ..route('/', get((request) async => {'at': 'root'}));

      expect(
        (await Future.sync(() => app.handler(request('GET', '/')))).statusCode,
        200,
      );
    });
  });

  group('redirecting', () {
    const layer = NormalizePath.redirecting();

    test('answers 308 rather than serving it', () async {
      final response = await send(layer, '/todos/');

      expect(response.statusCode, 308);
    });

    test('sends the client to the canonical path', () async {
      final response = await send(layer, '/todos/');

      expect(response.headers['location'], '/todos');
    });

    test('keeps the method, which a 301 would not', () async {
      // A POST that became a GET would lose the body silently.
      final response = await send(layer, '/todos/', method: 'POST');

      expect(response.statusCode, 308);
    });

    test('keeps the query string', () async {
      final app = Router()
        ..layer(layer)
        ..merge(routes());

      final response = await Future.sync(
        () => app.handler(request('GET', '/todos/?done=true')),
      );

      expect(response.headers['location'], '/todos?done=true');
    });

    test('leaves an already-canonical path alone', () async {
      expect((await send(layer, '/todos')).statusCode, 200);
    });

    test('leaves the root alone', () async {
      expect((await send(layer, '/')).statusCode, 200);
    });
  });
}
