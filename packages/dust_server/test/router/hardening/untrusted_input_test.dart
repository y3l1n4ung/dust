import 'dart:convert';

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
  group('untrusted request fields', () {
    test('a method with control characters cannot forge headers', () async {
      final app = _ours().handler;

      final response = await app(request('TRACE\r\nX-Injected: yes', '/a'));

      expect(response.statusCode, 405);
      expect(response.headers.containsKey('x-injected'), isFalse);
      expect(response.headers['allow'], 'GET, HEAD');
    });

    test('a path with control characters is escaped in the body', () async {
      final app = _ours().handler;

      final response =
          await app(request('GET', '/nope%0d%0aX-Injected:%20yes'));

      expect(response.statusCode, 404);
      expect(response.headers.containsKey('x-injected'), isFalse);
      expect(
        jsonDecode(await response.readAsString()),
        isA<Map<String, Object?>>(),
      );
    });

    test('a traversal segment is normalized before matching', () async {
      final app = _ours().handler;

      // Dart's Uri.parse resolves `..` and `%2e%2e` while parsing, so the path
      // reaching any router is already collapsed. shelf_router answers the
      // same way. Nothing here touches a filesystem, so this is only about
      // which route runs, and the normalized path picks it.
      final response = await app(request('GET', '/a/../x/y/z'));

      expect(response.statusCode, 200);
      expect(await response.readAsString(), '/x/y/z');
    });

    test('an encoded traversal cannot reach a different route', () async {
      final app = _ours().handler;

      final response = await app(request('GET', '/a/%2e%2e/x/y/z'));

      expect(await response.readAsString(), '/x/y/z');
    });

    test('an unknown method on an unknown path is still 404', () async {
      final app = _ours().handler;

      expect((await app(request('PROPFIND', '/nope'))).statusCode, 404);
    });

    test('a lowercase method still matches', () async {
      final app = _ours().handler;

      expect((await app(request('get', '/a'))).statusCode, 200);
    });
  });
}
