import 'dart:convert';
import 'dart:io';

import 'package:dust_server/server.dart';
import 'package:http/http.dart' as http;
import 'package:shelf/shelf.dart' as shelf;
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:shelf_router/shelf_router.dart' as shelf_router;
import 'package:test/test.dart';

import '../support.dart';

/// Proves the router is a plain `shelf` citizen: it serves over a real socket,
/// composes inside a `shelf` pipeline, hosts hand-written `shelf` handlers, and
/// can itself be mounted inside somebody else's router.

Router _app() {
  return Router()
    ..route('/hello', get((request) async => textResponse('hello')))
    ..route(
      '/echo/{word}',
      get((request) async {
        final word =
            await const PathExtractable<String>('word').extract(request);
        return jsonResponse({'word': expectOkValue(word)});
      }),
    )
    ..route('/created', post((request) async => jsonResponse({}, status: 201)));
}

/// Unwraps without pulling the test matchers into a server context.
T expectOkValue<T>(Result<T, Rejection> result) {
  return switch (result) {
    Ok(:final value) => value,
    Err(:final error) => throw StateError('unexpected rejection: $error'),
  };
}

void main() {
  group('over a real socket', () {
    late HttpServer server;
    late String origin;

    setUp(() async {
      server =
          await shelf_io.serve(_app().handler, InternetAddress.loopbackIPv4, 0);
      origin = 'http://${server.address.host}:${server.port}';
    });

    tearDown(() => server.close(force: true));

    test('serves a GET', () async {
      final response = await http.get(Uri.parse('$origin/hello'));

      expect(response.statusCode, 200);
      expect(response.body, 'hello');
    });

    test('captures a path parameter from the wire', () async {
      final response = await http.get(Uri.parse('$origin/echo/world'));

      expect(jsonDecode(response.body), {'word': 'world'});
    });

    test('decodes a percent-encoded segment from the wire', () async {
      final response = await http.get(Uri.parse('$origin/echo/a%20b'));

      expect(jsonDecode(response.body), {'word': 'a b'});
    });

    test('serves a POST with its declared status', () async {
      final response = await http.post(Uri.parse('$origin/created'));

      expect(response.statusCode, 201);
    });

    test('answers an unknown path with 404', () async {
      final response = await http.get(Uri.parse('$origin/nope'));

      expect(response.statusCode, 404);
    });

    test('answers a wrong method with 405 and Allow', () async {
      final response = await http.delete(Uri.parse('$origin/hello'));

      expect(response.statusCode, 405);
      expect(response.headers['allow'], 'GET, HEAD');
    });

    test('serves HEAD from the GET route', () async {
      final response = await http.head(Uri.parse('$origin/hello'));

      expect(response.statusCode, 200);
    });
  });

  group('inside a shelf pipeline', () {
    test('runs behind shelf middleware', () async {
      final handler = const shelf.Pipeline()
          .addMiddleware(
            shelf.createMiddleware(
              responseHandler: (response) =>
                  response.change(headers: {'x-wrapped': 'yes'}),
            ),
          )
          .addHandler(_app().handler);

      final response = await handler(request('GET', '/hello'));

      expect(response.headers['x-wrapped'], 'yes');
      expect(await response.readAsString(), 'hello');
    });

    test('accepts shelf middleware through layer', () async {
      final app = _app()
        ..layer(
          shelf.createMiddleware(
            responseHandler: (response) =>
                response.change(headers: {'x-layered': 'yes'}),
          ),
        );

      final response = await app.handler(request('GET', '/hello'));

      expect(response.headers['x-layered'], 'yes');
    });

    test('hosts a hand-written shelf handler as a route', () async {
      final app = Router()
        ..route('/plain', get((request) async => shelf.Response.ok('plain')));

      final response = await app.handler(request('GET', '/plain'));

      expect(await response.readAsString(), 'plain');
    });

    test('mounts inside another router', () async {
      final outer = shelf_router.Router()
        ..mount('/dust/', _app().handler)
        ..get(
            '/native', (shelf.Request request) => shelf.Response.ok('native'));

      expect(
        await (await outer.call(request('GET', '/dust/hello'))).readAsString(),
        'hello',
      );
      expect(
        await (await outer.call(request('GET', '/native'))).readAsString(),
        'native',
      );
    });

    test('falls back to a shelf Cascade when nothing matches', () async {
      final cascade = shelf.Cascade()
          .add(_app().handler)
          .add((request) async => shelf.Response.ok('fallback'));

      expect(
        await (await cascade.handler(request('GET', '/hello'))).readAsString(),
        'hello',
      );
      expect(
        await (await cascade.handler(request('GET', '/nope'))).readAsString(),
        'fallback',
      );
    });
  });
}
