import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

import '../models/form_field.dart';

part 'form_view_model.ease.dart';

/// Registration form ViewModel - demonstrates complex form handling
@ease
class RegistrationFormViewModel extends StateNotifier<RegistrationForm> {
  RegistrationFormViewModel() : super(const RegistrationForm());

  // Field setters with validation
  void setName(String value) {
    String? error;
    if (value.isEmpty) {
      error = 'Name is required';
    } else if (value.length < 2) {
      error = 'Name must be at least 2 characters';
    }
    state = state.copyWith(
      name: FormFieldModel(value: value, error: error, touched: true),
    );
  }

  void setEmail(String value) {
    String? error;
    if (value.isEmpty) {
      error = 'Email is required';
    } else if (!_isValidEmail(value)) {
      error = 'Invalid email format';
    }
    state = state.copyWith(
      email: FormFieldModel(value: value, error: error, touched: true),
    );
  }

  void setPassword(String value) {
    String? error;
    if (value.isEmpty) {
      error = 'Password is required';
    } else if (value.length < 8) {
      error = 'Password must be at least 8 characters';
    } else if (!_hasUppercase(value)) {
      error = 'Password must contain uppercase letter';
    } else if (!_hasNumber(value)) {
      error = 'Password must contain a number';
    }
    state = state.copyWith(
      password: FormFieldModel(value: value, error: error, touched: true),
    );
  }

  Future<void> submit() async {
    // Touch all fields to show errors
    setName(state.name.value);
    setEmail(state.email.value);
    setPassword(state.password.value);

    if (!state.isValid) return;

    state = state.copyWith(isSubmitting: true);

    // Simulate API call
    await Future.delayed(const Duration(seconds: 2));

    state = state.copyWith(isSubmitting: false, isSubmitted: true);
  }

  void reset() {
    state = const RegistrationForm();
  }

  // Validation helpers
  bool _isValidEmail(String email) {
    return RegExp(r'^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$').hasMatch(email);
  }

  bool _hasUppercase(String value) {
    return RegExp(r'[A-Z]').hasMatch(value);
  }

  bool _hasNumber(String value) {
    return RegExp(r'[0-9]').hasMatch(value);
  }
}
