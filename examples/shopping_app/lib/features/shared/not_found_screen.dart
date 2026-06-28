import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart' hide Route;

import '../../route.dart';

@Route('/404', name: 'notFound', guards: [])
class NotFoundScreen extends StatelessWidget {
  const NotFoundScreen({this.path = '', super.key});

  final String path;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const TranslatedText(
          'shop_page_not_found',
          defaultText: 'Page not found',
        ),
      ),
      body: Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(Icons.travel_explore, size: 80, color: Colors.grey),
              const SizedBox(height: 16),
              TranslatedText(
                'shop_no_route_for_path',
                defaultText: 'No route for {path}',
                args: {'path': path},
                textAlign: TextAlign.center,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 24),
              FilledButton.icon(
                onPressed: () => context.navigator.products().go(),
                icon: const Icon(Icons.storefront),
                label: const TranslatedText(
                  'shop_back_to_shop',
                  defaultText: 'Back to shop',
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
