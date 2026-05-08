/// Form field state with validation
class FormFieldModel {
  final String value;
  final String? error;
  final bool touched;

  const FormFieldModel({
    this.value = '',
    this.error,
    this.touched = false,
  });

  FormFieldModel copyWith({String? value, String? error, bool? touched}) {
    return FormFieldModel(
      value: value ?? this.value,
      error: error,
      touched: touched ?? this.touched,
    );
  }

  bool get isValid => error == null && touched;
}

/// Registration form state
class RegistrationForm {
  final FormFieldModel name;
  final FormFieldModel email;
  final FormFieldModel password;
  final bool isSubmitting;
  final bool isSubmitted;

  const RegistrationForm({
    this.name = const FormFieldModel(),
    this.email = const FormFieldModel(),
    this.password = const FormFieldModel(),
    this.isSubmitting = false,
    this.isSubmitted = false,
  });

  RegistrationForm copyWith({
    FormFieldModel? name,
    FormFieldModel? email,
    FormFieldModel? password,
    bool? isSubmitting,
    bool? isSubmitted,
  }) {
    return RegistrationForm(
      name: name ?? this.name,
      email: email ?? this.email,
      password: password ?? this.password,
      isSubmitting: isSubmitting ?? this.isSubmitting,
      isSubmitted: isSubmitted ?? this.isSubmitted,
    );
  }

  bool get isValid => name.isValid && email.isValid && password.isValid;
}
