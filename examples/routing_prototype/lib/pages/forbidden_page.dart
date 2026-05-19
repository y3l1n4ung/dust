import 'package:flutter/material.dart' hide Route;
import '../route.dart';

import '../app_state.dart';
import 'page_scaffold.dart';

@Route(
  '/forbidden',
  name: 'forbidden',
  guards: [],
  transition: FadeUpwardsPageTransitionsBuilder(),
)
class ForbiddenPage extends StatelessWidget {
  const ForbiddenPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Forbidden',
      subtitle:
          'The guard redirected here instead of rendering the protected page.',
      badges: const [
        StatusPill('Public fallback', icon: Icons.shield_outlined),
      ],
      body: InfoCard(
        title: 'Recovery actions',
        icon: Icons.restart_alt_outlined,
        child: Wrap(
          spacing: 12,
          runSpacing: 12,
          children: [
            FilledButton.icon(
              onPressed: () => appSession.logIn(admin: true),
              icon: const Icon(Icons.admin_panel_settings_outlined),
              label: const Text('Become admin'),
            ),
            OutlinedButton.icon(
              onPressed: () => context.routes.dashboard().go(),
              icon: const Icon(Icons.dashboard_outlined),
              label: const Text('Back to dashboard'),
            ),
          ],
        ),
      ),
    );
  }
}
