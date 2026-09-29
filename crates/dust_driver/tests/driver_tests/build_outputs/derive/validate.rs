//! Generated validate output, with and without Flutter form helpers.

use super::*;

#[test]
fn build_writes_dart_validate_output_without_form_helpers() {
    let workspace = make_workspace();
    write_signup_request(workspace.path());

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let output = fs::read_to_string(workspace.path().join("lib/signup.g.dart")).unwrap();

    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'signup.dart';

mixin _$SignupRequest implements Validatable {
  /// Validates this `SignupRequest`.
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
  }
}

extension _SignupRequestValidation on SignupRequest {
  static void _validateEmail(String email, List<ValidationError> errors) {
    if (!ValidationHelper.isEmail(email)) {
      errors.add(ValidationError(field: 'email', message: 'Invalid email'));
    }
  }

  static void _validateAge(int age, List<ValidationError> errors) {
    if (age < 18) {
      errors.add(ValidationError(field: 'age', message: 'Adult only'));
    }
    if (age > 120) {
      errors.add(ValidationError(field: 'age', message: 'Adult only'));
    }
  }

}
"#
        )
    );
}

#[test]
fn build_writes_flutter_form_helpers_for_flutter_packages() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("pubspec.yaml"),
        "name: dust_test\ndependencies:\n  flutter:\n    sdk: flutter\n",
    );
    write_signup_request(workspace.path());

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let output = fs::read_to_string(workspace.path().join("lib/signup.g.dart")).unwrap();

    assert_eq!(result.diagnostics, vec![]);
    assert!(output.contains("/// TextFormField validator for `SignupRequest.email`."));
    assert!(output.contains("String? validateSignupRequestEmailInput(String? value)"));
    assert!(output.contains("static String? validateEmailInput(String? value)"));
}

#[test]
fn build_resolves_validate_string_constants_from_the_same_library() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/comment.dart"),
        &[DustImport::Derive],
        "part 'comment.g.dart';\n\
         const notBlank = r'\\S';\n\
         const blankMessage = \"can't be blank\";\n\
         @Derive([Validate()])\n\
         class NewComment with _$NewComment {\n\
           const NewComment({required this.body});\n\
           @Validate(regex: notBlank, message: blankMessage)\n\
           final String body;\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });
    let output = fs::read_to_string(workspace.path().join("lib/comment.g.dart")).unwrap();

    assert_eq!(result.diagnostics, vec![]);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'comment.dart';

mixin _$NewComment implements Validatable {
  /// Validates this `NewComment`.
  ///
  /// Usage:
  /// ```dart
  /// final result = value.validate();
  /// if (result case Invalid(:final errors)) {
  ///   print(errors.first.message);
  /// }
  /// ```
  ValidationResult validate() {
    final self = this as NewComment;
    final errors = <ValidationError>[];
    _NewCommentValidation._validateBody(self.body, errors);
    return errors.isEmpty ? const Valid() : Invalid(errors);
  }

  /// Throws [ValidationException] when this `NewComment` is invalid.
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
  }
}

extension _NewCommentValidation on NewComment {
  static void _validateBody(String body, List<ValidationError> errors) {
    if (!RegExp('\\S').hasMatch(body)) {
      errors.add(ValidationError(field: 'body', message: 'can\'t be blank'));
    }
  }

}
"#,
        )
    );
}

fn write_signup_request(root: &std::path::Path) {
    write_dust_file(
        &root.join("lib/signup.dart"),
        &[DustImport::Derive],
        "part 'signup.g.dart';\n\
         @Derive([Validate()])\n\
         class SignupRequest with _$SignupRequest {\n\
           const SignupRequest({required this.email, required this.age});\n\
           @Validate(email: true)\n\
           final String email;\n\
           @Validate(range: Range(min: 18, max: 120), message: 'Adult only')\n\
           final int age;\n\
         }\n",
    );
}
