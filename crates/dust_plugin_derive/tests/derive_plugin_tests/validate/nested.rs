//! Validation across nullable, nested, custom and class-level rules.

use super::*;

#[test]
fn emits_nullable_nested_custom_and_class_validation() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &nested_library(),
            &dust_plugin_api::PluginContext {
                symbol_plan: &flutter_symbol_plan(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");
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
