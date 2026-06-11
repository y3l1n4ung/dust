import 'package:dust_dart/derive.dart';
import 'package:dust_benchmark_project/generated_models/validation_showcase.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('validation kitchen sink reports every supported rule', () {
    final result = _invalidRulesRequest().validate();

    switch (result) {
      case Valid():
        fail('expected invalid validation result');
      case Invalid(:final errors):
        expect(
          errors.map((error) => '${error.field}:${error.message}').toList(),
          equals([
            'email:Email rule',
            'url:URL rule',
            'code:Exact length rule',
            'shortText:Min length rule',
            'longText:Max length rule',
            'lowScore:Min range rule',
            'highScore:Max range rule',
            'containsDust:Contains rule',
            'cleanText:Does-not-contain rule',
            'pattern:Regex rule',
            'password:Password length rule',
            'confirmPassword:Must-match rule',
            'tags:List length rule',
            'aliases:Set length rule',
            'settings:Map length rule',
            'address.city:City is invalid',
            'address.zipCode:ZIP is invalid',
            'customValue:Custom rule',
            'requiredToken:Required rule',
            'optionalWebsite:Nullable URL rule',
          ]),
        );
    }
  });

  test('validation kitchen sink throws the same generated errors', () {
    expect(
      _invalidRulesRequest().validateOrThrow,
      throwsA(
        isA<ValidationException>().having(
          (error) => error.errors.map((e) => e.field).toList(),
          'fields',
          equals([
            'email',
            'url',
            'code',
            'shortText',
            'longText',
            'lowScore',
            'highScore',
            'containsDust',
            'cleanText',
            'pattern',
            'password',
            'confirmPassword',
            'tags',
            'aliases',
            'settings',
            'address.city',
            'address.zipCode',
            'customValue',
            'requiredToken',
            'optionalWebsite',
          ]),
        ),
      ),
    );
  });

  test('validation kitchen sink accepts valid mixed model', () {
    final result = _validRulesRequest(optionalWebsite: null).validate();

    expect(result, isA<Valid>());
  });

  test('generated input validators cover parsing and cross-field cases', () {
    final request = _validRulesRequest(
      password: 'long-password',
      confirmPassword: 'long-password',
    );

    expect(validateBenchmarkRulesValidationEmailInput('bad'), 'Email rule');
    expect(validateBenchmarkRulesValidationUrlInput('bad-url'), 'URL rule');
    expect(
      validateBenchmarkRulesValidationCodeInput('AB'),
      'Exact length rule',
    );
    expect(
      validateBenchmarkRulesValidationLowScoreInput('abc'),
      'Min range rule',
    );
    expect(
      validateBenchmarkRulesValidationHighScoreInput('11'),
      'Max range rule',
    );
    expect(
      validateBenchmarkRulesValidationContainsDustInput('clean'),
      'Contains rule',
    );
    expect(
      validateBenchmarkRulesValidationCleanTextInput('bad value'),
      'Does-not-contain rule',
    );
    expect(validateBenchmarkRulesValidationPatternInput('bad'), 'Regex rule');
    expect(
      validateBenchmarkRulesValidationConfirmPasswordInput(
        request,
        'different',
      ),
      'Must-match rule',
    );
    expect(
      validateBenchmarkRulesValidationRequiredTokenInput(null),
      'Required rule',
    );
    expect(validateBenchmarkRulesValidationOptionalWebsiteInput(null), isNull);
  });
}

BenchmarkRulesValidation _invalidRulesRequest() {
  return BenchmarkRulesValidation(
    email: 'bad',
    url: 'bad-url',
    code: 'AB',
    shortText: 'A',
    longText: 'ABCDE',
    lowScore: 0,
    highScore: 11,
    containsDust: 'clean',
    cleanText: 'bad value',
    pattern: 'bad',
    password: 'short',
    confirmPassword: 'different',
    tags: const ['one'],
    aliases: const {'one'},
    settings: const {},
    address: const BenchmarkAddressValidation(city: 'A', zipCode: 'abc'),
    customValue: 'blocked',
    optionalWebsite: 'bad-url',
  );
}

BenchmarkRulesValidation _validRulesRequest({
  String password = 'long-password',
  String confirmPassword = 'long-password',
  String? optionalWebsite = 'https://example.com',
}) {
  return BenchmarkRulesValidation(
    email: 'ok@example.com',
    url: 'https://example.com',
    code: 'ABC',
    shortText: 'AB',
    longText: 'ABCD',
    lowScore: 1,
    highScore: 10,
    containsDust: 'hello dust',
    cleanText: 'clean',
    pattern: 'AB12',
    password: password,
    confirmPassword: confirmPassword,
    tags: const ['one', 'two'],
    aliases: const {'one', 'two'},
    settings: const {'theme': 'dark'},
    address: const BenchmarkAddressValidation(city: 'Yangon', zipCode: '11111'),
    customValue: 'allowed',
    requiredToken: 'token',
    optionalWebsite: optionalWebsite,
  );
}
