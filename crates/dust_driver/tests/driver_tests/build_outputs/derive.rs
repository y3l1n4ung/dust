use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{generated_output, make_workspace, write_file};

#[test]
fn build_writes_real_outputs_for_multiple_libraries_and_classes() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/models.dart"),
        "part 'models.g.dart';\n\
         @Derive([ToString(), Eq(), CopyWith()])\n\
         class User {\n\
           final String id;\n\
           final int? age;\n\
           const User(this.id, this.age);\n\
         }\n\
         @CopyWith()\n\
         class Team {\n\
           final String name;\n\
           const Team(this.name);\n\
         }\n",
    );
    write_file(
        &workspace.path().join("lib/request.dart"),
        "part 'request.g.dart';\n\
         @CopyWith()\n\
         class Request {\n\
           final String path;\n\
           final Map<String, String> headers;\n\
           const Request.create({required this.path, required this.headers});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    let models_output = fs::read_to_string(workspace.path().join("lib/models.g.dart")).unwrap();
    let request_output = fs::read_to_string(workspace.path().join("lib/request.g.dart")).unwrap();

    assert!(!result.has_errors());
    assert_eq!(result.build_artifacts.len(), 2);
    assert_eq!(result.cache.as_ref().unwrap().misses, 2);
    assert_eq!(result.cache.as_ref().unwrap().hits, 0);
    assert!(
        result
            .build_artifacts
            .iter()
            .all(|artifact| artifact.written)
    );
    assert_eq!(
        models_output,
        generated_output(
            r#"part of 'models.dart';

final class _UserCopyWithUnset {
  const _UserCopyWithUnset();
}

const _userCopyWithUnset = _UserCopyWithUnset();

mixin _$User {
  @override
  String toString() {
    final self = this as User;
    return 'User('
        'id: ${self.id}, '
        'age: ${self.age}'
        ')';
  }

  @override
  bool operator ==(Object other) {
    final self = this as User;
    return identical(this, other) ||
        other is User &&
            runtimeType == other.runtimeType &&
            other.id == self.id &&
            other.age == self.age;
  }

  @override
  int get hashCode {
    final self = this as User;
    return Object.hashAll([
      runtimeType,
      self.id,
      self.age,
    ]);
  }

  /// Creates a copy of this `User` with selected fields replaced.
  ///
  /// Usage:
  /// ```dart
  /// final updated = user.copyWith(id: 'John');
  /// final cleared = user.copyWith(age: null);
  /// ```
  @pragma('vm:prefer-inline')
  _$UserCopyWith<User> get copyWith => _$UserCopyWithImpl<User>(this as User, (value) => value);
}

mixin _$Team {
  /// Creates a copy of this `Team` with selected fields replaced.
  ///
  /// Usage:
  /// ```dart
  /// final updated = team.copyWith(name: 'John');
  /// ```
  @pragma('vm:prefer-inline')
  _$TeamCopyWith<Team> get copyWith => _$TeamCopyWithImpl<Team>(this as Team, (value) => value);
}

// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$UserCopyWith<$Res> {
  $Res call({
    String? id,
    int? age,
  });
}

/// @nodoc
final class _$UserCopyWithImpl<$Res> implements _$UserCopyWith<$Res> {
  const _$UserCopyWithImpl(this._self, this._then);

  final User _self;
  final $Res Function(User) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? id = null,
    Object? age = _userCopyWithUnset,
  }) {
    return _then(
      User(
        id == null ? _self.id : id as String,
        identical(age, _userCopyWithUnset)
            ? _self.age
            : age as int?,
      )
    );
  }
}
/// @nodoc
abstract class _$TeamCopyWith<$Res> {
  $Res call({
    String? name,
  });
}

/// @nodoc
final class _$TeamCopyWithImpl<$Res> implements _$TeamCopyWith<$Res> {
  const _$TeamCopyWithImpl(this._self, this._then);

  final Team _self;
  final $Res Function(Team) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? name = null,
  }) {
    return _then(
      Team(
        name == null ? _self.name : name as String,
      )
    );
  }
}
"#
        )
    );
    assert_eq!(
        request_output,
        generated_output(
            r#"part of 'request.dart';

mixin _$Request {
  /// Creates a copy of this `Request` with selected fields replaced.
  ///
  /// Usage:
  /// ```dart
  /// final updated = request.copyWith(path: 'John');
  /// ```
  @pragma('vm:prefer-inline')
  _$RequestCopyWith<Request> get copyWith => _$RequestCopyWithImpl<Request>(this as Request, (value) => value);
}

// CopyWith API inspired by Freezed.

/// @nodoc
abstract class _$RequestCopyWith<$Res> {
  $Res call({
    String? path,
    Map<String, String>? headers,
  });
}

/// @nodoc
final class _$RequestCopyWithImpl<$Res> implements _$RequestCopyWith<$Res> {
  const _$RequestCopyWithImpl(this._self, this._then);

  final Request _self;
  final $Res Function(Request) _then;

  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? path = null,
    Object? headers = null,
  }) {
    return _then(
      Request.create(
        path: path == null ? _self.path : path as String,
        headers: headers == null ? _self.headers : headers as Map<String, String>,
      )
    );
  }
}
"#
        )
    );
}

#[test]
fn build_writes_validate_output_for_form_request() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/signup.dart"),
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

mixin _$SignupRequest {
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

/// TextFormField validator for `SignupRequest.email`.
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
      errors.add(ValidationError(field: 'age', message: 'Adult only'));
    }
    if (age > 120) {
      errors.add(ValidationError(field: 'age', message: 'Adult only'));
    }
  }

  static String? validateAgeInput(String? value) {
    final errors = <ValidationError>[];
    final age = int.tryParse(value ?? '');
    if (age == null) {
      errors.add(ValidationError(field: 'age', message: 'Adult only'));
    } else {
      _validateAge(age, errors);
    }
    return errors.isEmpty ? null : errors.first.message;
  }

}
"#
        )
    );
}
