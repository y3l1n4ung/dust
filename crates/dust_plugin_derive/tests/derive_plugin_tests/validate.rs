use dust_ir::{LibraryIr, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use crate::{
    support::members_for_class,
    validate_support::{class, field, library, validate},
};

#[test]
fn emits_validate_for_string_number_and_matching_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(&validation_library(), &SymbolPlan::default());
    let members = members_for_class(&contribution, "SignupRequest");

    assert_eq!(
        members,
        [r#"ValidationResult validate() {
  final self = this as SignupRequest;
  final errors = <ValidationError>[];
  _validateSignupRequestEmail(self.email, errors);
  _validateSignupRequestAge(self.age, errors);
  _validateSignupRequestPassword(self.password, errors);
  _validateSignupRequestConfirmPassword(self, self.confirmPassword, errors);
  return errors.isEmpty ? const Valid() : Invalid(errors);
}

void validateOrThrow() {
  final result = validate();
  if (result case Invalid(:final errors)) {
    throw ValidationException(errors);
  }
}"#
        .to_owned()]
        .as_slice()
    );

    assert_eq!(
        contribution.support_types,
        [
            r#"void _validateSignupRequestEmail(String email, List<ValidationError> errors) {
  if (!ValidationHelper.isEmail(email)) {
    errors.add(ValidationError(field: 'email', message: 'Invalid email'));
  }
}

String? validateSignupRequestEmailInput(String? value) {
  final errors = <ValidationError>[];
  _validateSignupRequestEmail(value ?? '', errors);
  return errors.isEmpty ? null : errors.first.message;
}

void _validateSignupRequestAge(int age, List<ValidationError> errors) {
  if (age < 18) {
    errors.add(ValidationError(field: 'age', message: 'Too small'));
  }
  if (age > 120) {
    errors.add(ValidationError(field: 'age', message: 'Too large'));
  }
}

String? validateSignupRequestAgeInput(String? value) {
  final errors = <ValidationError>[];
  final age = int.tryParse(value ?? '');
  if (age == null) {
    errors.add(ValidationError(field: 'age', message: 'Invalid number'));
  } else {
    _validateSignupRequestAge(age, errors);
  }
  return errors.isEmpty ? null : errors.first.message;
}

void _validateSignupRequestPassword(String password, List<ValidationError> errors) {
  if (password.length < 8) {
    errors.add(ValidationError(field: 'password', message: 'At least 8 characters'));
  }
  if (!RegExp('^(?=.*[A-Z]).+\$').hasMatch(password)) {
    errors.add(ValidationError(field: 'password', message: 'Need uppercase'));
  }
}

String? validateSignupRequestPasswordInput(String? value) {
  final errors = <ValidationError>[];
  _validateSignupRequestPassword(value ?? '', errors);
  return errors.isEmpty ? null : errors.first.message;
}

void _validateSignupRequestConfirmPassword(
  SignupRequest self,
  String confirmPassword,
  List<ValidationError> errors,
) {
  if (confirmPassword != self.password) {
    errors.add(ValidationError(field: 'confirmPassword', message: 'Fields do not match'));
  }
}

String? validateSignupRequestConfirmPasswordInput(
  SignupRequest self,
  String? value,
) {
  final errors = <ValidationError>[];
  _validateSignupRequestConfirmPassword(self, value ?? '', errors);
  return errors.isEmpty ? null : errors.first.message;
}"#
            .to_owned()
        ]
        .as_slice()
    );
}

#[test]
fn emits_nullable_nested_custom_and_class_validation() {
    let plugin = register_plugin();
    let contribution = plugin.emit(&nested_library(), &SymbolPlan::default());
    let members = members_for_class(&contribution, "Profile");

    assert_eq!(
        members,
        [r#"ValidationResult validate() {
  final self = this as Profile;
  final errors = <ValidationError>[];
  _validateProfileBio(self.bio, errors);
  _validateProfileAddress(self.address, errors);
  _validateProfilePhone(self.phone, errors);
  return errors.isEmpty ? const Valid() : Invalid(errors);
}

void validateOrThrow() {
  final result = validate();
  if (result case Invalid(:final errors)) {
    throw ValidationException(errors);
  }
}"#
        .to_owned()]
        .as_slice()
    );

    assert_eq!(
        contribution.support_types,
        [
            r#"void _validateAddressZip(String zip, List<ValidationError> errors) {
  if (zip.length != 5) {
    errors.add(ValidationError(field: 'zip', message: 'Invalid length'));
  }
}

String? validateAddressZipInput(String? value) {
  final errors = <ValidationError>[];
  _validateAddressZip(value ?? '', errors);
  return errors.isEmpty ? null : errors.first.message;
}"#
            .to_owned(),
            r#"void _validateProfileBio(String? bio, List<ValidationError> errors) {
  if (bio != null) {
    if (bio.length > 200) {
      errors.add(ValidationError(field: 'bio', message: 'Too long'));
    }
  }
}

String? validateProfileBioInput(String? value) {
  final errors = <ValidationError>[];
  _validateProfileBio(value, errors);
  return errors.isEmpty ? null : errors.first.message;
}

void _validateProfileAddress(Address address, List<ValidationError> errors) {
  final addressValidation = address.validate();
  if (addressValidation case Invalid(errors: final nestedErrors)) {
    for (final error in nestedErrors) {
      errors.add(ValidationError(field: 'address.${error.field}', message: error.message));
    }
  }
}

void _validateProfilePhone(String phone, List<ValidationError> errors) {
  final phoneCustomError = Profile.checkPhone(phone);
  if (phoneCustomError != null) {
    errors.add(phoneCustomError);
  }
}

String? validateProfilePhoneInput(String? value) {
  final errors = <ValidationError>[];
  _validateProfilePhone(value ?? '', errors);
  return errors.isEmpty ? null : errors.first.message;
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
