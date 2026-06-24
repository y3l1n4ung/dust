import 'package:dust_dart/derive.dart' as derive;
import 'package:dust_dart/fp.dart';
import 'package:test/test.dart';

void main() {
  test('None represents an absent value', () {
    const Option<String?> option = None<String?>();

    expect(option, isA<None<String?>>());
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
