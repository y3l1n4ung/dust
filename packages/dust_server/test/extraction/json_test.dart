import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';
import 'json_fixtures.dart';

const _extractor = JsonExtractable<CreateTodo>(CreateTodo.fromJson);

void main() {
  group('JsonExtractable', () {
    test('decodes a well-formed body', () async {
      final outcome = await _extractor
          .extract(jsonRequest('POST', '/todos', '{"title":"write tests"}'));

      expect(expectOk(outcome).title, 'write tests');
    });

    test('accepts a +json media type', () async {
      final outcome = await _extractor.extract(
        request(
          'POST',
          '/todos',
          headers: {'content-type': 'application/vnd.api+json'},
          body: '{"title":"ok"}',
        ),
      );

      expect(expectOk(outcome).title, 'ok');
    });

    test('rejects a missing content-type with 415', () async {
      final outcome = await _extractor
          .extract(request('POST', '/todos', body: '{"title":"x"}'));

      expect(expectStatus(outcome, 415).message, 'expected application/json');
    });

    test('rejects a non-JSON content-type with 415', () async {
      final outcome = await _extractor.extract(
        request(
          'POST',
          '/todos',
          headers: {'content-type': 'text/plain'},
          body: '{"title":"x"}',
        ),
      );

      expectStatus(outcome, 415);
    });

    test('rejects an empty body with 400', () async {
      final outcome =
          await _extractor.extract(jsonRequest('POST', '/todos', ''));

      expect(expectStatus(outcome, 400).message, 'request body is empty');
    });

    test('rejects malformed JSON with 400', () async {
      final outcome =
          await _extractor.extract(jsonRequest('POST', '/todos', '{"title"'));

      expect(expectStatus(outcome, 400).message, startsWith('malformed JSON'));
    });

    test('rejects a non-object body with 422', () async {
      final outcome =
          await _extractor.extract(jsonRequest('POST', '/todos', '[1,2]'));

      expect(expectStatus(outcome, 422).message, 'expected a JSON object');
    });

    test('rejects a wrong shape with 422', () async {
      final outcome = await _extractor
          .extract(jsonRequest('POST', '/todos', '{"title":7}'));

      expect(
        expectStatus(outcome, 422).message,
        startsWith('JSON body does not match'),
      );
    });
  });

  group('body limits', () {
    test('rejects an oversized declared body with 413', () async {
      final outcome = await const JsonExtractable<CreateTodo>(
        CreateTodo.fromJson,
        limit: 8,
      ).extract(jsonRequest('POST', '/todos', '{"title":"much too long"}'));

      expect(expectStatus(outcome, 413).message, 'body exceeds 8 bytes');
    });

    test('rejects an oversized streamed body with 413', () async {
      final body = Stream<List<int>>.fromIterable([
        [123, 34],
        List<int>.filled(64, 97),
      ]);
      final outcome = await const JsonExtractable<CreateTodo>(
        CreateTodo.fromJson,
        limit: 8,
      ).extract(
        request(
          'POST',
          '/todos',
          headers: {'content-type': 'application/json'},
          body: body,
        ),
      );

      expectStatus(outcome, 413);
    });

    test('honours a composition-time limit over the built-in one', () async {
      final outcome = await _extractor.extract(
        request(
          'POST',
          '/todos',
          headers: {'content-type': 'application/json'},
          context: {bodyLimitContextKey: 4},
          body: '{"title":"x"}',
        ),
      );

      expect(expectStatus(outcome, 413).message, 'body exceeds 4 bytes');
    });
  });

  group('readBody', () {
    test('returns the whole body', () async {
      final outcome = await readBody(request('POST', '/', body: 'abc'));

      expect(expectOk(outcome), [97, 98, 99]);
    });

    test('rejects past the limit with 413', () async {
      final outcome =
          await readBody(request('POST', '/', body: 'abcd'), limit: 3);

      expectStatus(outcome, 413);
    });
  });
}
