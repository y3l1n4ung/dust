import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../../support.dart';

/// A verb builder takes what the endpoint produced, not a response. These
/// check the conversion, the failure paths, and that a plain `shelf` handler
/// still passes through it untouched.

final class _Repo {
  const _Repo(this.name);

  final String name;
}

void main() {
  Future<Response> send(
    Router app,
    String method,
    String path, {
    Map<String, String> headers = const {},
  }) async =>
      app.handler(request(method, path, headers: headers));

  group('the returned value', () {
    test('reaches the wire as JSON', () async {
      final app = Router()..route('/', get((request) async => {'ok': true}));
      final response = await send(app, 'GET', '/');

      expect(jsonDecode(await response.readAsString()), {'ok': true});
    });

    test('answers 204 when it is null', () async {
      final app = Router()..route('/', delete((request) async => null));

      expect((await send(app, 'DELETE', '/')).statusCode, 204);
    });

    test('keeps a returned rejection’s own status', () async {
      final app = Router()
        ..route('/', get((request) async => const Rejection.notFound('gone')));

      expect((await send(app, 'GET', '/')).statusCode, 404);
    });

    test('is built from extractors read inside the endpoint', () async {
      final app = Router()
        ..route('/{id}', get((request) async {
          final id = await path<String>('id').require(request);
          final repository = await state<_Repo>().require(request);
          return {'id': id, 'repo': repository.name};
        }))
        ..withState(const _Repo('store'));

      final response = await send(app, 'GET', '/7');

      expect(jsonDecode(await response.readAsString()), {
        'id': '7',
        'repo': 'store',
      });
    });
  });

  group('failure', () {
    test('answers with the rejection an extractor produced', () async {
      final app = Router()
        ..route('/', get((request) async {
          await query<int>('required').require(request);
          return 'unreachable';
        }));

      expect((await send(app, 'GET', '/')).statusCode, 400);
    });

    test('turns a thrown rejection into its own response', () async {
      final app = Router()
        ..route('/', get((request) async {
          throw const Rejection.conflict('already there');
        }));

      expect((await send(app, 'GET', '/')).statusCode, 409);
    });

    test('turns anything else into an opaque 500', () async {
      final errors = <Object>[];
      final app = Router(onError: (error, stack) => errors.add(error))
        ..route('/', get((request) async => throw StateError('boom')));

      final response = await send(app, 'GET', '/');

      expect(response.statusCode, 500);
      expect(await response.readAsString(), isNot(contains('boom')));
      expect(errors.single, isA<StateError>());
    });
  });

  group('status', () {
    test('applies to a value the endpoint returned', () async {
      final app = Router()
        ..route('/', post((request) async => {'ok': true}, status: 201));

      expect((await send(app, 'POST', '/')).statusCode, 201);
    });

    test('does not override a returned rejection', () async {
      final app = Router()
        ..route(
          '/',
          post((request) async => const Rejection.notFound('gone'),
              status: 201),
        );

      expect((await send(app, 'POST', '/')).statusCode, 404);
    });

    test('does not override an empty answer', () async {
      final app = Router()
        ..route('/', post((request) async => null, status: 201));

      expect((await send(app, 'POST', '/')).statusCode, 204);
    });
  });

  group('a plain shelf handler', () {
    test('passes through the verb builder untouched', () async {
      final original = Response(207, body: 'multi');
      final app = Router()..route('/', get((request) async => original));

      expect(await (await send(app, 'GET', '/')).readAsString(), 'multi');
    });

    test('keeps its own status rather than the success status', () async {
      final app = Router()
        ..route(
          '/',
          post((request) async => Response(202, body: 'queued'), status: 201),
        );

      expect((await send(app, 'POST', '/')).statusCode, 202);
    });
  });
}
