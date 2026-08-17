import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';
import 'json_fixtures.dart';

const _extractor = JsonListExtractable<CreateTodo>(CreateTodo.fromJson);

void main() {
  group('JsonListExtractable', () {
    test('decodes an array body', () async {
      final outcome = await _extractor.extract(
        jsonRequest('POST', '/todos', '[{"title":"a"},{"title":"b"}]'),
      );

      expect(expectOk(outcome).map((todo) => todo.title), ['a', 'b']);
    });

    test('rejects a non-array body with 422', () async {
      final outcome = await _extractor
          .extract(jsonRequest('POST', '/todos', '{"title":"a"}'));

      expect(expectStatus(outcome, 422).message, 'expected a JSON array');
    });

    test('rejects a bad element with 422', () async {
      final outcome = await _extractor
          .extract(jsonRequest('POST', '/todos', '[{"title":7}]'));

      expectStatus(outcome, 422);
    });

    test('rejects the wrong media type with 415', () async {
      final outcome =
          await _extractor.extract(request('POST', '/todos', body: '[]'));

      expectStatus(outcome, 415);
    });

    test('rejects an empty body with 400', () async {
      final outcome =
          await _extractor.extract(jsonRequest('POST', '/todos', ''));

      expectStatus(outcome, 400);
    });

    test('rejects malformed JSON with 400', () async {
      final outcome =
          await _extractor.extract(jsonRequest('POST', '/todos', '[{'));

      expectStatus(outcome, 400);
    });
  });
}
