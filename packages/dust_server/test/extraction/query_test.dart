import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('QueryExtractable', () {
    test('reads and coerces a value', () async {
      final outcome = await const QueryExtractable<bool>('done')
          .extract(request('GET', '/todos?done=true'));

      expect(expectOk(outcome), isTrue);
    });

    test('treats a bare flag as true', () async {
      final outcome = await const QueryExtractable<bool>('done')
          .extract(request('GET', '/todos?done'));

      expect(expectOk(outcome), isTrue);
    });

    test('returns null for an absent optional value', () async {
      final outcome = await const QueryExtractable<bool?>('done')
          .extract(request('GET', '/todos'));

      expect(expectOk(outcome), isNull);
    });

    test('rejects an absent required value with 400', () async {
      final outcome = await const QueryExtractable<int>('page')
          .extract(request('GET', '/todos'));

      expect(expectStatus(outcome, 400).message, 'query "page" is required');
    });

    test('rejects an uncoercible value with 400', () async {
      final outcome = await const QueryExtractable<int>('page')
          .extract(request('GET', '/todos?page=many'));

      expectStatus(outcome, 400);
    });
  });

  group('QueryListExtractable', () {
    test('reads every value for a repeated key', () async {
      final outcome = await const QueryListExtractable<String>('tag')
          .extract(request('GET', '/todos?tag=a&tag=b'));

      expect(expectOk(outcome), ['a', 'b']);
    });

    test('coerces each value', () async {
      final outcome = await const QueryListExtractable<int>('id')
          .extract(request('GET', '/todos?id=1&id=2'));

      expect(expectOk(outcome), [1, 2]);
    });

    test('returns an empty list for an absent key', () async {
      final outcome = await const QueryListExtractable<String>('tag')
          .extract(request('GET', '/todos'));

      expect(expectOk(outcome), isEmpty);
    });

    test('rejects the whole list when one value fails to coerce', () async {
      final outcome = await const QueryListExtractable<int>('id')
          .extract(request('GET', '/todos?id=1&id=x'));

      expectStatus(outcome, 400);
    });
  });

  group('QueriesExtractable', () {
    test('decodes the whole query map', () async {
      final extractor = QueriesExtractable<Map<String, String>>((raw) => raw);
      final outcome = await extractor.extract(request('GET', '/todos?a=1&b=2'));

      expect(expectOk(outcome), {'a': '1', 'b': '2'});
    });

    test('reports a failed decode as 422', () async {
      final extractor = QueriesExtractable<int>(
        (raw) => throw const FormatException('bad shape'),
      );
      final outcome = await extractor.extract(request('GET', '/todos?a=1'));

      expectStatus(outcome, 422);
    });
  });

  group('RawQueryExtractable', () {
    test('returns the undecoded query', () async {
      final outcome = await const RawQueryExtractable()
          .extract(request('GET', '/todos?a=1&a=2'));

      expect(expectOk(outcome), 'a=1&a=2');
    });

    test('returns null when there is no query', () async {
      final outcome =
          await const RawQueryExtractable().extract(request('GET', '/todos'));

      expect(expectOk(outcome), isNull);
    });
  });
}
