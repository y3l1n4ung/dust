import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

/// Throws if called, so a test can prove a lazy branch never ran.
Never _mustNotRun([Object? _]) => throw StateError('lazy callback ran');

const Result<int, String> _ok = Ok<int, String>(7);
const Result<int, String> _err = Err<int, String>('offline');

void main() {
  group('queries', () {
    test('isOkAnd and isErrAnd test only the variant that applies', () {
      expect(_ok.isOkAnd((value) => value > 5), isTrue);
      expect(_ok.isOkAnd((value) => value > 9), isFalse);
      expect(_err.isOkAnd(_mustNotRun), isFalse);

      expect(_err.isErrAnd((error) => error == 'offline'), isTrue);
      expect(_err.isErrAnd((error) => error.isEmpty), isFalse);
      expect(_ok.isErrAnd(_mustNotRun), isFalse);
    });

    test('ok and err keep one side as an Option', () {
      expect(_ok.ok(), const Some<int>(7));
      expect(_err.ok(), const None<int>());
      expect(_ok.err(), const None<String>());
      expect(_err.err(), const Some<String>('offline'));
    });

    test('a present null stays present', () {
      expect(const Ok<int?, String>(null).ok(), const Some<int?>(null));
    });
  });

  group('extraction', () {
    test('unwrap and expect return the value', () {
      expect(_ok.unwrap(), 7);
      expect(_ok.expect('never shown'), 7);
    });

    test('unwrap and expect throw a StateError naming the error', () {
      expect(
        _err.unwrap,
        throwsA(isA<StateError>().having(
          (error) => error.message,
          'message',
          'called unwrap() on Err: offline',
        )),
      );
      expect(
        () => _err.expect('the cache is warm'),
        throwsA(isA<StateError>().having(
          (error) => error.message,
          'message',
          'the cache is warm: offline',
        )),
      );
    });

    test('unwrapErr and expectErr return the error', () {
      expect(_err.unwrapErr(), 'offline');
      expect(_err.expectErr('never shown'), 'offline');
    });

    test('unwrapErr and expectErr throw a StateError naming the value', () {
      expect(
        _ok.unwrapErr,
        throwsA(isA<StateError>().having(
          (error) => error.message,
          'message',
          'called unwrapErr() on Ok: 7',
        )),
      );
      expect(
        () => _ok.expectErr('the request fails'),
        throwsA(isA<StateError>().having(
          (error) => error.message,
          'message',
          'the request fails: 7',
        )),
      );
    });
  });

  group('transforms', () {
    test('mapOr maps a value and falls back on an error', () {
      expect(_ok.mapOr('none', (value) => 'n=$value'), 'n=7');
      expect(_err.mapOr('none', _mustNotRun), 'none');
    });

    test('mapOrElse calls only the branch that applies', () {
      expect(_ok.mapOrElse(_mustNotRun, (value) => 'n=$value'), 'n=7');
      expect(
        _err.mapOrElse((error) => 'failed: $error', _mustNotRun),
        'failed: offline',
      );
    });

    test('inspect and inspectErr see one variant and return it unchanged', () {
      final seen = <Object>[];

      expect(_ok.inspect(seen.add), same(_ok));
      expect(_err.inspect(_mustNotRun), same(_err));
      expect(_err.inspectErr(seen.add), same(_err));
      expect(_ok.inspectErr(_mustNotRun), same(_ok));
      expect(seen, <Object>[7, 'offline']);
    });

    test('and keeps the first error, or takes the other result', () {
      expect(
        _ok.and(const Ok<String, String>('next')),
        const Ok<String, String>('next'),
      );
      expect(
        _err.and(const Ok<String, String>('next')),
        const Err<String, String>('offline'),
      );
    });

    test('or keeps the first value, or takes the other result', () {
      expect(_ok.or(const Err<int, int>(0)), const Ok<int, int>(7));
      expect(_err.or(const Ok<int, int>(1)), const Ok<int, int>(1));
      expect(_err.or(const Err<int, int>(2)), const Err<int, int>(2));
    });
  });

  group('flatten', () {
    test('removes one level of nesting', () {
      const Result<Result<int, String>, String> okOk = Ok(Ok(1));
      const Result<Result<int, String>, String> okErr = Ok(Err('inner'));
      const Result<Result<int, String>, String> err = Err('outer');

      expect(okOk.flatten(), const Ok<int, String>(1));
      expect(okErr.flatten(), const Err<int, String>('inner'));
      expect(err.flatten(), const Err<int, String>('outer'));
    });
  });

  group('transpose', () {
    test('never hides an error as absence', () {
      const Result<Option<int>, String> some = Ok(Some(1));
      const Result<Option<int>, String> none = Ok(None());
      const Result<Option<int>, String> err = Err('offline');

      expect(some.transpose(), const Some<Result<int, String>>(Ok(1)));
      expect(none.transpose(), const None<Result<int, String>>());
      expect(err.transpose(), const Some<Result<int, String>>(Err('offline')));
    });

    test('is the inverse of Option.transpose', () {
      const Result<Option<int>, String> row = Ok(Some(1));

      expect(row.transpose().transpose(), row);
    });
  });

  group('collect', () {
    test('returns every value in order', () {
      expect(
        const [Ok<int, String>(1), Ok<int, String>(2)].collect(),
        isA<Ok<List<int>, String>>()
            .having((result) => result.value, 'value', [1, 2]),
      );
      expect(
        const <Result<int, String>>[].collect(),
        isA<Ok<List<int>, String>>()
            .having((result) => result.value, 'value', isEmpty),
      );
    });

    test('stops at the first error without reading further', () {
      var read = 0;
      Iterable<Result<int, String>> results() sync* {
        read++;
        yield const Ok(1);
        read++;
        yield const Err('second');
        read++;
        yield const Err('third');
      }

      expect(results().collect(), const Err<List<int>, String>('second'));
      expect(read, 2);
    });
  });
}
