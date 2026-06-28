import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';
import '../models/demo_cart_state.dart';
import '../view_models/demo_cart_api_view_model.dart';

@Route('/demo-carts', name: 'demoCarts', guards: [])
class DemoCartsScreen extends StatelessWidget {
  const DemoCartsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final state = context.watchDemoCartApiViewModel().value;

    return Scaffold(
      appBar: AppBar(
        title: const TranslatedText(
          'shop_remote_carts',
          defaultText: 'FakeStore Carts',
        ),
      ),
      body: switch (state.status) {
        DemoCartStatus.initial || DemoCartStatus.loading => const Center(
            child: CircularProgressIndicator(),
          ),
        DemoCartStatus.error => Center(
            child: Padding(
              padding: const EdgeInsets.all(24),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.cloud_off, size: 64, color: Colors.red),
                  const SizedBox(height: 16),
                  Text(
                    state.errorMessage ??
                        context.tr(
                          'shop_load_carts_failed',
                          defaultText: 'Failed to load carts',
                        ),
                  ),
                  const SizedBox(height: 16),
                  FilledButton.icon(
                    onPressed: () =>
                        context.readDemoCartApiViewModel().loadUserCarts(1),
                    icon: const Icon(Icons.refresh),
                    label: const TranslatedText(
                      'shop_retry',
                      defaultText: 'Retry',
                    ),
                  ),
                ],
              ),
            ),
          ),
        DemoCartStatus.success => ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Card(
                color: Theme.of(context).colorScheme.primaryContainer,
                child: const Padding(
                  padding: EdgeInsets.all(16),
                  child: TranslatedText(
                    'shop_remote_carts_note',
                    defaultText:
                        "This page uses new generated FakeStore cart endpoints. It does not replace the app's local cart flow.",
                  ),
                ),
              ),
              const SizedBox(height: 12),
              ...state.carts.map(
                (cart) => Card(
                  child: ListTile(
                    leading: CircleAvatar(child: Text('${cart.id}')),
                    title: TranslatedText(
                      'shop_remote_cart_number',
                      defaultText: 'Remote cart #{id}',
                      args: {'id': cart.id},
                    ),
                    subtitle: TranslatedText(
                      'shop_remote_cart_summary',
                      defaultText:
                          'User {userId} - {itemCount} item(s) - {rowCount} product rows',
                      args: {
                        'userId': cart.userId,
                        'itemCount': cart.itemCount,
                        'rowCount': cart.products.length,
                      },
                    ),
                    trailing: const Icon(Icons.chevron_right),
                  ),
                ),
              ),
            ],
          ),
      },
    );
  }
}
