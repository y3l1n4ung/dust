import 'template_support.dart';

const validationShowcaseFile = 'validation_showcase';

String renderValidationShowcaseFile() {
  return renderFile(
    fileName: validationShowcaseFile,
    imports: ["import 'package:dust_dart/derive.dart';"],
    declarations: [
      '''
@Derive([Validate()])
class BenchmarkAddressValidation with _\$BenchmarkAddressValidation {
  const BenchmarkAddressValidation({required this.city, required this.zipCode});

  @Validate(length: Length(min: 2, max: 64), message: 'City is invalid')
  final String city;

  @Validate(regex: r'^\\d{5}\$', message: 'ZIP is invalid')
  final String zipCode;
}''',
      '''
@Derive([Validate()])
class BenchmarkSignupValidation with _\$BenchmarkSignupValidation {
  const BenchmarkSignupValidation({
    required this.email,
    required this.password,
    required this.confirmPassword,
    required this.age,
    required this.tags,
    required this.address,
    this.website,
  });

  @Validate(email: true, message: 'Email is invalid')
  @Validate<String>(custom: validateBenchmarkEmail)
  final String email;

  @Validate(length: Length(min: 8), message: 'Password is too short')
  @Validate(doesNotContain: 'password', message: 'Password is too weak')
  final String password;

  @Validate(mustMatch: 'password', message: 'Passwords must match')
  final String confirmPassword;

  @Validate(range: Range(min: 18, max: 120), message: 'Age is invalid')
  final int age;

  @Validate(length: Length(min: 1, max: 5), message: 'Tags are invalid')
  final List<String> tags;

  @Validate(nested: true)
  final BenchmarkAddressValidation address;

  @Validate(url: true, message: 'Website is invalid')
  final String? website;
}''',
      '''
@Derive([Validate()])
class BenchmarkRulesValidation with _\$BenchmarkRulesValidation {
  const BenchmarkRulesValidation({
    required this.email,
    required this.url,
    required this.code,
    required this.shortText,
    required this.longText,
    required this.lowScore,
    required this.highScore,
    required this.containsDust,
    required this.cleanText,
    required this.pattern,
    required this.password,
    required this.confirmPassword,
    required this.tags,
    required this.aliases,
    required this.settings,
    required this.address,
    required this.customValue,
    this.requiredToken,
    this.optionalWebsite,
  });

  @Validate(email: true, message: 'Email rule')
  final String email;

  @Validate(url: true, message: 'URL rule')
  final String url;

  @Validate(length: Length(exact: 3), message: 'Exact length rule')
  final String code;

  @Validate(length: Length(min: 2), message: 'Min length rule')
  final String shortText;

  @Validate(length: Length(max: 4), message: 'Max length rule')
  final String longText;

  @Validate(range: Range(min: 1), message: 'Min range rule')
  final int lowScore;

  @Validate(range: Range(max: 10), message: 'Max range rule')
  final int highScore;

  @Validate(contains: 'dust', message: 'Contains rule')
  final String containsDust;

  @Validate(doesNotContain: 'bad', message: 'Does-not-contain rule')
  final String cleanText;

  @Validate(regex: r'^[A-Z]{2}\\d{2}\$', message: 'Regex rule')
  final String pattern;

  @Validate(length: Length(min: 8), message: 'Password length rule')
  final String password;

  @Validate(mustMatch: 'password', message: 'Must-match rule')
  final String confirmPassword;

  @Validate(length: Length(min: 2, max: 3), message: 'List length rule')
  final List<String> tags;

  @Validate(length: Length(exact: 2), message: 'Set length rule')
  final Set<String> aliases;

  @Validate(length: Length(min: 1, max: 2), message: 'Map length rule')
  final Map<String, String> settings;

  @Validate(nested: true)
  final BenchmarkAddressValidation address;

  @Validate<String>(custom: validateBenchmarkCustomValue)
  final String customValue;

  @Validate(required: true, message: 'Required rule')
  final String? requiredToken;

  @Validate(url: true, message: 'Nullable URL rule')
  final String? optionalWebsite;
}''',
      '''
ValidationError? validateBenchmarkEmail(String value) {
  if (value.endsWith('@blocked.example')) {
    return const ValidationError(
      field: r'email',
      message: r'Blocked email domain',
    );
  }
  return null;
}''',
      '''
ValidationError? validateBenchmarkCustomValue(String value) {
  if (value == 'blocked') {
    return const ValidationError(
      field: r'customValue',
      message: r'Custom rule',
    );
  }
  return null;
}''',
    ],
  );
}
