# Validation

Dust can generate model validation from field-level `@Validate(...)` annotations.
Use this for request models, form models, checkout models, and other DTOs where
validation should live beside the data contract.

---

## Basic Example

```dart
import 'package:dust_dart/derive.dart';

part 'signup_request.g.dart';

@Derive([Validate()])
class SignupRequest with _$SignupRequest {
  const SignupRequest({
    required this.email,
    required this.password,
    required this.confirmPassword,
    required this.age,
  });

  @Validate(email: true, message: 'Invalid email')
  final String email;

  @Validate(length: Length(min: 8), message: 'Min 8 characters')
  final String password;

  @Validate(mustMatch: 'password', message: 'Passwords do not match')
  final String confirmPassword;

  @Validate(range: Range(min: 18, max: 120), message: 'Must be 18-120')
  final int age;
}
```

Generated usage:

```dart
final request = SignupRequest(
  email: emailController.text,
  password: passwordController.text,
  confirmPassword: confirmPasswordController.text,
  age: int.parse(ageController.text),
);

final result = request.validate();
if (result case Invalid(:final errors)) {
  // Render or log errors.
}

request.validateOrThrow();
```

---

## Generated API

For each `@Derive([Validate()])` class, Dust generates:

```dart
ValidationResult validate();
void validateOrThrow();
String? validateSignupRequestEmailInput(String? value);
String? validateSignupRequestPasswordInput(String? value);
String? validateSignupRequestAgeInput(String? value);
String? validateSignupRequestConfirmPasswordInput(
  SignupRequest self,
  String? value,
);
```

`validate()` checks the typed object. `validateClassFieldInput(...)` checks raw
text input for Flutter `TextFormField` usage. Cross-field validators take the
current request as the first argument. App code imports the source library, for
example `signup_request.dart`; never import `signup_request.g.dart` directly.

---

## Flutter Form Usage

Keep form code small by using generated top-level validators directly.

```dart
final _emailController = TextEditingController();
final _passwordController = TextEditingController();
final _confirmPasswordController = TextEditingController();
final _ageController = TextEditingController();

SignupRequest _request() {
  return SignupRequest(
    email: _emailController.text,
    password: _passwordController.text,
    confirmPassword: _confirmPasswordController.text,
    age: int.tryParse(_ageController.text) ?? 0,
  );
}
```

Then wire fields directly:

```dart
TextFormField(
  controller: _emailController,
  validator: validateSignupRequestEmailInput,
)

TextFormField(
  controller: _passwordController,
  validator: validateSignupRequestPasswordInput,
)

TextFormField(
  controller: _ageController,
  validator: validateSignupRequestAgeInput,
)

TextFormField(
  controller: _confirmPasswordController,
  validator: (value) {
    return validateSignupRequestConfirmPasswordInput(_request(), value);
  },
)
```

Numeric form validators parse `String?` input before applying `Range(...)`.
If parsing fails, the field returns that rule's `message` when present.

---

## Supported Rules

| Rule | Field Type | Example |
| :--- | :--- | :--- |
| `email` | `String`, `String?` | `@Validate(email: true)` |
| `url` | `String`, `String?` | `@Validate(url: true)` |
| `length` | `String`, `List`, `Set`, `Map` | `@Validate(length: Length(min: 1, max: 64))` |
| `range` | `int`, `double`, `num` | `@Validate(range: Range(min: 18, max: 120))` |
| `contains` | `String`, `String?` | `@Validate(contains: '@')` |
| `doesNotContain` | `String`, `String?` | `@Validate(doesNotContain: 'password')` |
| `regex` | `String`, `String?` | `@Validate(regex: r'^\\d{5}$')` |
| `mustMatch` | same field type | `@Validate(mustMatch: 'password')` |
| `nested` | `@Derive([Validate()])` type | `@Validate(nested: true)` |
| `custom` | callback type | `@Validate<String>(custom: validateEmailDomain)` |
| `required` | nullable fields | `@Validate(required: true)` |

---

## Length And Range

Use typed config objects, not records or maps.

```dart
@Validate(length: Length(min: 1), message: 'Required')
final String name;

@Validate(length: Length(exact: 5), message: 'ZIP must be 5 digits')
final String zipCode;

@Validate(range: Range(min: 18, max: 120), message: 'Must be 18-120')
final int age;
```

Dust validates these at generation time:

- `Length(...)` only supports integer `min`, `max`, and `exact`.
- `Length(exact: ...)` cannot be combined with `min` or `max`.
- `Range(...)` only supports numeric `min` and `max`.
- `min` must be less than or equal to `max`.
- Raw records such as `length: (min: 1)` are rejected.

---

## Custom Field Validator

Custom validators are field-level and type-safe.

```dart
@Validate<String>(custom: validateBusinessEmail)
final String email;

ValidationError? validateBusinessEmail(String value) {
  if (value.endsWith('@blocked.example')) {
    return const ValidationError(
      field: 'email',
      message: 'Blocked email domain',
    );
  }
  return null;
}
```

---

## Nested Validation

Nested objects must also derive `Validate()`.

```dart
@Derive([Validate()])
class Address with _$Address {
  const Address({required this.zipCode});

  @Validate(length: Length(exact: 5), message: 'ZIP is invalid')
  final String zipCode;
}

@Derive([Validate()])
class Profile with _$Profile {
  const Profile({required this.address});

  @Validate(nested: true)
  final Address address;
}
```

Nested errors use dot paths, for example `address.zipCode`.

---

## Generated Shape

Simplified generated output:

```dart
mixin _$SignupRequest {
  ValidationResult validate() {
    final self = this as SignupRequest;
    final errors = <ValidationError>[];
    _validateSignupRequestEmail(self.email, errors);
    _validateSignupRequestPassword(self.password, errors);
    _validateSignupRequestAge(self.age, errors);
    return errors.isEmpty ? const Valid() : Invalid(errors);
  }
}

void _validateSignupRequestEmail(String email, List<ValidationError> errors) {
  if (!ValidationHelper.isEmail(email)) {
    errors.add(ValidationError(field: 'email', message: 'Invalid email'));
  }
}

String? validateSignupRequestEmailInput(String? value) {
  final errors = <ValidationError>[];
  _validateSignupRequestEmail(value ?? '', errors);
  return errors.isEmpty ? null : errors.first.message;
}
```
