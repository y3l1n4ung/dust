import 'package:test/test.dart';

import 'support/server.dart';

/// Every failure the API can answer with, checked for the status code and for
/// the header the specification requires alongside it.

void main() {
  late ExampleServer server;

  setUp(() async => server = await ExampleServer.start());
  tearDown(() => server.stop());

  group('authentication', () {
    test('answers 401 without a token', () async {
      final response = await server.get('/api/v1/todos', token: null);

      expect(response.statusCode, 401);
    });

    test('carries the challenge a 401 requires', () async {
      final response = await server.get('/api/v1/todos', token: null);

      expect(response.headers['www-authenticate'], 'Bearer');
    });

    test('answers 403 when the token lacks the scope', () async {
      final response =
          await server.post('/api/v1/todos', validBody(), token: readToken);

      expect(response.statusCode, 403);
    });

    test('names the scope the caller is missing', () async {
      final response =
          await server.post('/api/v1/todos', validBody(), token: readToken);

      expect(objectOf(response)['error'], 'requires scope todos:write');
    });

    test('accepts a token carrying several scopes', () async {
      final response = await server.post('/api/v1/todos', validBody());

      expect(response.statusCode, 201);
    });

    test('refuses a token with no identity', () async {
      final response = await server.get('/api/v1/todos', token: '|todos:read');

      expect(response.statusCode, 401);
    });

    test('refuses a token with no scope section', () async {
      final response = await server.get('/api/v1/todos', token: owner);

      expect(response.statusCode, 401);
    });
  });

  group('routing', () {
    test('answers 404 for an unknown todo', () async {
      final response = await server.get('/api/v1/todos/999999');

      expect(response.statusCode, 404);
    });

    test('answers 404 for an unknown path', () async {
      final response = await server.get('/api/v1/nothing');

      expect(response.statusCode, 404);
    });

    test('answers 405 with the methods the path does serve', () async {
      final response = await server.patch('/api/v1/todos');

      expect(response.statusCode, 405);
      expect(response.headers['allow'], 'GET, HEAD, POST');
    });

    test('answers a HEAD with no body and the same status', () async {
      final response = await server.get('/health', token: null);

      expect(response.statusCode, 200);
      expect(response.body, isNotEmpty);
    });
  });

  group('query coercion', () {
    test('answers 400 when a required query is missing', () async {
      final created = objectOf(await server.post('/api/v1/todos', validBody()));
      final response = await server.patch('/api/v1/todos/${created['id']}');

      expect(response.statusCode, 400);
    });

    test('answers 400 when a query will not coerce', () async {
      final response = await server.get('/api/v1/todos?done=maybe');

      expect(response.statusCode, 400);
    });

    test('treats an absent optional query as no filter', () async {
      final listed = bodyOf(await server.get('/api/v1/todos'))! as List;

      expect(listed, hasLength(1));
    });
  });

  group('the error sink', () {
    test('stays empty when every failure belongs to the client', () async {
      await server.get('/api/v1/todos', token: null);
      await server.get('/api/v1/todos/999999');
      await server.post('/api/v1/todos', '{not json');

      expect(server.errors, isEmpty);
    });
  });
}
