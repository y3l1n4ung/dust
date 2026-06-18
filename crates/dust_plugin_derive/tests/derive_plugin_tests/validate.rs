use dust_ir::{FunctionIr, LibraryIr, NameIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use crate::{
    support::members_for_class,
    support::span,
    validate_support::{class, field, library, validate},
};

#[test]
fn emits_validate_for_string_number_and_matching_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(&validation_library(), &SymbolPlan::default());
    let members = members_for_class(&contribution, "SignupRequest");

    assert_eq!(
        members,
        [r#"/// Validates this `SignupRequest`.
///
/// Usage:
/// ```dart
/// final result = value.validate();
/// if (result case Invalid(:final errors)) {
///   print(errors.first.message);
/// }
/// ```
ValidationResult validate() {
  final self = this as SignupRequest;
  final errors = <ValidationError>[];
  _SignupRequestValidation._validateEmail(self.email, errors);
  _SignupRequestValidation._validateAge(self.age, errors);
  _SignupRequestValidation._validatePassword(self.password, errors);
  _SignupRequestValidation._validateConfirmPassword(self, self.confirmPassword, errors);
  return errors.isEmpty ? const Valid() : Invalid(errors);
}

/// Throws [ValidationException] when this `SignupRequest` is invalid.
///
/// Usage:
/// ```dart
/// value.validateOrThrow();
/// ```
void validateOrThrow() {
  final result = validate();
  if (result case Invalid(errors: final errors)) {
    throw ValidationException(errors);
  }
}"#
        .to_owned()]
        .as_slice()
    );

    assert_eq!(
        contribution.support_types,
        [r#"/// TextFormField validator for `SignupRequest.email`.
///
/// Usage:
/// ```dart
/// TextFormField(
///   validator: validateSignupRequestEmailInput,
/// )
/// ```
String? validateSignupRequestEmailInput(String? value) {
  return _SignupRequestValidation.validateEmailInput(value);
}

/// TextFormField validator for `SignupRequest.age`.
///
/// Usage:
/// ```dart
/// TextFormField(
///   validator: validateSignupRequestAgeInput,
/// )
/// ```
String? validateSignupRequestAgeInput(String? value) {
  return _SignupRequestValidation.validateAgeInput(value);
}

/// TextFormField validator for `SignupRequest.password`.
///
/// Usage:
/// ```dart
/// TextFormField(
///   validator: validateSignupRequestPasswordInput,
/// )
/// ```
String? validateSignupRequestPasswordInput(String? value) {
  return _SignupRequestValidation.validatePasswordInput(value);
}

/// TextFormField validator for `SignupRequest.confirmPassword`.
///
/// Usage:
/// ```dart
/// TextFormField(
///   validator: (value) => validateSignupRequestConfirmPasswordInput(self, value),
/// )
/// ```
String? validateSignupRequestConfirmPasswordInput(
  SignupRequest self,
  String? value,
) {
  return _SignupRequestValidation.validateConfirmPasswordInput(self, value);
}

extension _SignupRequestValidation on SignupRequest {
  static void _validateEmail(String email, List<ValidationError> errors) {
    if (!ValidationHelper.isEmail(email)) {
      errors.add(ValidationError(field: 'email', message: 'Invalid email'));
    }
  }

  static String? validateEmailInput(String? value) {
    final errors = <ValidationError>[];
    _validateEmail(value ?? '', errors);
    return errors.isEmpty ? null : errors.first.message;
  }

  static void _validateAge(int age, List<ValidationError> errors) {
    if (age < 18) {
      errors.add(ValidationError(field: 'age', message: 'Too small'));
    }
    if (age > 120) {
      errors.add(ValidationError(field: 'age', message: 'Too large'));
    }
  }

  static String? validateAgeInput(String? value) {
    final errors = <ValidationError>[];
    final age = int.tryParse(value ?? '');
    if (age == null) {
      errors.add(ValidationError(field: 'age', message: 'Invalid number'));
    } else {
      _validateAge(age, errors);
    }
    return errors.isEmpty ? null : errors.first.message;
  }

  static void _validatePassword(String password, List<ValidationError> errors) {
    if (password.length < 8) {
      errors.add(ValidationError(field: 'password', message: 'At least 8 characters'));
    }
    if (!RegExp('^(?=.*[A-Z]).+\$').hasMatch(password)) {
      errors.add(ValidationError(field: 'password', message: 'Need uppercase'));
    }
  }

  static String? validatePasswordInput(String? value) {
    final errors = <ValidationError>[];
    _validatePassword(value ?? '', errors);
    return errors.isEmpty ? null : errors.first.message;
  }

  static void _validateConfirmPassword(
    SignupRequest self,
    String confirmPassword,
    List<ValidationError> errors,
  ) {
    if (confirmPassword != self.password) {
      errors.add(ValidationError(field: 'confirmPassword', message: 'Fields do not match'));
    }
  }

  static String? validateConfirmPasswordInput(
    SignupRequest self,
    String? value,
  ) {
    final errors = <ValidationError>[];
    _validateConfirmPassword(self, value ?? '', errors);
    return errors.isEmpty ? null : errors.first.message;
  }

}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn rejects_public_validator_name_collisions() {
    let plugin = register_plugin();
    let mut library = validation_library();
    library.functions.push(FunctionIr {
        name: name("validateSignupRequestEmailInput"),
        return_type: TypeIr::named("String").nullable(),
        params: Vec::new(),
        annotations: Vec::new(),
        span: span(90, 120),
    });

    let diagnostics = plugin.validate(&library);
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "generated validator `validateSignupRequestEmailInput` for `SignupRequest.email` conflicts with an existing top-level declaration",
        ]
    );
}

#[test]
fn emits_nullable_nested_custom_and_class_validation() {
    let plugin = register_plugin();
    let contribution = plugin.emit(&nested_library(), &SymbolPlan::default());
    let members = members_for_class(&contribution, "Profile");

    assert_eq!(
        members,
        [r#"/// Validates this `Profile`.
///
/// Usage:
/// ```dart
/// final result = value.validate();
/// if (result case Invalid(:final errors)) {
///   print(errors.first.message);
/// }
/// ```
ValidationResult validate() {
  final self = this as Profile;
  final errors = <ValidationError>[];
  _ProfileValidation._validateBio(self.bio, errors);
  _ProfileValidation._validateAddress(self.address, errors);
  _ProfileValidation._validatePhone(self.phone, errors);
  return errors.isEmpty ? const Valid() : Invalid(errors);
}

/// Throws [ValidationException] when this `Profile` is invalid.
///
/// Usage:
/// ```dart
/// value.validateOrThrow();
/// ```
void validateOrThrow() {
  final result = validate();
  if (result case Invalid(errors: final errors)) {
    throw ValidationException(errors);
  }
}"#
        .to_owned()]
        .as_slice()
    );

    assert_eq!(
        contribution.support_types,
        [
            r#"/// TextFormField validator for `Address.zip`.
///
/// Usage:
/// ```dart
/// TextFormField(
///   validator: validateAddressZipInput,
/// )
/// ```
String? validateAddressZipInput(String? value) {
  return _AddressValidation.validateZipInput(value);
}

extension _AddressValidation on Address {
  static void _validateZip(String zip, List<ValidationError> errors) {
    if (zip.length != 5) {
      errors.add(ValidationError(field: 'zip', message: 'Invalid length'));
    }
  }

  static String? validateZipInput(String? value) {
    final errors = <ValidationError>[];
    _validateZip(value ?? '', errors);
    return errors.isEmpty ? null : errors.first.message;
  }

}"#
            .to_owned(),
            r#"/// TextFormField validator for `Profile.bio`.
///
/// Usage:
/// ```dart
/// TextFormField(
///   validator: validateProfileBioInput,
/// )
/// ```
String? validateProfileBioInput(String? value) {
  return _ProfileValidation.validateBioInput(value);
}

/// TextFormField validator for `Profile.phone`.
///
/// Usage:
/// ```dart
/// TextFormField(
///   validator: validateProfilePhoneInput,
/// )
/// ```
String? validateProfilePhoneInput(String? value) {
  return _ProfileValidation.validatePhoneInput(value);
}

extension _ProfileValidation on Profile {
  static void _validateBio(String? bio, List<ValidationError> errors) {
    if (bio != null) {
      if (bio.length > 200) {
        errors.add(ValidationError(field: 'bio', message: 'Too long'));
      }
    }
  }

  static String? validateBioInput(String? value) {
    final errors = <ValidationError>[];
    _validateBio(value, errors);
    return errors.isEmpty ? null : errors.first.message;
  }

  static void _validateAddress(Address address, List<ValidationError> errors) {
    final addressValidation = address.validate();
    if (addressValidation case Invalid(errors: final nestedErrors)) {
      for (final error in nestedErrors) {
        errors.add(ValidationError(field: 'address.${error.field}', message: error.message));
      }
    }
  }

  static void _validatePhone(String phone, List<ValidationError> errors) {
    final phoneCustomError = Profile.checkPhone(phone);
    if (phoneCustomError != null) {
      errors.add(phoneCustomError);
    }
  }

  static String? validatePhoneInput(String? value) {
    final errors = <ValidationError>[];
    _validatePhone(value ?? '', errors);
    return errors.isEmpty ? null : errors.first.message;
  }

}"#
            .to_owned(),
        ]
        .as_slice()
    );
}

fn validation_library() -> LibraryIr {
    let mut class = class("SignupRequest");
    class.fields = vec![
        field("email", TypeIr::string(), vec![validate("(email: true)")]),
        field(
            "age",
            TypeIr::int(),
            vec![validate("(range: Range(min: 18, max: 120))")],
        ),
        field(
            "password",
            TypeIr::string(),
            vec![
                validate("(length: Length(min: 8), message: 'At least 8 characters')"),
                validate("(regex: r'^(?=.*[A-Z]).+$', message: 'Need uppercase')"),
            ],
        ),
        field(
            "confirmPassword",
            TypeIr::string(),
            vec![validate("(mustMatch: 'password')")],
        ),
    ];
    library(vec![class])
}

fn nested_library() -> LibraryIr {
    let mut address = class("Address");
    address.fields = vec![field(
        "zip",
        TypeIr::string(),
        vec![validate("(length: Length(exact: 5))")],
    )];

    let mut profile = class("Profile");
    profile.fields = vec![
        field(
            "bio",
            TypeIr::string().nullable(),
            vec![validate("(length: Length(max: 200))")],
        ),
        field(
            "address",
            TypeIr::named("Address"),
            vec![validate("(nested: true)")],
        ),
        field(
            "phone",
            TypeIr::string(),
            vec![validate("(custom: Profile.checkPhone)")],
        ),
    ];
    library(vec![address, profile])
}

fn name(source: &str) -> NameIr {
    NameIr {
        source: source.to_owned(),
        short: source.to_owned(),
        prefix: None,
        span: span(0, source.len() as u32),
    }
}
