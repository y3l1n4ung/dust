import 'dart:convert';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';
import 'functions.dart';

Handler _app(Map<String, String> seed) {
  final notes = Router()
    ..merge(noteRoutes())
    ..withState(NoteStore(seed));

  return (Router()..nest('/notes', notes)).handler;
}

Request _authed(String method, String path, {Object? body}) {
  return request(
    method,
    path,
    headers: const {'authorization': 'Bearer todos:read'},
    body: body,
  );
}

void main() {
  tearDown(() => ServerErrors.reporter = null);

  group('function handlers', () {
    test('serve at the prefix the composition site chose', () async {
      final response = await _app({'1': 'first'})(_authed('GET', '/notes'));

      expect(response.statusCode, 200);
      expect(jsonDecode(await response.readAsString()), ['first']);
    });

    test('take path parameters like class handlers do', () async {
      final response = await _app({'1': 'first'})(_authed('GET', '/notes/1'));

      expect(jsonDecode(await response.readAsString()), {'note': 'first'});
    });

    test('map the error arm of a Result return type', () async {
      final response = await _app({})(_authed('GET', '/notes/9'));

      expect(response.statusCode, 404);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'no note 9'},
      );
    });

    test('short-circuit on an extractor rejection', () async {
      final response = await _app({})(request('GET', '/notes'));

      expect(response.statusCode, 401);
    });

    test('apply the declared success status', () async {
      final response =
          await _app({})(_authed('POST', '/notes', body: 'written'));

      expect(response.statusCode, 201);
      expect(
        jsonDecode(await response.readAsString()),
        {'note': 'written'},
      );
    });

    test('report missing state as 500, not as a client error', () async {
      final response = await (Router()..nest('/notes', noteRoutes()))
          .handler(_authed('GET', '/notes'));

      expect(response.statusCode, 500);
    });
  });

  group('both styles compose together', () {
    test('mount side by side under one root', () async {
      final notes = Router()
        ..merge(noteRoutes())
        ..withState(NoteStore({'1': 'first'}));
      final app = Router()..nest('/notes', notes);
      final handler = app.handler;

      expect(
        (await handler(_authed('GET', '/notes'))).statusCode,
        200,
      );
    });

    test('appear in describe() the same way', () {
      final app = Router()..nest('/notes', noteRoutes());

      expect(
        app.describe().map((route) => '${route.method} ${route.path}'),
        containsAll(<String>[
          'GET /notes',
          'GET /notes/{id}',
          'POST /notes',
        ]),
      );
    });
  });
}
