import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';

@AppRoute(
  '/order-confirmation/:orderId',
  name: 'orderConfirmation',
  guards: [],
  transition: ZoomPageTransitionsBuilder(),
)
class OrderConfirmationScreen extends StatelessWidget {
  final String orderId;

  const OrderConfirmationScreen({super.key, required this.orderId});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.check_circle, size: 100, color: Colors.green),
              const SizedBox(height: 24),
              TranslatedText(
                'shop_order_placed',
                defaultText: 'Order Placed!',
                style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
              ),
              const SizedBox(height: 8),
              TranslatedText(
                'shop_purchase_thanks',
                defaultText: 'Thank you for your purchase',
                style: Theme.of(
                  context,
                ).textTheme.bodyLarge?.copyWith(color: Colors.grey),
              ),
              const SizedBox(height: 24),
              Container(
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: Colors.grey.shade100,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  children: [
                    const TranslatedText(
                      'shop_order_id',
                      defaultText: 'Order ID',
                    ),
                    const SizedBox(height: 4),
                    Text(
                      '#${orderId.substring(orderId.length - 6)}',
                      style: Theme.of(context).textTheme.titleLarge?.copyWith(
                            fontWeight: FontWeight.bold,
                          ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 48),
              SizedBox(
                width: double.infinity,
                child: FilledButton(
                  onPressed: () =>
                      context.navigator.orderDetail(orderId: orderId).go(),
                  child: const TranslatedText(
                    'shop_track_order',
                    defaultText: 'Track Order',
                  ),
                ),
              ),
              const SizedBox(height: 12),
              SizedBox(
                width: double.infinity,
                child: OutlinedButton(
                  onPressed: () => context.navigator.orders().go(),
                  child: const TranslatedText(
                    'shop_view_orders',
                    defaultText: 'View Orders',
                  ),
                ),
              ),
              const SizedBox(height: 12),
              SizedBox(
                width: double.infinity,
                child: OutlinedButton(
                  onPressed: () => context.navigator.products().go(),
                  child: const TranslatedText(
                    'shop_continue_shopping',
                    defaultText: 'Continue Shopping',
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
