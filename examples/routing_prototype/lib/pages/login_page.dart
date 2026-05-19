import 'package:flutter/material.dart' hide Route;
import '../route.dart';

import '../app_state.dart';
import 'page_scaffold.dart';

@Route(
  '/login',
  name: 'login',
  guards: [],
  transition: CupertinoPageTransitionsBuilder(),
)
class LoginPage extends StatelessWidget {
  const LoginPage({this.from, super.key});

  final String? from;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Login',
      subtitle:
          'Public route. When a protected deep link is requested, redirect stores the safe local return URL in the from query parameter.',
      badges: const [
        StatusPill('Public', icon: Icons.lock_open_outlined),
        StatusPill('Redirect target', icon: Icons.turn_right_outlined),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InfoCard(
            title: 'Redirect state',
            icon: Icons.link_outlined,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Redirected from: ${from ?? 'direct visit'}'),
                const SizedBox(height: 12),
                CodePill(from ?? '/login'),
              ],
            ),
          ),
          const SizedBox(height: 18),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              FilledButton.icon(
                onPressed: () => appSession.logIn(),
                icon: const Icon(Icons.login_outlined),
                label: const Text('Log in'),
              ),
              FilledButton.tonalIcon(
                onPressed: () => appSession.logIn(admin: true),
                icon: const Icon(Icons.admin_panel_settings_outlined),
                label: const Text('Log in as admin'),
              ),
              OutlinedButton.icon(
                onPressed: () =>
                    context.routes.invite(token: 'acme-token').go(),
                icon: const Icon(Icons.mail_outline),
                label: const Text('Open public invite'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
