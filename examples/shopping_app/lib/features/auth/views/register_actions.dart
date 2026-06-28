import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';
import '../models/auth_state.dart';

class RegisterActions extends StatelessWidget {
  const RegisterActions({
    required this.status,
    required this.redirectPath,
    required this.onRegister,
    super.key,
  });

  final AuthStatus status;
  final String? redirectPath;
  final VoidCallback onRegister;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FilledButton(
          onPressed: status == AuthStatus.loading ? null : onRegister,
          style: FilledButton.styleFrom(
            padding: const EdgeInsets.symmetric(vertical: 16),
          ),
          child: status == AuthStatus.loading
              ? const SizedBox(
                  height: 20,
                  width: 20,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    color: Colors.white,
                  ),
                )
              : const TranslatedText(
                  'shop_create_account',
                  defaultText: 'Create Account',
                ),
        ),
        const SizedBox(height: 16),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const TranslatedText(
              'shop_have_account',
              defaultText: 'Already have an account?',
            ),
            TextButton(
              onPressed: () {
                if (redirectPath != null) {
                  context.navigator.login(redirectPath: redirectPath).go();
                } else {
                  context.navigator.login().go();
                }
              },
              child:
                  const TranslatedText('shop_sign_in', defaultText: 'Sign In'),
            ),
          ],
        ),
      ],
    );
  }
}
