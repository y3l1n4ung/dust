import 'package:flutter/material.dart' hide Route;
import '../route.dart';

import 'page_scaffold.dart';

@Route(
  '/404/:path',
  name: 'notFound',
  guards: [],
  transition: FadeUpwardsPageTransitionsBuilder(),
)
class NotFoundPage extends StatelessWidget {
  const NotFoundPage({required this.path, super.key});

  final String path;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Not found',
      subtitle:
          'Unknown URLs are encoded into one safe query value so multi-segment bad links round-trip correctly.',
      badges: const [
        StatusPill('404', icon: Icons.warning_amber_outlined, active: false),
      ],
      body: InfoCard(
        title: 'Unmatched URL',
        icon: Icons.link_off_outlined,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('No route matched "$path".'),
            const SizedBox(height: 12),
            CodePill(path),
            const SizedBox(height: 18),
            FilledButton.icon(
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
