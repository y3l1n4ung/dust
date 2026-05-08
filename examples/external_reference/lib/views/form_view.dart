import 'package:flutter/material.dart';

import '../view_models/form_view_model.dart';

/// Form View - demonstrates complex form validation
class FormView extends StatelessWidget {
  const FormView({super.key});

  @override
  Widget build(BuildContext context) {
    final form = context.registrationFormViewModel;

    if (form.state.isSubmitted) {
      return _SuccessScreen(onReset: form.reset);
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Registration Form'),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'Create Account',
              style: TextStyle(
                fontSize: 28,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Fill in your details to register',
              style: TextStyle(
                fontSize: 16,
                color: Colors.grey[600],
              ),
            ),
            const SizedBox(height: 32),

            // Name field
            _FormField(
              label: 'Full Name',
              value: form.state.name.value,
              error: form.state.name.touched ? form.state.name.error : null,
              onChanged: form.setName,
              prefixIcon: Icons.person_outline,
            ),
            const SizedBox(height: 16),

            // Email field
            _FormField(
              label: 'Email',
              value: form.state.email.value,
              error: form.state.email.touched ? form.state.email.error : null,
              onChanged: form.setEmail,
              prefixIcon: Icons.email_outlined,
              keyboardType: TextInputType.emailAddress,
            ),
            const SizedBox(height: 16),

            // Password field
            _FormField(
              label: 'Password',
              value: form.state.password.value,
              error: form.state.password.touched
                  ? form.state.password.error
                  : null,
              onChanged: form.setPassword,
              prefixIcon: Icons.lock_outline,
              obscureText: true,
            ),
            const SizedBox(height: 8),

            // Password requirements
            _PasswordRequirements(password: form.state.password.value),
            const SizedBox(height: 32),

            // Submit button
            SizedBox(
              height: 56,
              child: ElevatedButton(
                onPressed: form.state.isSubmitting ? null : form.submit,
                child: form.state.isSubmitting
                    ? const SizedBox(
                        width: 24,
                        height: 24,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          color: Colors.white,
                        ),
                      )
                    : const Text(
                        'Create Account',
                        style: TextStyle(fontSize: 18),
                      ),
              ),
            ),
            const SizedBox(height: 16),

            // Reset button
            TextButton(
              onPressed: form.reset,
              child: const Text('Reset Form'),
            ),
          ],
        ),
      ),
    );
  }
}

class _FormField extends StatelessWidget {
  final String label;
  final String value;
  final String? error;
  final ValueChanged<String> onChanged;
  final IconData prefixIcon;
  final bool obscureText;
  final TextInputType? keyboardType;

  const _FormField({
    required this.label,
    required this.value,
    required this.error,
    required this.onChanged,
    required this.prefixIcon,
    this.obscureText = false,
    this.keyboardType,
  });

  @override
  Widget build(BuildContext context) {
    return TextFormField(
      initialValue: value,
      onChanged: onChanged,
      obscureText: obscureText,
      keyboardType: keyboardType,
      decoration: InputDecoration(
        labelText: label,
        prefixIcon: Icon(prefixIcon),
        errorText: error,
        border: const OutlineInputBorder(),
      ),
    );
  }
}

class _PasswordRequirements extends StatelessWidget {
  final String password;

  const _PasswordRequirements({required this.password});

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _RequirementRow(
          text: 'At least 8 characters',
          met: password.length >= 8,
        ),
        _RequirementRow(
          text: 'Contains uppercase letter',
          met: RegExp(r'[A-Z]').hasMatch(password),
        ),
        _RequirementRow(
          text: 'Contains a number',
          met: RegExp(r'[0-9]').hasMatch(password),
        ),
      ],
    );
  }
}

class _RequirementRow extends StatelessWidget {
  final String text;
  final bool met;

  const _RequirementRow({required this.text, required this.met});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          Icon(
            met ? Icons.check_circle : Icons.circle_outlined,
            size: 16,
            color: met ? Colors.green : Colors.grey,
          ),
          const SizedBox(width: 8),
          Text(
            text,
            style: TextStyle(
              fontSize: 12,
              color: met ? Colors.green : Colors.grey,
            ),
          ),
        ],
      ),
    );
  }
}

class _SuccessScreen extends StatelessWidget {
  final VoidCallback onReset;

  const _SuccessScreen({required this.onReset});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(
              Icons.check_circle,
              size: 100,
              color: Colors.green,
            ),
            const SizedBox(height: 24),
            const Text(
              'Registration Successful!',
              style: TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Your account has been created.',
              style: TextStyle(
                fontSize: 16,
                color: Colors.grey[600],
              ),
            ),
            const SizedBox(height: 32),
            ElevatedButton(
              onPressed: onReset,
              child: const Text('Register Another'),
            ),
          ],
        ),
      ),
    );
  }
}
