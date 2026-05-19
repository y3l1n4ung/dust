import 'package:flutter/material.dart' hide Route;

import '../app_state.dart';
import '../layout/app_shell.dart';
import '../route.dart';
import 'page_scaffold.dart';

@Route('/admin', name: 'admin', shell: AppShell, guards: [AdminGuard])
class AdminPage extends StatelessWidget {
  const AdminPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Admin',
      subtitle:
          'A protected route with a route-level async guard. Toggle the session flag to test reactive guard refresh.',
      badges: const [
        StatusPill('Requires auth', icon: Icons.lock_outline),
        StatusPill('AdminGuard', icon: Icons.policy_outlined),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const InfoCard(
            title: 'Guard behavior',
            icon: Icons.security_outlined,
            child: Text(
              'Async guard allows this page only for admin sessions.',
            ),
          ),
          const SizedBox(height: 18),
          InfoCard(
            title: 'Session controls',
            icon: Icons.tune_outlined,
            child: ListenableBuilder(
              listenable: appSession,
              builder: (context, _) => SwitchListTile(
                title: const Text('Admin flag'),
                subtitle: Text(
                  appSession.isAdmin
                      ? 'Guard passes'
                      : 'Guard redirects to /forbidden',
                ),
                value: appSession.isAdmin,
                onChanged: (_) => appSession.toggleAdmin(),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
