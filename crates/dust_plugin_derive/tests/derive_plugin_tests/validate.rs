use std::sync::Arc;

use dust_ir::{DartFileIr, FunctionIr, NameIr, TypeIr};
use dust_plugin_api::{
    DustPlugin, PACKAGE_FEATURE_FLUTTER, PACKAGE_FEATURES_ANALYSIS_KEY, SymbolPlan,
    WorkspaceAnalysisBuilder,
};
use dust_plugin_derive::register_plugin;

use crate::{
    support::span,
    support::{interfaces_for_class, members_for_class},
    validate_support::{class, field, library, validate},
};

/// Library fixtures the validation tests build on.
#[path = "validate/fixtures.rs"]
mod fixtures;
use self::fixtures::*;

/// Flutter form helpers, and the name collisions they can cause.
#[path = "validate/helpers.rs"]
mod helpers;

/// Validation across nullable, nested, custom and class-level rules.
#[path = "validate/nested.rs"]
mod nested;

#[test]
fn emits_validate_for_string_number_and_matching_fields() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &validation_library(),
            &dust_plugin_api::PluginContext {
                symbol_plan: &flutter_symbol_plan(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
    let interfaces = interfaces_for_class(&contribution, "SignupRequest");
    let members = members_for_class(&contribution, "SignupRequest");

    assert_eq!(interfaces, &["Validatable".to_owned()]);
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

    let plan = flutter_symbol_plan();
    let diagnostics = plugin.validate_with_plan(&library, &plan);
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
