import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';

import '../../../shared/widgets/dialogs/confirm_dialog.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../models/cart_state.dart';
import '../view_models/cart_view_model.dart';
import 'package:shopping_app/features/cart/models/cart_item.dart';

part 'cart_screen_item.dart';

/// Cart screen.
@AppRoute(
  '/cart',
  name: 'cart',
  guards: [],
  transition: BottomToTopPageTransitionsBuilder(),
)
class CartScreen extends StatefulWidget {
  /// Creates a [CartScreen].
  const CartScreen({super.key});

  @override
  State<CartScreen> createState() => _CartScreenState();
}

class _CartScreenState extends State<CartScreen> {
  void _showNotification(CartNotification notification) {
    switch (notification.type) {
      case CartNotificationType.itemAdded:
        AppSnackbar.success(context, notification.message);
      case CartNotificationType.itemRemoved:
        AppSnackbar.info(context, notification.message);
      case CartNotificationType.quantityUpdated:
        AppSnackbar.info(context, notification.message);
      case CartNotificationType.cleared:
        AppSnackbar.info(context, notification.message);
    }
  }

  @override
  Widget build(BuildContext context) {
    final state = context.watchCartViewModel().value;

    return CartViewModelListener(
      listener: (context, effect) {
        if (effect is CartNotification) {
          _showNotification(effect);
        }
      },
      child: Scaffold(
        appBar: AppBar(
          title: const TranslatedText('shop_cart', defaultText: 'Cart'),
          actions: [
            if (state.items.isNotEmpty)
              IconButton(
                icon: const Icon(Icons.delete_sweep),
                tooltip: context.tr(
                  'shop_clear_cart',
                  defaultText: 'Clear Cart',
                ),
                onPressed: () async {
                  final confirmed = await ConfirmDialog.show(
                    context: context,
                    title: context.tr(
                      'shop_clear_cart',
                      defaultText: 'Clear Cart',
                    ),
                    message: context.tr(
                      'shop_clear_cart_message',
                      defaultText:
                          'Are you sure you want to remove all {count} items from your cart?',
                      args: {'count': state.itemCount},
                    ),
                    confirmText: context.tr(
                      'shop_clear_all',
                      defaultText: 'Clear All',
                    ),
                    cancelText:
                        context.tr('shop_cancel', defaultText: 'Cancel'),
                    isDangerous: true,
                  );
                  if (confirmed == true && context.mounted) {
                    context.readCartViewModel().clearCart();
                  }
                },
              ),
          ],
        ),
        body: state.items.isEmpty
            ? const Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    Icon(
                      Icons.shopping_cart_outlined,
                      size: 80,
                      color: Colors.grey,
                    ),
                    SizedBox(height: 16),
                    TranslatedText(
                      'shop_cart_empty',
                      defaultText: 'Your cart is empty',
                    ),
                  ],
                ),
              )
            : Column(
                children: [
                  Expanded(
                    child: ListView.builder(
                      padding: const EdgeInsets.all(16),
                      itemCount: state.items.length,
                      itemBuilder: (context, index) {
                        final item = state.items[index];
                        return _CartItemCard(item: item);
                      },
                    ),
                  ),
                  Container(
                    width: double.infinity,
                    decoration: BoxDecoration(
                      color: Theme.of(context).cardColor,
                      boxShadow: [
                        BoxShadow(
                          color: Colors.black.withAlpha(25),
                          blurRadius: 10,
                          offset: const Offset(0, -2),
                        ),
                      ],
                    ),
                    child: SafeArea(
                      child: Center(
                        child: Container(
                          constraints: const BoxConstraints(maxWidth: 800),
                          padding: const EdgeInsets.all(16),
                          child: Column(
                            children: [
                              Row(
                                mainAxisAlignment:
                                    MainAxisAlignment.spaceBetween,
                                children: [
                                  TranslatedText(
                                    'shop_total_items',
                                    defaultText: 'Total ({count} items)',
                                    args: {'count': state.itemCount},
                                    style: Theme.of(
                                      context,
                                    ).textTheme.titleMedium,
                                  ),
                                  TranslatedText(
                                    'shop_product_price',
                                    defaultText: r'${price}',
                                    args: {
                                      'price':
                                          state.totalPrice.toStringAsFixed(2),
                                    },
                                    style: Theme.of(context)
                                        .textTheme
                                        .titleLarge
                                        ?.copyWith(fontWeight: FontWeight.bold),
                                  ),
                                ],
                              ),
                              const SizedBox(height: 16),
                              SizedBox(
                                width: double.infinity,
                                child: FilledButton(
                                  onPressed: () =>
                                      context.navigator.checkout().push(),
                                  child: const TranslatedText(
                                    'shop_proceed_to_checkout',
                                    defaultText: 'Proceed to Checkout',
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
      ),
    );
  }
}
