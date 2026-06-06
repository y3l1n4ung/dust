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
ValidationError? validateBenchmarkEmail(String value) {
  if (value.endsWith('@blocked.example')) {
    return const ValidationError(
      field: r'email',
      message: r'Blocked email domain',
    );
  }
  return null;
}''',
    ],
  );
}
