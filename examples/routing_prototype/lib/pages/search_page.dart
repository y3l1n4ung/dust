import 'package:flutter/material.dart' hide Route;

import '../layout/app_shell.dart';
import '../route.dart';
import 'page_scaffold.dart';

@Route('/search', name: 'search', shell: AppShell)
final class SearchPage extends StatelessWidget {
  const SearchPage({this.q, this.page = 1, super.key});

  final String? q;
  final int page;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Search',
      subtitle:
          'Query-only route. Defaults are omitted from generated URLs until they differ from the constructor default.',
      badges: const [
        StatusPill('Query params', icon: Icons.manage_search),
        StatusPill('replace()', icon: Icons.swap_horiz_outlined),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InfoCard(
            title: 'Search state',
            icon: Icons.search,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Query: ${q ?? '(empty)'}'),
                Text('Page: $page'),
                const SizedBox(height: 12),
                CodePill('/search?q=${q ?? ''}&page=$page'),
              ],
            ),
          ),
          const SizedBox(height: 18),
          Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              FilledButton.tonal(
                onPressed: () =>
                    context.routes.search(q: 'next', page: page + 1).replace(),
                child: const Text('Replace with next page'),
              ),
              OutlinedButton(
                onPressed: () => context.routes.search().go(),
                child: const Text('Clear query'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
