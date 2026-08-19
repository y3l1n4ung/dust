import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

Handler _throws(Object error) =>
    (request) async => guard(() async => throw error);

void main() {
  tearDown(() => ServerErrors.reporter = null);

  group('error reporting', () {
    test('turns an uncaught error into an opaque 500', () async {
      final errors = <Object>[];
      final app = Router(onError: (error, stack) => errors.add(error))
        ..route('/boom', get(_throws(StateError('boom'))));

      final response = await app.handler(request('GET', '/boom'));

      expect(response.statusCode, 500);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'Internal server error'},
      );
      expect(errors.single, isA<StateError>());
    });

    test('honours a rejection thrown from a handler', () async {
      final errors = <Object>[];
      final app = Router(onError: (error, stack) => errors.add(error))
        ..route('/conflict', get(_throws(const Rejection.conflict('dup'))));

      expect((await app.handler(request('GET', '/conflict'))).statusCode, 409);
      expect(errors, isEmpty, reason: 'a rejection is an answer, not a fault');
    });

    test('keeps two routers in one isolate independent', () async {
      final first = <Object>[];
      final second = <Object>[];
      final a = Router(onError: (error, stack) => first.add(error))
        ..route('/boom', get(_throws(StateError('from a'))));
      final b = Router(onError: (error, stack) => second.add(error))
        ..route('/boom', get(_throws(StateError('from b'))));

      await a.handler(request('GET', '/boom'));
      await b.handler(request('GET', '/boom'));

      expect(first.single.toString(), contains('from a'));
      expect(second.single.toString(), contains('from b'));
    });

    test('falls back to the process-wide sink when a router set none',
        () async {
      final errors = <Object>[];
      ServerErrors.reporter = (error, stack) => errors.add(error);
      final app = Router()..route('/boom', get(_throws(StateError('boom'))));

      await app.handler(request('GET', '/boom'));

      expect(errors.single, isA<StateError>());
    });

    test("prefers the router's sink over the process-wide one", () async {
      final global = <Object>[];
      final scoped = <Object>[];
      ServerErrors.reporter = (error, stack) => global.add(error);
      final app = Router(onError: (error, stack) => scoped.add(error))
        ..route('/boom', get(_throws(StateError('boom'))));

      await app.handler(request('GET', '/boom'));

      expect(scoped, hasLength(1));
      expect(global, isEmpty);
    });

    test('drops errors when nobody is listening', () async {
      final app = Router()..route('/boom', get(_throws(StateError('boom'))));

      expect((await app.handler(request('GET', '/boom'))).statusCode, 500);
    });
  });
}
