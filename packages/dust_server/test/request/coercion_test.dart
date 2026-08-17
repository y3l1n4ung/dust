import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

void main() {
  group('coerce', () {
    test('passes strings through', () {
      expect(expectOk(coerce<String>('abc', source: 'q')), 'abc');
      expect(expectOk(coerce<String?>('abc', source: 'q')), 'abc');
    });

    test('parses integers', () {
      expect(expectOk(coerce<int>('42', source: 'q')), 42);
      expect(expectOk(coerce<int>('-7', source: 'q')), -7);
      expect(expectOk(coerce<int?>('42', source: 'q')), 42);
    });

    test('rejects a malformed integer with 400', () {
      final rejection =
          expectStatus(coerce<int>('4.2', source: 'query "n"'), 400);

      expect(rejection.message, 'query "n" is not a valid integer');
    });

    test('parses doubles and nums', () {
      expect(expectOk(coerce<double>('1.5', source: 'q')), 1.5);
      expect(expectOk(coerce<num>('2', source: 'q')), 2);
      expect(expectOk(coerce<num>('2.5', source: 'q')), 2.5);
      expectStatus(coerce<double>('x', source: 'q'), 400);
      expectStatus(coerce<num>('x', source: 'q'), 400);
    });

    test('parses the accepted boolean spellings', () {
      expect(expectOk(coerce<bool>('true', source: 'q')), isTrue);
      expect(expectOk(coerce<bool>('TRUE', source: 'q')), isTrue);
      expect(expectOk(coerce<bool>('1', source: 'q')), isTrue);
      expect(expectOk(coerce<bool>('', source: 'q')), isTrue);
      expect(expectOk(coerce<bool>('false', source: 'q')), isFalse);
      expect(expectOk(coerce<bool>('0', source: 'q')), isFalse);
    });

    test('rejects other boolean spellings with 400', () {
      expectStatus(coerce<bool>('yes', source: 'q'), 400);
      expectStatus(coerce<bool>('2', source: 'q'), 400);
    });

    test('parses BigInt, DateTime, and Uri', () {
      expect(
        expectOk(coerce<BigInt>('123456789012345678901', source: 'q')),
        BigInt.parse('123456789012345678901'),
      );
      expect(
        expectOk(coerce<DateTime>('2026-08-15T10:00:00Z', source: 'q')),
        DateTime.utc(2026, 8, 15, 10),
      );
      expect(
        expectOk(coerce<Uri>('https://example.com/a', source: 'q')),
        Uri.parse('https://example.com/a'),
      );
    });

    test('rejects malformed BigInt and DateTime with 400', () {
      expectStatus(coerce<BigInt>('1.5', source: 'q'), 400);
      expectStatus(coerce<DateTime>('yesterday', source: 'q'), 400);
    });

    test('names the source in every rejection', () {
      final rejection =
          expectStatus(coerce<int>('x', source: 'header "x-count"'), 400);

      expect(rejection.message, startsWith('header "x-count"'));
    });

    test('throws for a type it cannot convert', () {
      expect(
        () => coerce<Duration>('1s', source: 'q'),
        throwsA(isA<ArgumentError>()),
      );
    });
  });

  group('typeOf', () {
    test('reifies nullable types so they compare equal', () {
      expect(typeOf<String?>(), typeOf<String?>());
      expect(typeOf<String?>() == String, isFalse);
    });
  });
}
