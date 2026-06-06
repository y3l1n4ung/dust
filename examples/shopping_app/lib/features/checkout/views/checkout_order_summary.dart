import 'package:flutter/material.dart';

import '../../cart/models/cart_state.dart';
import '../models/checkout_quote.dart';
import '../models/checkout_state.dart';

class CheckoutOrderSummary extends StatelessWidget {
  const CheckoutOrderSummary({
    required this.cartState,
    required this.checkoutState,
    required this.couponController,
    required this.onApplyCoupon,
    super.key,
  });

  final CartState cartState;
  final CheckoutState checkoutState;
  final TextEditingController couponController;
  final Future<void> Function() onApplyCoupon;

  @override
  Widget build(BuildContext context) {
    final total = checkoutState.quote?.total ?? cartState.totalPrice;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            ...cartState.items.map(
              (item) => Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Expanded(
                      child: Text(
                        '${item.product.title} x${item.quantity}',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    Text('\$${item.totalPrice.toStringAsFixed(2)}'),
                  ],
                ),
              ),
            ),
            const Divider(),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: couponController,
                    decoration: const InputDecoration(
                      labelText: 'Coupon code',
                      helperText: 'Try DUST10 or SHIPFREE',
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                FilledButton.tonal(
                  onPressed: checkoutState.isQuoteLoading
                      ? null
                      : onApplyCoupon,
                  child: checkoutState.isQuoteLoading
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Text('Apply'),
                ),
              ],
            ),
            if (checkoutState.quote != null) ...[
              const SizedBox(height: 12),
              _QuoteBreakdown(quote: checkoutState.quote!),
              const Divider(),
            ] else
              const Divider(),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('Total', style: Theme.of(context).textTheme.titleMedium),
                Text(
                  '\$${total.toStringAsFixed(2)}',
                  style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                    color: Colors.deepPurple,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _QuoteBreakdown extends StatelessWidget {
  const _QuoteBreakdown({required this.quote});

  final CheckoutQuote quote;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        _QuoteRow(label: 'Subtotal', value: quote.subtotal),
        _QuoteRow(label: 'Discount', value: -quote.discount),
        _QuoteRow(label: 'Shipping', value: quote.shipping),
        _QuoteRow(label: 'Tax', value: quote.tax),
        if (quote.appliedCoupon != null)
          Align(
            alignment: Alignment.centerLeft,
            child: Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Text(
                'Applied ${quote.appliedCoupon}',
                style: Theme.of(context).textTheme.labelMedium?.copyWith(
                  color: Colors.green,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ),
      ],
    );
  }
}

class _QuoteRow extends StatelessWidget {
  const _QuoteRow({required this.label, required this.value});

  final String label;
  final double value;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label),
          Text(
            value < 0
                ? '-\$${(-value).toStringAsFixed(2)}'
                : '\$${value.toStringAsFixed(2)}',
          ),
        ],
      ),
    );
  }
}
