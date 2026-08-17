import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('HeaderExtractable', () {
    test('matches the name case-insensitively', () async {
      final outcome = await const HeaderExtractable<String>('X-Request-Id')
          .extract(request('GET', '/', headers: {'x-request-id': 'abc'}));

      expect(expectOk(outcome), 'abc');
    });

    test('coerces to the declared type', () async {
      final outcome = await const HeaderExtractable<int>('x-count')
          .extract(request('GET', '/', headers: {'x-count': '3'}));

      expect(expectOk(outcome), 3);
    });

    test('returns null for an absent optional header', () async {
      final outcome = await const HeaderExtractable<String?>('x-trace')
          .extract(request('GET', '/'));

      expect(expectOk(outcome), isNull);
    });

    test('rejects an absent required header with 400', () async {
      final outcome = await const HeaderExtractable<String>('x-trace')
          .extract(request('GET', '/'));

      expect(
        expectStatus(outcome, 400).message,
        'header "x-trace" is required',
      );
    });
  });

  group('HeaderMapExtractable', () {
    test('returns every header, lower-cased', () async {
      final outcome = await const HeaderMapExtractable()
          .extract(request('GET', '/', headers: {'X-One': '1', 'X-Two': '2'}));
      final headers = expectOk(outcome);

      expect(headers['x-one'], '1');
      expect(headers['x-two'], '2');
    });

    test('returns an unmodifiable map', () async {
      final outcome =
          await const HeaderMapExtractable().extract(request('GET', '/'));

      expect(() => expectOk(outcome)['x'] = 'y', throwsUnsupportedError);
    });
  });
}
