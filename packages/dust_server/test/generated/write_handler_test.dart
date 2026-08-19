import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import 'controller.dart';

void main() {
  tearDown(() => ServerErrors.reporter = null);

  group('generated POST handler', () {
    test('applies the declared success status', () async {
      final response = await todoApp()(
        authed(
          'POST',
          '/api/v1/todos',
          scopes: 'todos:write',
          json: true,
          body: '{"title":"write tests"}',
        ),
      );

      expect(response.statusCode, 201);
      expect(
        jsonDecode(await response.readAsString()),
        {'id': '8', 'title': 'write tests'},
      );
    });

    test('rejects an insufficient scope with 403 before reading the body',
        () async {
      final response = await todoApp()(
        authed(
          'POST',
          '/api/v1/todos',
          json: true,
          body: '{"title":"write tests"}',
        ),
      );

      expect(response.statusCode, 403);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'requires scope todos:write'},
      );
    });

    test('rejects a missing content-type with 415', () async {
      final response = await todoApp()(
        authed(
          'POST',
          '/api/v1/todos',
          scopes: 'todos:write',
          body: '{"title":"x"}',
        ),
      );

      expect(response.statusCode, 415);
    });

    test('rejects malformed JSON with 400', () async {
      final response = await todoApp()(
        authed(
          'POST',
          '/api/v1/todos',
          scopes: 'todos:write',
          json: true,
          body: '{"title"',
        ),
      );

      expect(response.statusCode, 400);
    });

    test('rejects a wrong shape with 422', () async {
      final response = await todoApp()(
        authed(
          'POST',
          '/api/v1/todos',
          scopes: 'todos:write',
          json: true,
          body: '{"title":7}',
        ),
      );

      expect(response.statusCode, 422);
    });

    test('rejects a validation failure with 422 and field errors', () async {
      final response = await todoApp()(
        authed(
          'POST',
          '/api/v1/todos',
          scopes: 'todos:write',
          json: true,
          body: '{"title":""}',
        ),
      );

      expect(response.statusCode, 422);
      expect(jsonDecode(await response.readAsString()), {
        'error': 'Unprocessable entity',
        'fields': {
          'title': ['must not be empty'],
        },
      });
    });
  });

  group('errors from the handler body', () {
    test('honours a thrown rejection', () async {
      final response = await todoApp()(
        authed(
          'POST',
          '/api/v1/todos',
          scopes: 'todos:write',
          json: true,
          body: '{"title":"existing"}',
        ),
      );

      expect(response.statusCode, 409);
    });

    test('turns an uncaught error into an opaque 500 and reports it', () async {
      final errors = <Object>[];
      final handler = todoApp(onError: (error, stack) => errors.add(error));

      final response = await handler(
        authed(
          'POST',
          '/api/v1/todos',
          scopes: 'todos:write',
          json: true,
          body: '{"title":"boom"}',
        ),
      );

      expect(response.statusCode, 500);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'Internal server error'},
      );
      expect(errors.single, isA<StateError>());
    });
  });
}
