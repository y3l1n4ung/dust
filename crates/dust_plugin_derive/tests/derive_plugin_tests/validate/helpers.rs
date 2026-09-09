//! Flutter form helpers, and the name collisions they can cause.

use super::*;

#[test]
fn dart_validation_omits_flutter_form_helpers() {
    let plugin = register_plugin();
    let contribution = plugin
        .generate(
            &validation_library(),
            &dust_plugin_api::PluginContext {
                symbol_plan: &SymbolPlan::default(),
            },
        )
        .into_iter()
        .next()
        .expect("plugin must generate one contribution");

    assert_eq!(
        contribution.support_types,
        [r#"extension _SignupRequestValidation on SignupRequest {
  static void _validateEmail(String email, List<ValidationError> errors) {
    if (!ValidationHelper.isEmail(email)) {
      errors.add(ValidationError(field: 'email', message: 'Invalid email'));
    }
  }

  static void _validateAge(int age, List<ValidationError> errors) {
    if (age < 18) {
      errors.add(ValidationError(field: 'age', message: 'Too small'));
    }
    if (age > 120) {
      errors.add(ValidationError(field: 'age', message: 'Too large'));
    }
  }

  static void _validatePassword(String password, List<ValidationError> errors) {
    if (password.length < 8) {
      errors.add(ValidationError(field: 'password', message: 'At least 8 characters'));
    }
    if (!RegExp('^(?=.*[A-Z]).+\$').hasMatch(password)) {
      errors.add(ValidationError(field: 'password', message: 'Need uppercase'));
    }
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

}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn dart_validation_allows_form_helper_name_collisions() {
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

    assert!(diagnostics.is_empty());
}
