import 'dart:convert';
import 'dart:typed_data';

import 'package:dust_server/server.dart';
import 'package:test/test.dart';

void main() {
  tearDown(() => ServerErrors.reporter = null);

  group('response helpers', () {
    test('encodes JSON with the right content type', () async {
      final response = jsonResponse({'id': 1}, status: 201);

      expect(response.statusCode, 201);
      expect(response.headers['content-type'], 'application/json');
      expect(jsonDecode(await response.readAsString()), {'id': 1});
    });

    test('encodes text', () async {
      final response = textResponse('hello');

      expect(response.headers['content-type'], 'text/plain; charset=utf-8');
      expect(await response.readAsString(), 'hello');
    });

    test('encodes bytes', () async {
      final response = bytesResponse(Uint8List.fromList([1, 2, 3]));

      expect(response.headers['content-type'], 'application/octet-stream');
      expect(await response.read().first, [1, 2, 3]);
    });

    test('streams bytes', () async {
      final response = streamResponse(
        Stream<List<int>>.fromIterable([
          [1],
          [2],
        ]),
      );

      expect(
        await response.read().fold<List<int>>([], (a, b) => a..addAll(b)),
        [1, 2],
      );
    });

    test('encodes an empty 204', () {
      expect(noContent().statusCode, 204);
    });
  });

  group('guard', () {
    test('passes a normal response through', () async {
      final response = await guard(() async => textResponse('ok'));

      expect(response.statusCode, 200);
    });

    test('turns a thrown rejection into its own response', () async {
      final response = await guard(
        () async => throw const Rejection.conflict('duplicate title'),
      );

      expect(response.statusCode, 409);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'duplicate title'},
      );
    });

    test('turns any other error into an opaque 500', () async {
      final response = await guard(() async => throw StateError('boom'));

      expect(response.statusCode, 500);
      expect(
        jsonDecode(await response.readAsString()),
        {'error': 'Internal server error'},
      );
    });

    test('reports the error to the composition-site sink', () async {
      Object? reported;
      ServerErrors.reporter = (error, stack) => reported = error;

      await guard(() async => throw StateError('boom'));

      expect(reported, isA<StateError>());
    });

    test('does not report a thrown rejection', () async {
      var reported = 0;
      ServerErrors.reporter = (error, stack) => reported++;

      await guard(() async => throw const Rejection.notFound('gone'));

      expect(reported, 0);
    });
  });
}
