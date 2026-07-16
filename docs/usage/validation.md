# Validation

Dust generates typed model validation from field-level `@Validate(...)`
annotations.

## Add the Package

Install the Dust CLI from the [root guide](../../README.md#installation), then
add the Dart runtime package:

```bash
dart pub add dust_dart
```

The focused validation API is available from
`package:dust_dart/derive.dart`.

## Quick Start

Derive `Validate()` on the model and place rules on its fields:

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

  @Validate(email: true, message: 'Enter a valid email')
  final String email;

  @Validate(length: Length(min: 8), message: 'Use at least 8 characters')
  final String password;

  @Validate(mustMatch: 'password', message: 'Passwords do not match')
  final String confirmPassword;

  @Validate(range: Range(min: 18), message: 'You must be 18 or older')
  final int age;
}
```

Generate the implementation:

```bash
dust build
```

Validate without throwing:

```dart
final result = request.validate();

switch (result) {
  case Valid():
    submit(request);
  case Invalid(:final errors):
    showErrors(errors);
}
```

Use `request.validateOrThrow()` when an exception is the better control flow.
It throws `ValidationException` with the same error list.

> [!IMPORTANT]
> The model needs the matching `part` directive and generated mixin. Field
> annotations do not generate validation unless the class derives `Validate()`.

## Validation Result

`validate()` returns one of two typed results:

| Result | Meaning |
| :--- | :--- |
| `Valid` | No rule failed. `errors` is empty and `isValid` is `true`. |
| `Invalid` | One or more rules failed. `errors` contains `ValidationError` values. |

Each error contains a field path and message. Nested paths use dot notation,
such as `address.zipCode`.

## Rules

| Rule | Supported field types |
| :--- | :--- |
| `email: true` | `String`, `String?` |
| `url: true` | `String`, `String?` |
| `length: Length(...)` | `String`, `List`, `Set`, `Map`, including nullable forms |
| `range: Range(...)` | `int`, `double`, `num`, including nullable forms |
| `contains` | `String`, `String?` |
| `doesNotContain` | `String`, `String?` |
| `regex` | `String`, `String?` |
| `mustMatch` | Another field with the same type |
| `nested: true` | A model with generated validation |
| `custom` | A matching `FieldValidator<T>` callback |
| `required: true` | Nullable fields |

`Length` accepts `min`, `max`, or `exact`. `Range` accepts `min` and `max`.
Dust rejects incompatible field types, invalid bounds, empty `Length()` or
`Range()` rules, and `Length(exact: ...)` combined with `min` or `max` during
generation.

> [!NOTE]
> Rules on nullable fields ignore `null` by default. Add `required: true` when
> `null` should produce an error.

## Multiple Rules

Place multiple annotations on one field when each rule needs its own message:

```dart
@Validate(length: Length(min: 1), message: 'Enter an email')
@Validate(email: true, message: 'Enter a valid email')
final String email;
```

Rules run in annotation order and all failures are returned. When one
annotation contains multiple rules, its `message` applies to each rule.

## Cross-Field Validation

`mustMatch` compares the annotated field with another field on the same model:

```dart
@Validate(mustMatch: 'password', message: 'Passwords do not match')
final String confirmPassword;
```

Dust verifies that the referenced field exists and has the same type.

## Nested Validation

Both models derive `Validate()`, then the parent opts into nested validation:

```dart
@Derive([Validate()])
class Address with _$Address {
  const Address({required this.zipCode});

  @Validate(regex: r'^\d{5}$', message: 'Invalid ZIP code')
  final String zipCode;
}

@Derive([Validate()])
class Profile with _$Profile {
  const Profile({required this.address});

  @Validate(nested: true)
  final Address address;
}
```

An `Address.zipCode` failure is returned as `address.zipCode` on `Profile`.

## Custom Validators

A custom validator returns one error or `null`:

```dart
@Validate<String>(custom: validateBusinessEmail)
final String email;

ValidationError? validateBusinessEmail(String value) {
  if (value.endsWith('@blocked.example')) {
    return const ValidationError(
      field: 'email',
      message: 'Email domain is blocked',
    );
  }
  return null;
}
```

> [!TIP]
> Use built-in rules for common checks and custom validators for product or
> domain rules that do not belong in the generator.

## Flutter Forms

In Flutter packages, Dust also generates `TextFormField.validator` helpers for
validated `String`, `int`, `double`, and `num` fields:

```dart
TextFormField(
  controller: emailController,
  validator: validateSignupRequestEmailInput,
)
```

Cross-field rules receive the current model:

```dart
TextFormField(
  controller: confirmPasswordController,
  validator: (value) =>
      validateSignupRequestConfirmPasswordInput(currentRequest(), value),
)
```

Numeric helpers parse the text before applying range rules. A parse failure
uses the annotation message when available, otherwise `Invalid number`.
Pure Dart packages generate model validation without Flutter form helpers.

## Examples

- [Product showcase validation model](../../examples/product_showcase/lib/models/latest_dart_derive_showcase.dart)
- [Product showcase validation test](../../examples/product_showcase/test/generated_latest_dart_derive_showcase_test.dart)
- [Complete validation rule fixture](../../examples/benchmark_project/tool/src/validate_templates.dart)
- [Complete validation rule tests](../../examples/benchmark_project/test/validation_rules_test.dart)
- [Shopping app registration model](../../examples/shopping_app/lib/features/auth/models/register_request.dart)
- [Shopping app form integration](../../examples/shopping_app/lib/features/auth/views/register_screen.dart)
