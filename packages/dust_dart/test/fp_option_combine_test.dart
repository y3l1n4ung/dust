import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

/// Throws if called, so a test can prove a lazy branch never ran.
Never _mustNotRun([Object? _, Object? __]) =>
    throw StateError('lazy callback ran');

void main() {
  group('zip and zipWith', () {
    test('pair two present values', () {
      expect(
        const Some<int>(1).zip(const Some<String>('a')),
        const Some<(int, String)>((1, 'a')),
      );
      expect(
        const Some<int>(1).zipWith(const Some<int>(2), (a, b) => a + b),
        const Some<int>(3),
      );
    });

    test('are absent unless both sides are present', () {
      expect(
        const Some<int>(1).zip(const None<String>()),
        const None<(int, String)>(),
      );
      expect(
        const None<int>().zipWith(const Some<int>(2), _mustNotRun),
        const None<Object?>(),
      );
    });
  });

  group('unzip', () {
    test('splits a present pair', () {
      final (left, right) = const Some<(int, String)>((1, 'a')).unzip();

      expect(left, const Some<int>(1));
      expect(right, const Some<String>('a'));
    });

    test('splits absence into two absences', () {
      final (left, right) = const None<(int, String)>().unzip();

      expect(left, const None<int>());
      expect(right, const None<String>());
    });
  });

  group('okOr and okOrElse', () {
    test('a present value becomes Ok', () {
      expect(const Some<int>(1).okOr('missing'), const Ok<int, String>(1));
      expect(
          const Some<int>(1).okOrElse(_mustNotRun), const Ok<int, String>(1));
    });

    test('absence becomes Err', () {
      expect(
          const None<int>().okOr('missing'), const Err<int, String>('missing'));
      expect(
        const None<int>().okOrElse(() => 'missing'),
        const Err<int, String>('missing'),
      );
    });

    test('a present null stays present rather than becoming an error', () {
      expect(
        const Some<int?>(null).okOr('missing'),
        const Ok<int?, String>(null),
      );
    });
  });

  group('flatten', () {
    test('removes one level of nesting', () {
      expect(
        const Some<Option<int>>(Some<int>(1)).flatten(),
        const Some<int>(1),
      );
      expect(
        const Some<Option<int>>(None<int>()).flatten(),
        const None<int>(),
      );
      expect(const None<Option<int>>().flatten(), const None<int>());
    });
  });

  group('transpose', () {
    test('absence is a successful absence', () {
      expect(
        const None<Result<int, String>>().transpose(),
        const Ok<Option<int>, String>(None<int>()),
      );
    });

    test('a present success becomes a successful presence', () {
      expect(
        const Some<Result<int, String>>(Ok(1)).transpose(),
        const Ok<Option<int>, String>(Some<int>(1)),
      );
    });

    test('a present failure is not hidden inside an option', () {
      expect(
        const Some<Result<int, String>>(Err('bad')).transpose(),
        const Err<Option<int>, String>('bad'),
      );
    });
  });
}
