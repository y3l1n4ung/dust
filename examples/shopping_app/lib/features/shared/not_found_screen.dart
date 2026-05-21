import 'package:flutter/material.dart' hide Route;

import '../../route.dart';

@Route('/404/:path', name: 'notFound', guards: [])
class NotFoundScreen extends StatelessWidget {
  const NotFoundScreen({required this.path, super.key});

  final String path;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Page not found')),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.travel_explore, size: 80, color: Colors.grey),
              const SizedBox(height: 16),
              Text(
                'No route for $path',
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 24),
              FilledButton.icon(
                onPressed: () => context.routes.products().go(),
                icon: const Icon(Icons.storefront),
                label: const Text('Back to shop'),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
