import 'package:flutter/material.dart' hide Route;

import '../route.dart';
import 'page_scaffold.dart';

@Route(
  '/checkout/:plan',
  name: 'checkout',
  guards: [],
  transition: BottomToTopPageTransitionsBuilder(),
  fullscreenDialog: true,
)
class CheckoutPage extends StatelessWidget {
  const CheckoutPage({required this.plan, this.annual = false, super.key});

  final String plan;
  final bool annual;

  @override
  Widget build(BuildContext context) {
    return PageScaffold(
      title: 'Checkout',
      subtitle:
          'Public fullscreen dialog route. It demonstrates a path parameter plus a defaulted boolean query parameter.',
      badges: const [
        StatusPill('Public', icon: Icons.lock_open_outlined),
        StatusPill('Modal route', icon: Icons.open_in_full_outlined),
      ],
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          InfoCard(
            title: 'Selected plan',
            icon: Icons.shopping_bag_outlined,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Plan: $plan'),
                Text('Billing: ${annual ? 'annual' : 'monthly'}'),
                const SizedBox(height: 12),
                CodePill('/checkout/$plan?annual=$annual'),
              ],
            ),
          ),
          const SizedBox(height: 18),
          FilledButton.icon(
            onPressed: context.routes.pop,
            icon: const Icon(Icons.close),
            label: const Text('Close modal'),
          ),
        ],
      ),
    );
  }
}
