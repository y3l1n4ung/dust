import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';

import '../models/order.dart';
import '../view_models/orders_view_model.dart';

@Route('/orders', name: 'orders')
class OrdersScreen extends StatelessWidget {
  const OrdersScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final ordersState = context.watchOrdersViewModel().value;

    return Scaffold(
      appBar: AppBar(
        title: const TranslatedText('shop_my_orders', defaultText: 'My Orders'),
      ),
      body: ordersState.orders.isEmpty
          ? const Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Icon(
                    Icons.receipt_long_outlined,
                    size: 80,
                    color: Colors.grey,
                  ),
                  SizedBox(height: 16),
                  TranslatedText(
                    'shop_no_orders',
                    defaultText: 'No orders yet',
                  ),
                ],
              ),
            )
          : GridView.builder(
              padding: const EdgeInsets.all(16),
              gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                maxCrossAxisExtent: 450,
                mainAxisExtent: 180,
                crossAxisSpacing: 16,
                mainAxisSpacing: 16,
              ),
              itemCount: ordersState.orders.length,
              itemBuilder: (context, index) {
                final order = ordersState.orders[index];
                return _OrderCard(order: order);
              },
            ),
    );
  }
}

class _OrderCard extends StatelessWidget {
  final Order order;

  const _OrderCard({required this.order});

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: EdgeInsets.zero,
      child: InkWell(
        onTap: () => context.navigator.orderDetail(orderId: order.id).push(),
        borderRadius: BorderRadius.circular(12),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  TranslatedText(
                    'shop_order_number',
                    defaultText: 'Order #{id}',
                    args: {'id': order.id.substring(order.id.length - 6)},
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                  _StatusChip(status: order.status),
                ],
              ),
              const SizedBox(height: 8),
              TranslatedText(
                'shop_item_count',
                defaultText: '{count} item(s)',
                args: {'count': order.items.length},
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              const SizedBox(height: 4),
              Text(
                _formatDate(order.createdAt),
                style: Theme.of(
                  context,
                ).textTheme.bodySmall?.copyWith(color: Colors.grey),
              ),
              const Divider(height: 24),
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  const TranslatedText('shop_total', defaultText: 'Total'),
                  TranslatedText(
                    'shop_product_price',
                    defaultText: r'${price}',
                    args: {'price': order.totalAmount.toStringAsFixed(2)},
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _formatDate(DateTime date) {
    return '${date.day}/${date.month}/${date.year} ${date.hour}:${date.minute.toString().padLeft(2, '0')}';
  }
}

class _StatusChip extends StatelessWidget {
  final OrderStatus status;

  const _StatusChip({required this.status});

  @override
  Widget build(BuildContext context) {
    final (color, labelKey, defaultLabel) = switch (status) {
      OrderStatus.pending => (Colors.orange, 'shop_status_pending', 'Pending'),
      OrderStatus.processing => (
          Colors.blue,
          'shop_status_processing',
          'Processing',
        ),
      OrderStatus.shipped => (
          Colors.purple,
          'shop_status_shipped',
          'Shipped',
        ),
      OrderStatus.delivered => (
          Colors.green,
          'shop_status_delivered',
          'Delivered',
        ),
      OrderStatus.cancelled => (
          Colors.red,
          'shop_status_cancelled',
          'Cancelled',
        ),
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color.withAlpha(25),
        borderRadius: BorderRadius.circular(12),
      ),
      child: TranslatedText(
        labelKey,
        defaultText: defaultLabel,
        style: TextStyle(
          color: color,
          fontSize: 12,
          fontWeight: FontWeight.w500,
        ),
      ),
    );
  }
}
