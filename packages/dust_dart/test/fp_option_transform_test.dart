import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

/// Throws if called, so a test can prove a lazy branch never ran.
Never _mustNotRun([Object? _]) => throw StateError('lazy callback ran');

void main() {
  group('mapOr and mapOrElse', () {
    test('map a present value', () {
      expect(const Some<String>('John').mapOr(0, (name) => name.length), 4);
      expect(
        const Some<String>('John')
            .mapOrElse(_mustNotRun, (name) => name.length),
        4,
      );
    });

    test('fall back when absent', () {
      expect(const None<String>().mapOr(0, _mustNotRun), 0);
      expect(const None<String>().mapOrElse(() => 0, _mustNotRun), 0);
    });
  });

  group('inspect', () {
    test('sees a present value and returns the option unchanged', () {
      final seen = <int>[];
      const option = Some<int>(7);

      expect(option.inspect(seen.add), same(option));
      expect(seen, [7]);
    });

    test('does nothing for absence', () {
      const option = None<int>();

      expect(option.inspect(_mustNotRun), same(option));
    });
  });

  group('filter', () {
    test('keeps a value that passes', () {
      const option = Some<int>(4);

      expect(option.filter((value) => value.isEven), same(option));
    });

    test('drops a value that fails, and absence stays absent', () {
      expect(const Some<int>(3).filter((value) => value.isEven),
          const None<int>());
      expect(const None<int>().filter(_mustNotRun), const None<int>());
    });
  });

  group('and, or, orElse', () {
    test('and returns the other option only after presence', () {
      expect(
        const Some<int>(1).and(const Some<String>('a')),
        const Some<String>('a'),
      );
      expect(
        const None<int>().and(const Some<String>('a')),
        const None<String>(),
      );
    });

    test('or keeps this option when present, and takes the other when not', () {
      expect(const Some<int>(1).or(const Some<int>(2)), const Some<int>(1));
      expect(const None<int>().or(const Some<int>(2)), const Some<int>(2));
    });

    test('orElse only computes the fallback when absent', () {
      expect(const Some<int>(1).orElse(_mustNotRun), const Some<int>(1));
      expect(
        const None<int>().orElse(() => const Some<int>(2)),
        const Some<int>(2),
      );
    });
  });

  group('xor', () {
    test('returns whichever side is present when exactly one is', () {
      expect(const Some<int>(1).xor(const None<int>()), const Some<int>(1));
      expect(const None<int>().xor(const Some<int>(2)), const Some<int>(2));
    });

    test('is absent when both or neither are present', () {
      expect(const Some<int>(1).xor(const Some<int>(2)), const None<int>());
      expect(const None<int>().xor(const None<int>()), const None<int>());
    });
  });
}
