import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('PathExtractable', () {
    test('reads a captured segment', () async {
      final outcome = await const PathExtractable<String>('id')
          .extract(request('GET', '/todos/7', pathParameters: {'id': '7'}));

      expect(expectOk(outcome), '7');
    });

    test('coerces to the declared type', () async {
      final outcome = await const PathExtractable<int>('id')
          .extract(request('GET', '/todos/7', pathParameters: {'id': '7'}));

      expect(expectOk(outcome), 7);
    });

    test('rejects a missing segment with 400', () async {
      final outcome = await const PathExtractable<String>('id')
          .extract(request('GET', '/'));

      expect(
        expectStatus(outcome, 400).message,
        'path parameter "id" is missing',
      );
    });

    test('rejects an uncoercible segment with 400', () async {
      final outcome = await const PathExtractable<int>('id')
          .extract(request('GET', '/todos/x', pathParameters: {'id': 'x'}));

      expectStatus(outcome, 400);
    });
  });

  group('percent decoding', () {
    test('decodes a captured segment', () async {
      final outcome = await const PathExtractable<String>('name').extract(
        request('GET', '/files/a%20b', pathParameters: {'name': 'a%20b'}),
      );

      expect(expectOk(outcome), 'a b');
    });

    test('decodes before coercing', () async {
      final outcome = await const PathExtractable<int>('id')
          .extract(request('GET', '/todos/7', pathParameters: {'id': '%37'}));

      expect(expectOk(outcome), 7);
    });

    test('rejects malformed percent-encoding with 400', () async {
      final outcome = await const PathExtractable<String>('name')
          .extract(request('GET', '/files/x', pathParameters: {'name': '%zz'}));

      expect(
        expectStatus(outcome, 400).message,
        'path parameter "name" is not valid percent-encoding',
      );
    });

    test('decodes an encoded slash without splitting the segment', () async {
      final outcome = await const PathExtractable<String>('name').extract(
        request('GET', '/files/a%2Fb', pathParameters: {'name': 'a%2Fb'}),
      );

      expect(expectOk(outcome), 'a/b');
    });
  });
}
