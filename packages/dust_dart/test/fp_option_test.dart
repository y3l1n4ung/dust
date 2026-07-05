import 'package:dust_dart/derive.dart' as derive;
import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

void main() {
  test('None represents an absent value', () {
    const Option<String?> option = None<String?>();

    expect(option, isA<None<String?>>());
    expect(option.isSome, isFalse);
    expect(option.isNone, isTrue);
    expect(option.map((value) => value?.length), const None<int?>());
    expect(option.andThen((value) => Some<int?>(value?.length)),
        const None<int?>());
    expect(option.unwrapOr('current'), 'current');
    expect(option.unwrapOrElse(() => 'computed'), 'computed');
    expect(
        option.match(some: (value) => value, none: () => 'matched'), 'matched');
    expect(option, const None<String?>());
    expect(option.hashCode, const None<String?>().hashCode);
    expect(option == const Some<String?>('John'), isFalse);
    expect(option.toString(), 'None()');
    expect(
        switch (option) {
          None<String?>() => 'current',
          Some<String?>(:final value) => value,
        },
        'current');
  });

  test('Some carries a non-null value', () {
    const Option<String?> option = Some<String?>('John');

    expect(option, isA<Some<String?>>());
    expect(option.isSome, isTrue);
    expect(option.isNone, isFalse);
    expect(option.map((value) => value?.length), const Some<int?>(4));
    expect(option.andThen((value) => Some<int?>(value?.length)),
        const Some<int?>(4));
    expect(option.unwrapOr('current'), 'John');
    expect(option.unwrapOrElse(() => 'computed'), 'John');
    expect(option.match(some: (value) => value, none: () => 'matched'), 'John');
    expect(option, const Some<String?>('John'));
    expect(option.hashCode, const Some<String?>('John').hashCode);
    expect(option == const Some<String?>('Jane'), isFalse);
    expect(option.toString(), 'Some(John)');
    expect(
        switch (option) {
          None<String?>() => 'current',
          Some<String?>(:final value) => value,
        },
        'John');
  });

  test('Some can carry a null value', () {
    const Option<String?> option = Some<String?>(null);

    expect(option, isA<Some<String?>>());
    expect(
        switch (option) {
          None<String?>() => 'current',
          Some<String?>(:final value) => value,
        },
        isNull);
    expect(option.unwrapOr('fallback'), isNull);
    expect(option.match(some: (value) => value, none: () => 'matched'), isNull);
  });

  test('Option equality is symmetric across generic variance', () {
    const precise = Some<int>(2);
    const wider = Some<num>(2);
    const missingPrecise = None<int>();
    const missingWider = None<num>();

    expect(precise == wider, isTrue);
    expect(wider == precise, isTrue);
    expect(missingPrecise == missingWider, isTrue);
    expect(missingWider == missingPrecise, isTrue);
  });

  test('derive barrel re-exports generated-code Option symbols', () {
    const option = derive.Some<String?>(null);

    expect(option, isA<derive.Some<String?>>());
  });

  test('fp barrel exports Result and Unit primitives', () {
    const result = Ok<Unit, String>(unit);

    expect(result.value, unit);
    expect(result.isOk, isTrue);
  });
}
