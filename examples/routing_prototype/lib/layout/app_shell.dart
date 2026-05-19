import 'package:flutter/material.dart';

import '../app_state.dart';
import '../route.dart';
import '../pages/page_scaffold.dart';

class AppShell extends StatelessWidget {
  const AppShell({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    final current = DustRouter.of<AppRoutePath>(context).currentRoute;

    return Scaffold(
      appBar: AppBar(
        title: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Container(
              width: 34,
              height: 34,
              decoration: BoxDecoration(
                color: Theme.of(context).colorScheme.primary,
                borderRadius: BorderRadius.circular(12),
              ),
              child: const Icon(Icons.auto_awesome, color: Colors.white),
            ),
            const SizedBox(width: 12),
            const Text('Dust Router Lab'),
          ],
        ),
        actions: [
          Padding(
            padding: const EdgeInsets.only(right: 12),
            child: ListenableBuilder(
              listenable: appSession,
              builder: (context, _) => Wrap(
                spacing: 8,
                crossAxisAlignment: WrapCrossAlignment.center,
                children: [
                  StatusPill(
                    appSession.isLoggedIn ? 'Signed in' : 'Guest',
                    icon: appSession.isLoggedIn
                        ? Icons.verified_user_outlined
                        : Icons.person_outline,
                    active: appSession.isLoggedIn,
                  ),
                  TextButton.icon(
                    onPressed: appSession.isLoggedIn
                        ? appSession.logOut
                        : () => appSession.logIn(),
                    icon: Icon(
                      appSession.isLoggedIn
                          ? Icons.logout_outlined
                          : Icons.login_outlined,
                    ),
                    label: Text(appSession.isLoggedIn ? 'Log out' : 'Log in'),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final nav = _ShellNavigation(current: current);
          if (constraints.maxWidth < 760) {
            return Column(
              children: [
                Material(
                  color: Colors.white.withValues(alpha: 0.76),
                  child: SingleChildScrollView(
                    scrollDirection: Axis.horizontal,
                    padding: const EdgeInsets.symmetric(
                      horizontal: 12,
                      vertical: 8,
                    ),
                    child: nav.compact(context),
                  ),
                ),
                Expanded(child: child),
              ],
            );
          }
          return Row(
            children: [
              SizedBox(width: 260, child: nav.rail(context)),
              const VerticalDivider(width: 1),
              Expanded(child: child),
            ],
          );
        },
      ),
    );
  }
}

class _ShellNavigation {
  const _ShellNavigation({required this.current});

  final AppRoutePath current;

  int? get selectedIndex => switch (current) {
    DashboardRoute() => 0,
    SearchRoute() => 1,
    ProjectRoute() || ProjectSettingsRoute() => 2,
    AdminRoute() => 3,
    InvoiceRoute() => 4,
    _ => null,
  };

  void select(BuildContext context, int index) {
    switch (index) {
      case 0:
        context.routes.dashboard().go();
      case 1:
        context.routes.search(q: 'router', page: 1).go();
      case 2:
        context.routes.project(projectId: 712, tab: 'activity').go();
      case 3:
        context.routes.admin().go();
      case 4:
        context.routes.invoice(invoiceId: 'INV-2026-001', download: true).go();
    }
  }

  Widget rail(BuildContext context) {
    return DecoratedBox(
      decoration: const BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [Color(0xFFFFFFFF), Color(0xFFF0E8D8)],
        ),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const Padding(
            padding: EdgeInsets.fromLTRB(20, 20, 20, 8),
            child: Text(
              'Route examples',
              style: TextStyle(fontWeight: FontWeight.w900, fontSize: 18),
            ),
          ),
          Expanded(
            child: NavigationRail(
              extended: true,
              selectedIndex: selectedIndex,
              onDestinationSelected: (index) => select(context, index),
              backgroundColor: Colors.transparent,
              labelType: NavigationRailLabelType.none,
              destinations: _destinations,
            ),
          ),
          const Padding(
            padding: EdgeInsets.all(16),
            child: CodePill('Navigator 2.0 + generated typed routes'),
          ),
        ],
      ),
    );
  }

  Widget compact(BuildContext context) {
    return SegmentedButton<int>(
      showSelectedIcon: false,
      segments: const [
        ButtonSegment(value: 0, icon: Icon(Icons.dashboard_outlined)),
        ButtonSegment(value: 1, icon: Icon(Icons.search)),
        ButtonSegment(value: 2, icon: Icon(Icons.folder_open_outlined)),
        ButtonSegment(
          value: 3,
          icon: Icon(Icons.admin_panel_settings_outlined),
        ),
        ButtonSegment(value: 4, icon: Icon(Icons.receipt_long_outlined)),
      ],
      selected: {?selectedIndex},
      onSelectionChanged: (selection) => select(context, selection.single),
    );
  }

  static const _destinations = [
    NavigationRailDestination(
      icon: Icon(Icons.dashboard_outlined),
      selectedIcon: Icon(Icons.dashboard),
      label: Text('Dashboard'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.search),
      selectedIcon: Icon(Icons.manage_search),
      label: Text('Search query'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.folder_open_outlined),
      selectedIcon: Icon(Icons.folder),
      label: Text('Nested project'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.admin_panel_settings_outlined),
      selectedIcon: Icon(Icons.admin_panel_settings),
      label: Text('Admin guard'),
    ),
    NavigationRailDestination(
      icon: Icon(Icons.receipt_long_outlined),
      selectedIcon: Icon(Icons.receipt_long),
      label: Text('Billing guard'),
    ),
  ];
}
