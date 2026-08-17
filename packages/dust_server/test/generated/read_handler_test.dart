import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';
import 'controller.dart';

void main() {
  tearDown(() => ServerErrors.reporter = null);

  group('generated GET handler', () {
    test('serves the success case as JSON', () async {
      final response = await todoApp()(authed('GET', '/api/v1/todos/7'));

      expect(response.statusCode, 200);
      expect(
        jsonDecode(await response.readAsString()),
        {'id': '7', 'title': 'existing'},
      );
    });

    test('maps the error arm of a Result return type', () async {
      final response = await todoApp()(authed('GET', '/api/v1/todos/404'));

      expect(response.statusCode, 404);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'no todo 404'},
      );
    });

    test('short-circuits on the first extractor rejection', () async {
      final response = await todoApp()(request('GET', '/api/v1/todos/7'));

      expect(response.statusCode, 401);
      expect(response.headers['www-authenticate'], 'Bearer');
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'missing bearer token'},
      );
    });
  });

  group('generated DELETE handler', () {
    test('returns 204 for a void handler', () async {
      final response = await todoApp()(
        authed('DELETE', '/api/v1/todos/7', scopes: 'todos:write'),
      );

      expect(response.statusCode, 204);
      expect(await response.readAsString(), isEmpty);
    });
  });

  group('routing', () {
    test('answers an unknown path with 404', () async {
      final response = await todoApp()(authed('GET', '/api/v1/nope'));

      expect(response.statusCode, 404);
    });

    test('answers an unknown method on a known path with 405', () async {
      final response = await todoApp()(authed('PUT', '/api/v1/todos/7'));

      expect(response.statusCode, 405);
      expect(response.headers['allow'], 'DELETE, GET, HEAD');
    });
  });
}
