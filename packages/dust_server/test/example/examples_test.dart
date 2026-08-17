import 'package:test/test.dart';

import '../../example/headers_and_host.dart' as headers_and_host;
import '../../example/hello_world.dart' as hello_world;
import '../../example/path_params.dart' as path_params;
import '../../example/query_params.dart' as query_params;
import '../../example/routing.dart' as routing;
import 'serve.dart';

/// One suite for every one-question example.
///
/// Each example gets the shortest test that proves the thing it demonstrates,
/// and they share one harness. Thirty examples with thirty harnesses is thirty
/// places for a leaked socket to hide; more to the point, an example nothing
/// serves is an example that quietly stops compiling.

void main() {
  group('hello_world', () {
    test('answers the root as text, not as a quoted JSON string', () async {
      final app = await example(hello_world.buildApp());

      final response = await app.get('/');

      expect(response.statusCode, 200);
      expect(response.body, 'Hello, world!');
      expect(response.headers['content-type'], startsWith('text/plain'));
    });

    test('reads the name out of the path', () async {
      final app = await example(hello_world.buildApp());

      expect((await app.get('/hello/ada')).body, 'Hello, ada!');
    });

    test('encodes a returned map as JSON without being asked', () async {
      final app = await example(hello_world.buildApp());

      final response = await app.get('/json');

      expect(response.headers['content-type'], 'application/json');
      expect(app.object(response), {'greeting': 'Hello, world!'});
    });

    test('an unmatched path answers 404 without a fallback', () async {
      final app = await example(hello_world.buildApp());

      expect((await app.get('/nope')).statusCode, 404);
    });
  });

  group('routing', () {
    test('nest serves the inner routes under the prefix', () async {
      final app = await example(routing.buildApp());

      expect(app.array(await app.get('/api/notes')), ['first', 'second']);
      expect(app.object(await app.get('/api/notes/7')), {'id': '7'});
    });

    test('two verbs chain onto one path', () async {
      final app = await example(routing.buildApp());

      expect((await app.get('/api/notes')).statusCode, 200);
      expect((await app.post('/api/notes', null)).statusCode, 201);
    });

    test('merge folds a router in at the same level', () async {
      final app = await example(routing.buildApp());

      expect(app.object(await app.get('/health')), {'status': 'ok'});
    });

    test('a known path with an unknown method answers 405 and Allow', () async {
      final app = await example(routing.buildApp());

      final response = await app.send('PUT', '/api/notes');

      expect(response.statusCode, 405);
      expect(response.headers['allow'], 'GET, HEAD, POST');
    });

    test('the fallback answers what matched nothing', () async {
      final app = await example(routing.buildApp());

      final response = await app.get('/nothing-here');

      expect(response.statusCode, 404);
      expect(app.object(response)['error'], 'no such route');
    });
  });

  group('path_params', () {
    test('a plain parameter coerces to the type asked for', () async {
      final app = await example(path_params.buildApp());

      expect(app.object(await app.get('/orders/41')), {'id': 41});
    });

    test('a value that will not coerce is a 400, not a 500', () async {
      final app = await example(path_params.buildApp());

      expect((await app.get('/orders/abc')).statusCode, 400);
    });

    test('coercion is base 10, so 0x10 is not 16', () async {
      final app = await example(path_params.buildApp());

      expect((await app.get('/orders/0x10')).statusCode, 400);
    });

    test('a constrained parameter refuses at the route, with 404', () async {
      // The difference from the case above: nothing matched, so no handler and
      // no extractor ever ran.
      final app = await example(path_params.buildApp());

      expect((await app.get('/strict/41')).statusCode, 200);
      expect((await app.get('/strict/abc')).statusCode, 404);
    });

    test('a catch-all keeps the slashes', () async {
      final app = await example(path_params.buildApp());

      expect(
        app.object(await app.get('/files/css/app.css')),
        {'path': 'css/app.css'},
      );
    });

    test('several parameters are read by name', () async {
      final app = await example(path_params.buildApp());

      expect(
        app.object(await app.get('/teams/dust/members/ada')),
        {'team': 'dust', 'member': 'ada'},
      );
    });
  });

  group('query_params', () {
    test('a required value is read and coerced', () async {
      final app = await example(query_params.buildApp());

      expect(
        app.object(await app.get('/search?q=shirt&page=2')),
        {'q': 'shirt', 'page': 2},
      );
    });

    test('a nullable type makes it optional', () async {
      final app = await example(query_params.buildApp());

      expect(app.object(await app.get('/search?q=shirt'))['page'], 1);
    });

    test('a missing required value is a 400', () async {
      final app = await example(query_params.buildApp());

      expect((await app.get('/search')).statusCode, 400);
    });

    test('queryList takes every value of a repeated key', () async {
      final app = await example(query_params.buildApp());

      expect(
        app.object(await app.get('/filter?tag=red&tag=blue'))['tags'],
        ['red', 'blue'],
      );
    });

    test('rawQuery hands back the undecoded string', () async {
      final app = await example(query_params.buildApp());

      expect(
        app.object(await app.get('/raw?a=1&b=%20two'))['query'],
        'a=1&b=%20two',
      );
    });
  });

  group('headers_and_host', () {
    test('a header is read, and an absent one is null', () async {
      final app = await example(headers_and_host.buildApp());

      expect(
        app.object(await app.get('/echo', headers: {'x-trace': 'abc'})),
        {'trace': 'abc'},
      );
      expect(app.object(await app.get('/echo')), {'trace': null});
    });

    test('every header can be read at once', () async {
      final app = await example(headers_and_host.buildApp());

      final all = app.object(await app.get('/all', headers: {'a': '1'}));

      expect((all['headers']! as Map)['a'], '1');
    });

    test('a host on the list is accepted', () async {
      final app = await example(headers_and_host.buildApp());

      expect((await app.get('/host')).statusCode, 200);
    });

    test('a forged host is refused, because Host is client input', () async {
      final app = await example(headers_and_host.buildApp());

      final response = await app.get(
        '/host',
        headers: {'host': 'evil.example'},
      );

      expect(response.statusCode, 400);
      expect(app.object(response)['error'], 'unrecognised host');
    });
  });
}
