import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../models/auth_state.dart';

class RegisterAuthError extends StatelessWidget {
  const RegisterAuthError({required this.state, super.key});

  final AuthState state;

  @override
  Widget build(BuildContext context) {
    if (state.status != AuthStatus.error) {
      return const SizedBox.shrink();
    }

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.red.withAlpha(25),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          const Icon(Icons.error_outline, color: Colors.red),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              state.errorMessage ??
                  context.tr(
                    'shop_registration_failed',
                    defaultText: 'Registration failed',
                  ),
              style: const TextStyle(color: Colors.red),
            ),
          ),
        ],
      ),
    );
  }
}
