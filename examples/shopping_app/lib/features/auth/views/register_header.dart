import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

class RegisterHeader extends StatelessWidget {
  const RegisterHeader({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        const Icon(Icons.person_add, size: 60, color: Colors.deepPurple),
        const SizedBox(height: 16),
        TranslatedText(
          'shop_join_us',
          defaultText: 'Join Us',
          style: Theme.of(
            context,
          ).textTheme.headlineMedium?.copyWith(fontWeight: FontWeight.bold),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        TranslatedText(
          'shop_create_account_subtitle',
          defaultText: 'Create an account to start shopping',
          style: Theme.of(
            context,
          ).textTheme.bodyMedium?.copyWith(color: Colors.grey),
          textAlign: TextAlign.center,
        ),
      ],
    );
  }
}
