import 'package:ease_annotation/ease_annotation.dart';
import 'package:ease_state_helper/ease_state_helper.dart';

part 'local_form_view_model.ease.dart';

/// Example of a local provider that is NOT registered in the global Ease widget.
///
/// This is useful for:
/// - Form state that should be disposed when leaving a screen
/// - List item state (each item has its own instance)
/// - Dialog/modal specific state
/// - Feature-scoped state that shouldn't pollute the global tree
///
/// Usage:
/// ```dart
/// LocalFormViewModelProvider(
///   child: MyFormWidget(),
/// )
/// ```
@Ease(local: true)
class LocalFormViewModel extends StateNotifier<LocalFormState> {
  LocalFormViewModel() : super(const LocalFormState());

  void updateName(String name) {
    setState(state.copyWith(name: name), action: 'updateName');
  }

  void updateEmail(String email) {
    setState(state.copyWith(email: email), action: 'updateEmail');
  }

  void updatePhone(String phone) {
    setState(state.copyWith(phone: phone), action: 'updatePhone');
  }

  void setSubmitting(bool submitting) {
    setState(state.copyWith(isSubmitting: submitting), action: 'setSubmitting');
  }

  void reset() {
    setState(const LocalFormState(), action: 'reset');
  }

  bool get isValid =>
      state.name.isNotEmpty &&
      state.email.isNotEmpty &&
      state.email.contains('@');

  Future<void> submit() async {
    if (!isValid) return;

    setSubmitting(true);

    // Simulate API call
    await Future.delayed(const Duration(seconds: 1));

    setSubmitting(false);
    reset();
  }
}

class LocalFormState {
  final String name;
  final String email;
  final String phone;
  final bool isSubmitting;

  const LocalFormState({
    this.name = '',
    this.email = '',
    this.phone = '',
    this.isSubmitting = false,
  });

  LocalFormState copyWith({
    String? name,
    String? email,
    String? phone,
    bool? isSubmitting,
  }) {
    return LocalFormState(
      name: name ?? this.name,
      email: email ?? this.email,
      phone: phone ?? this.phone,
      isSubmitting: isSubmitting ?? this.isSubmitting,
    );
  }
}
