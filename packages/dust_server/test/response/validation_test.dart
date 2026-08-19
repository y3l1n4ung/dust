import 'package:dust_dart/derive.dart';
import 'package:dust_server/server.dart';
import 'package:test/test.dart';

import '../support.dart';

/// Two ways a `Validate()` derive reaches HTTP: through `ValidatedExtractable`
/// after decoding, and through a generated `deserialize` that enforces its own
/// constraints and throws. Both have to arrive as the same 422 naming the same
/// fields, or a client sees two error formats for one kind of mistake.

final class _Signup implements Validatable {
  const _Signup(this.email, this.password);

  /// Stands in for a generated `deserialize` that validates as it builds.
  static _Signup deserialize(Map<String, Object?> json) {
    final value = _Signup(
      json['email']! as String,
      json['password']! as String,
    );
    value.validateOrThrow();
    return value;
  }

  final String email;
  final String password;

  @override
  ValidationResult validate() {
    final errors = <ValidationError>[
      if (!email.contains('@'))
        const ValidationError(field: 'email', message: 'must be an address'),
      if (password.length < 8)
        const ValidationError(field: 'password', message: 'too short'),
      if (password == email)
        const ValidationError(field: 'password', message: 'must differ'),
    ];
    return errors.isEmpty ? const Valid() : Invalid(errors);
  }

  @override
  void validateOrThrow() {
    if (validate() case Invalid(:final errors)) {
      throw ValidationException(errors);
    }
  }
}

void main() {
  group('a ValidationResult', () {
    test('has no rejection when it is valid', () {
      expect(const Valid().rejection, isNull);
    });

    test('becomes a 422', () {
      const result = Invalid([
        ValidationError(field: 'email', message: 'must be an address'),
      ]);

      expect(result.rejection!.status, 422);
    });

    test('says the failure was validation, not shape', () {
      const result = Invalid([
        ValidationError(field: 'email', message: 'must be an address'),
      ]);

      expect(result.rejection!.message, 'Validation failed');
    });

    test('groups two failures on one field under that field', () {
      const result = Invalid([
        ValidationError(field: 'password', message: 'too short'),
        ValidationError(field: 'password', message: 'must differ'),
      ]);

      expect(result.rejection!.fields, {
        'password': ['too short', 'must differ'],
      });
    });

    test('keeps the order the errors were reported in', () {
      const result = Invalid([
        ValidationError(field: 'a', message: 'first'),
        ValidationError(field: 'b', message: 'second'),
      ]);

      expect(result.rejection!.fields.keys, ['a', 'b']);
    });
  });

  group('validateOrReject', () {
    test('does nothing when the value satisfies its constraints', () {
      expect(
        () => const _Signup('a@b.test', 'longenough').validateOrReject(),
        returnsNormally,
      );
    });

    test('throws the 422 rather than a ValidationException', () {
      expect(
        () => const _Signup('nope', 'short').validateOrReject(),
        throwsA(isA<Rejection>().having((r) => r.status, 'status', 422)),
      );
    });

    test('carries every failed field into the thrown rejection', () {
      try {
        const _Signup('nope', 'short').validateOrReject();
        fail('expected a rejection');
      } on Rejection catch (rejection) {
        expect(rejection.fields.keys, ['email', 'password']);
      }
    });
  });

  group('a deserializer that validates as it builds', () {
    test('answers 422 rather than a shape failure', () async {
      final outcome = await const JsonExtractable(_Signup.deserialize).extract(
        jsonRequest('POST', '/', '{"email":"nope","password":"short"}'),
      );

      expect(expectStatus(outcome, 422).message, 'Validation failed');
    });

    test('hands back the fields the exception named', () async {
      final outcome = await const JsonExtractable(_Signup.deserialize).extract(
        jsonRequest('POST', '/', '{"email":"nope","password":"short"}'),
      );

      expect(expectStatus(outcome, 422).fields, {
        'email': ['must be an address'],
        'password': ['too short'],
      });
    });

    test('still reports a shape failure as a shape failure', () async {
      final outcome = await const JsonExtractable(_Signup.deserialize).extract(
        jsonRequest('POST', '/', '{"email":7,"password":"longenough"}'),
      );

      final rejection = expectStatus(outcome, 422);
      expect(rejection.fields, isEmpty);
      expect(rejection.message, contains('does not match the expected shape'));
    });

    test('agrees with the extractor that validates after decoding', () async {
      const throughDeserialize = JsonExtractable(_Signup.deserialize);
      const afterDecoding = ValidatedExtractable(
        JsonExtractable(_Signup.deserialize),
      );
      const body = '{"email":"nope","password":"short"}';

      final first = await throughDeserialize.extract(
        jsonRequest('POST', '/', body),
      );
      final second = await afterDecoding.extract(
        jsonRequest('POST', '/', body),
      );

      expect(expectErr(first).fields, expectErr(second).fields);
      expect(expectErr(first).status, expectErr(second).status);
    });
  });
}
