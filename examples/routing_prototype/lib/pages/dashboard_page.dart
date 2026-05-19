import 'package:flutter/material.dart' hide Route;

import '../layout/app_shell.dart';
import '../route.dart';
import 'page_scaffold.dart';

@Route(
  '/',
  name: 'dashboard',
  shell: AppShell,
  transition: FadeUpwardsPageTransitionsBuilder(),
)
class DashboardPage extends StatelessWidget {
  const DashboardPage({super.key});

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Dashboard',
      subtitle:
          'A production-shaped showcase for generated routes: path params, query params, guards, redirects, nested paths, modal routes, and browser URLs.',
      badges: const [
        StatusPill('Shell route', icon: Icons.layers_outlined),
        StatusPill('Typed navigation', icon: Icons.alt_route_outlined),
        StatusPill('Web deep links', icon: Icons.public_outlined),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          const InfoCard(
            title: 'What this prototype proves',
            icon: Icons.check_circle_outline,
            child: Text(
              'Navigator 2.0 route generated from @Route(). App pages stay in normal lib/pages files; route.g.dart owns imports, URL parsing, route data, guards, and typed navigation helpers.',
            ),
          ),
          const SizedBox(height: 18),
          Wrap(
            spacing: 14,
            runSpacing: 14,
            children: [
              RouteActionCard(
                title: 'Push a details page',
                path: '/posts/42',
                description:
                    'Path param id:int becomes a typed constructor field.',
                icon: Icons.article_outlined,
                onPressed: () => context.routes.postDetail(id: 42).push(),
              ),
              RouteActionCard(
                title: 'Replace search query',
                path: '/search?q=dust&page=2',
                description: 'Nullable and default params become query params.',
                icon: Icons.manage_search,
                onPressed: () => context.routes.search(q: 'dust', page: 2).go(),
              ),
              RouteActionCard(
                title: 'Try admin guard',
                path: '/admin',
                description: 'Route-level guard redirects non-admin users.',
                icon: Icons.admin_panel_settings_outlined,
                onPressed: () => context.routes.admin().go(),
              ),
              RouteActionCard(
                title: 'Open nested project',
                path: '/projects/712?tab=activity',
                description: 'Shell wrapping is inferred from route metadata.',
                icon: Icons.folder_open_outlined,
                onPressed: () => context.routes
                    .project(projectId: 712, tab: 'activity')
                    .go(),
              ),
              RouteActionCard(
                title: 'Open billing feature',
                path: '/billing/invoices/INV-2026-001?download=true',
                description: 'Feature guard protects a deeply nested route.',
                icon: Icons.receipt_long_outlined,
                onPressed: () => context.routes
                    .invoice(invoiceId: 'INV-2026-001', download: true)
                    .go(),
              ),
              RouteActionCard(
                title: 'Push checkout modal',
                path: '/checkout/team?annual=true',
                description:
                    'Public fullscreen dialog with a route transition builder.',
                icon: Icons.shopping_bag_outlined,
                onPressed: () =>
                    context.routes.checkout(plan: 'team', annual: true).push(),
              ),
            ],
          ),
          const SizedBox(height: 14),
          OutlinedButton.icon(
            onPressed: () => context.routes.notFound(path: 'bad-link').go(),
            icon: const Icon(Icons.warning_amber_outlined),
            label: const Text('Open not found'),
          ),
        ],
      ),
    );
  }
}
