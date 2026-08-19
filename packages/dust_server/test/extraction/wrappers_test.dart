import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('OptionalExtractable', () {
    test('wraps a successful extraction in Some', () async {
      final outcome = await const OptionalExtractable(
        HeaderExtractable<String>('x-user'),
      ).extract(request('GET', '/', headers: {'x-user': 'ada'}));

      expect(expectOk(outcome), const Some('ada'));
    });

    test('collapses a rejection to None', () async {
      final outcome = await const OptionalExtractable(
        HeaderExtractable<String>('x-user'),
      ).extract(request('GET', '/'));

      expect(expectOk(outcome), const None<String>());
    });
  });

  group('FallibleExtractable', () {
    test('hands a successful extraction to the handler', () async {
      final outcome = await const FallibleExtractable(
        HeaderExtractable<String>('x-user'),
      ).extract(request('GET', '/', headers: {'x-user': 'ada'}));

      expect(expectOk(expectOk(outcome)), 'ada');
    });

    test('hands the rejection to the handler instead of short-circuiting',
        () async {
      final outcome = await const FallibleExtractable(
        HeaderExtractable<String>('x-user'),
      ).extract(request('GET', '/'));

      final inner = expectOk(outcome);
      expect(expectStatus(inner, 400).message, contains('x-user'));
    });

    test('nests with Optional to model a fully permissive parameter', () async {
      final outcome = await const FallibleExtractable(
        OptionalExtractable(HeaderExtractable<String>('x-user')),
      ).extract(request('GET', '/'));

      expect(expectOk(expectOk(outcome)), const None<String>());
    });
  });

  group('Option over a server-side failure', () {
    test('collapses a client-error rejection to None', () async {
      final outcome = await const OptionalExtractable(
        HeaderExtractable<String>('x-user'),
      ).extract(request('GET', '/'));

      expect(expectOk(outcome), const None<String>());
    });

    test('propagates a 5xx rejection instead of hiding it', () async {
      final outcome = await const OptionalExtractable(
        ContextExtractable<String>('tenant'),
      ).extract(request('GET', '/'));

      expect(expectStatus(outcome, 500).message, contains('tenant'));
    });

    test('still collapses a 4xx from a body extractor', () async {
      final outcome = await const OptionalExtractable(
        RawBodyExtractable(limit: 2),
      ).extract(request('POST', '/', body: 'far too long'));

      expect(expectOk(outcome), isA<None<dynamic>>());
    });
  });
}
