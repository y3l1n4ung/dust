import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

/// Throws if called, so a test can prove a lazy branch never ran.
Never _mustNotRun([Object? _]) => throw StateError('lazy callback ran');

void main() {
  group('Option.fromNullable', () {
    test('null becomes absence and anything else presence', () {
      expect(Option<String>.fromNullable(null), const None<String>());
      expect(Option<String>.fromNullable('John'), const Some<String>('John'));
    });

    test('a present null has to be built directly', () {
      // fromNullable is the one place null means absence; Some(null) is how
      // a caller keeps null as a present value.
      expect(Option<String?>.fromNullable(null), const None<String?>());
      expect(const Some<String?>(null).isSome, isTrue);
    });
  });

  group('toNullable and toIterable', () {
    test('a present value comes back out', () {
      expect(const Some<int>(1).toNullable(), 1);
      expect(const Some<int>(1).toIterable(), [1]);
    });

    test('absence is null or empty', () {
      expect(const None<int>().toNullable(), isNull);
      expect(const None<int>().toIterable(), isEmpty);
    });

    test('toNullable collapses a present null into absence', () {
      expect(const Some<int?>(null).toNullable(), isNull);
      expect(const Some<int?>(null).toIterable(), [null]);
    });
  });

  group('contains, isSomeAnd, isNoneOr', () {
    test('contains compares a present value', () {
      expect(const Some<String>('admin').contains('admin'), isTrue);
      expect(const Some<String>('admin').contains('guest'), isFalse);
      expect(const None<String>().contains('admin'), isFalse);
      expect(const Some<String?>(null).contains(null), isTrue);
    });

    test('isSomeAnd needs presence and the predicate', () {
      expect(const Some<int>(21).isSomeAnd((age) => age >= 18), isTrue);
      expect(const Some<int>(12).isSomeAnd((age) => age >= 18), isFalse);
      expect(const None<int>().isSomeAnd(_mustNotRun), isFalse);
    });

    test('isNoneOr accepts absence without asking the predicate', () {
      expect(const None<int>().isNoneOr(_mustNotRun), isTrue);
      expect(const Some<int>(21).isNoneOr((age) => age >= 18), isTrue);
      expect(const Some<int>(12).isNoneOr((age) => age >= 18), isFalse);
    });
  });

  group('unwrap and expect', () {
    test('return a present value', () {
      expect(const Some<int>(7).unwrap(), 7);
      expect(const Some<int>(7).expect('an id was assigned'), 7);
      expect(const Some<int?>(null).unwrap(), isNull);
    });

    test('unwrap on absence throws a StateError', () {
      expect(
        () => const None<int>().unwrap(),
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            'called unwrap() on None',
          ),
        ),
      );
    });

    test('expect on absence throws with the caller message', () {
      expect(
        () => const None<int>().expect('the session has a user'),
        throwsA(
          isA<StateError>().having(
            (error) => error.message,
            'message',
            'the session has a user',
          ),
        ),
      );
    });
  });
}
