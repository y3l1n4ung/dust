import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

void main() {
  group('Rejection', () {
    test('maps each constructor to its status code', () {
      expect(const Rejection.badRequest('x').status, 400);
      expect(const Rejection.unauthorized('x').status, 401);
      expect(const Rejection.forbidden('x').status, 403);
      expect(const Rejection.notFound('x').status, 404);
      expect(const Rejection.methodNotAllowed('x').status, 405);
      expect(const Rejection.conflict('x').status, 409);
      expect(const Rejection.payloadTooLarge('x').status, 413);
      expect(const Rejection.unsupportedMediaType('x').status, 415);
      expect(const Rejection.unprocessable({}).status, 422);
      expect(const Rejection.internal().status, 500);
      expect(const Rejection.status(418, 'x').status, 418);
    });

    test('encodes the message as JSON', () async {
      final response = const Rejection.forbidden('nope').intoResponse();

      expect(response.statusCode, 403);
      expect(response.headers['content-type'], 'application/json');
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'nope'},
      );
    });

    test('encodes field errors only when present', () async {
      final withFields = const Rejection.unprocessable({
        'title': ['must not be empty'],
      }).intoResponse();
      final withoutFields = const Rejection.badRequest('bad').intoResponse();

      expect(jsonDecode(await withFields.readAsString()), {
        'error': 'Unprocessable entity',
        'fields': {
          'title': ['must not be empty'],
        },
      });
      expect(jsonDecode(await withoutFields.readAsString()), {'error': 'bad'});
    });

    test('carries a custom message alongside field errors', () {
      const rejection = Rejection.unprocessable(
        {
          'title': ['too long']
        },
        message: 'validation failed',
      );

      expect(rejection.message, 'validation failed');
      expect(rejection.fields['title'], ['too long']);
    });

    test('defaults the internal message', () {
      expect(const Rejection.internal().message, 'Internal server error');
    });

    test('is const-constructible for use in generated code', () {
      const first = Rejection.unauthorized('missing bearer token');
      const second = Rejection.unauthorized('missing bearer token');

      expect(identical(first, second), isTrue);
    });

    test('describes itself', () {
      expect(
        const Rejection.notFound('gone').toString(),
        'Rejection(404, gone)',
      );
    });
  });

  group('401 challenge', () {
    test('carries WWW-Authenticate by default', () {
      final response =
          const Rejection.unauthorized('missing bearer token').intoResponse();

      expect(response.statusCode, 401);
      expect(response.headers['www-authenticate'], 'Bearer');
    });

    test('accepts another scheme', () {
      final response = const Rejection.unauthorized(
        'missing credentials',
        challenge: 'Basic realm="todos"',
      ).intoResponse();

      expect(response.headers['www-authenticate'], 'Basic realm="todos"');
    });

    test('leaves the header off other statuses', () {
      expect(
        const Rejection.forbidden('nope')
            .intoResponse()
            .headers
            .containsKey('www-authenticate'),
        isFalse,
      );
    });
  });
}
