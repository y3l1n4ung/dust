import 'package:flutter/material.dart' hide Route;
import '../route.dart';

import '../app_state.dart';
import 'page_scaffold.dart';

@Route('/invite/:token', name: 'invite', guards: [])
class InvitePage extends StatelessWidget {
  const InvitePage({required this.token, super.key});

  final String token;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Invite',
      subtitle: 'A public deep link that accepts a token path parameter.',
      badges: const [StatusPill('Public deep link', icon: Icons.mail_outline)],
      body: InfoCard(
        title: 'Invite payload',
        icon: Icons.key_outlined,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Public invite token: $token'),
            const SizedBox(height: 12),
            CodePill('/invite/$token'),
            const SizedBox(height: 18),
            FilledButton.icon(
              onPressed: () => appSession.logIn(),
              icon: const Icon(Icons.check_circle_outline),
              label: const Text('Accept invite and log in'),
            ),
          ],
        ),
      ),
    );
  }
}
