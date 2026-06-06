import 'package:dust_dart/derive.dart';
import 'package:test/test.dart';

void main() {
  test('Validate stores field validator configuration', () {
    const validate = Validate<String>(
      email: true,
      length: Length(min: 2),
      range: Range(min: 1, max: 9),
      contains: '@',
      doesNotContain: 'admin',
      regex: r'^[a-z]+$',
      mustMatch: 'confirmation',
      nested: true,
      custom: _customValidator,
      required: true,
      message: 'custom message',
    );

    expect(validate.email, isTrue);
    expect(validate.length, const Length(min: 2));
    expect(validate.range, const Range(min: 1, max: 9));
    expect(validate.contains, '@');
    expect(validate.doesNotContain, 'admin');
    expect(validate.regex, r'^[a-z]+$');
    expect(validate.mustMatch, 'confirmation');
    expect(validate.nested, isTrue);
    expect(validate.custom, _customValidator);
    expect(validate.required, isTrue);
    expect(validate.message, 'custom message');
  });

  test('ValidationResult exposes valid and invalid states', () {
    const valid = Valid();
    const error = ValidationError(field: 'email', message: 'Invalid email');
    const invalid = Invalid([error]);

    expect(valid.isValid, isTrue);
    expect(valid.errors, isEmpty);
    expect(valid, const Valid());
    expect(valid.hashCode, const Valid().hashCode);
    expect(invalid.isValid, isFalse);
    expect(invalid.errors, [error]);
    expect(invalid, const Invalid([error]));
    expect(invalid.hashCode, const Invalid([error]).hashCode);
    expect(invalid == const Invalid([]), isFalse);
    expect(invalid == const Invalid([_nameRequiredError]), isFalse);
  });

  test('ValidationError converts to json-compatible shape', () {
    const error = ValidationError(field: 'age', message: 'Must be at least 18');

    expect(error.toJson(), <String, Object?>{
      'field': 'age',
      'message': 'Must be at least 18',
    });
    expect(
      error.hashCode,
      const ValidationError(
        field: 'age',
        message: 'Must be at least 18',
      ).hashCode,
    );
  });

  test('ValidationException carries validation errors', () {
    const errors = [ValidationError(field: 'name', message: 'Required')];
    const exception = ValidationException(errors);

    expect(exception.errors, errors);
    expect(exception.toString(), 'ValidationException($errors)');
  });

  test('ValidationHelper validates email and URL shapes', () {
    expect(ValidationHelper.isEmail('dust@example.com'), isTrue);
    expect(ValidationHelper.isEmail('dust'), isFalse);
    expect(ValidationHelper.isUrl('https://example.com'), isTrue);
    expect(ValidationHelper.isUrl('not-a-url'), isFalse);
  });
}

ValidationError? _customValidator(String value) {
  return value.isEmpty
      ? const ValidationError(field: 'value', message: 'Required')
      : null;
}

const _nameRequiredError = ValidationError(field: 'name', message: 'Required');
