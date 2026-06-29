import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';
import '../models/order.dart';
import '../models/order_tracking.dart';
import '../models/order_tracking_state.dart';
import '../view_models/order_tracking_view_model.dart';
import '../view_models/orders_view_model.dart';

@AppRoute('/orders/:orderId', name: 'orderDetail')
class OrderDetailScreen extends StatefulWidget {
  const OrderDetailScreen({required this.orderId, super.key});

  final String orderId;

  @override
  State<OrderDetailScreen> createState() => _OrderDetailScreenState();
}

class _OrderDetailScreenState extends State<OrderDetailScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) context.readOrderTrackingViewModel().load(widget.orderId);
    });
  }

  @override
  void didUpdateWidget(OrderDetailScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.orderId != widget.orderId) {
      context.readOrderTrackingViewModel().load(widget.orderId);
    }
  }

  @override
  Widget build(BuildContext context) {
    final order = _findOrder(context.watchOrdersViewModel().value.orders);
    final trackingState = context.watchOrderTrackingViewModel().value;

    return Scaffold(
      appBar: AppBar(
        title: Text(
          context.tr(
            'shop_order_number',
            defaultText: 'Order #{id}',
            args: {'id': _shortId(widget.orderId)},
          ),
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          if (order != null) _OrderSummary(order: order),
          const SizedBox(height: 16),
          TranslatedText(
            'shop_tracking',
            defaultText: 'Tracking',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 12),
          switch (trackingState.status) {
            OrderTrackingStatus.initial ||
            OrderTrackingStatus.loading =>
              const Center(child: CircularProgressIndicator()),
            OrderTrackingStatus.error => Text(
                trackingState.errorMessage ??
                    context.tr(
                      'shop_tracking_failed',
                      defaultText: 'Failed to load tracking.',
                    ),
                style: const TextStyle(color: Colors.red),
              ),
            OrderTrackingStatus.success => Column(
                children: trackingState.events
                    .map((event) => _TrackingTile(event: event))
                    .toList(),
              ),
          },
        ],
      ),
    );
  }

  Order? _findOrder(List<Order> orders) {
    for (final order in orders) {
      if (order.id == widget.orderId) return order;
    }
    return null;
  }

  String _shortId(String id) =>
      id.length <= 6 ? id : id.substring(id.length - 6);
}

class _OrderSummary extends StatelessWidget {
  const _OrderSummary({required this.order});

  final Order order;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            TranslatedText(
              'shop_order_summary',
              defaultText: 'Order summary',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            TranslatedText(
              'shop_item_count',
              defaultText: '{count} item(s)',
              args: {'count': order.items.length},
            ),
            TranslatedText(
              'shop_total_price',
              defaultText: '{price} total',
              args: {
                'price': context.tr(
                  'shop_product_price',
                  defaultText: r'${price}',
                  args: {'price': order.totalAmount.toStringAsFixed(2)},
                ),
              },
            ),
            TranslatedText(
              'shop_ship_to',
              defaultText: 'Ship to {name}',
              args: {'name': order.shippingAddress.fullName},
            ),
          ],
        ),
      ),
    );
  }
}

class _TrackingTile extends StatelessWidget {
  const _TrackingTile({required this.event});

  final TrackingEvent event;

  @override
  Widget build(BuildContext context) {
    final color = event.completed ? Colors.green : Colors.grey;
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Column(
          children: [
            Icon(Icons.check_circle, color: color),
            Container(width: 2, height: 48, color: color.withAlpha(80)),
          ],
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.only(bottom: 16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  event.title,
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                Text(event.description),
                Text(
                  event.location,
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
