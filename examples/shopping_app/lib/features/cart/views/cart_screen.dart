import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';

import '../../../shared/widgets/dialogs/confirm_dialog.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../models/cart_state.dart';
import '../view_models/cart_view_model.dart';

@Route(
  '/cart',
  name: 'cart',
  guards: [],
  transition: BottomToTopPageTransitionsBuilder(),
)
class CartScreen extends StatefulWidget {
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
          title: const Text('Cart'),
          actions: [
            if (state.items.isNotEmpty)
              IconButton(
                icon: const Icon(Icons.delete_sweep),
                tooltip: 'Clear Cart',
                onPressed: () async {
                  final confirmed = await ConfirmDialog.show(
                    context: context,
                    title: 'Clear Cart',
                    message:
                        'Are you sure you want to remove all ${state.itemCount} items from your cart?',
                    confirmText: 'Clear All',
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
                    Text('Your cart is empty'),
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
                        return Card(
                          margin: const EdgeInsets.only(bottom: 12),
                          child: Padding(
                            padding: const EdgeInsets.all(12),
                            child: Row(
                              children: [
                                Container(
                                  width: 80,
                                  height: 80,
                                  decoration: BoxDecoration(
                                    color: Colors.white,
                                    borderRadius: BorderRadius.circular(8),
                                  ),
                                  padding: const EdgeInsets.all(8),
                                  child: Image.network(
                                    item.product.image,
                                    fit: BoxFit.contain,
                                  ),
                                ),
                                const SizedBox(width: 12),
                                Expanded(
                                  child: Column(
                                    crossAxisAlignment:
                                        CrossAxisAlignment.start,
                                    children: [
                                      Text(
                                        item.product.title,
                                        maxLines: 2,
                                        overflow: TextOverflow.ellipsis,
                                        style: Theme.of(
                                          context,
                                        ).textTheme.bodyMedium,
                                      ),
                                      const SizedBox(height: 4),
                                      Text(
                                        '\$${item.product.price.toStringAsFixed(2)}',
                                        style: Theme.of(context)
                                            .textTheme
                                            .titleSmall
                                            ?.copyWith(
                                              fontWeight: FontWeight.bold,
                                            ),
                                      ),
                                    ],
                                  ),
                                ),
                                Column(
                                  children: [
                                    Row(
                                      children: [
                                        IconButton(
                                          icon: const Icon(
                                            Icons.remove_circle_outline,
                                          ),
                                          onPressed: () => context
                                              .readCartViewModel()
                                              .updateQuantity(
                                                item.product.id,
                                                item.quantity - 1,
                                              ),
                                        ),
                                        Text('${item.quantity}'),
                                        IconButton(
                                          icon: const Icon(
                                            Icons.add_circle_outline,
                                          ),
                                          onPressed: () => context
                                              .readCartViewModel()
                                              .updateQuantity(
                                                item.product.id,
                                                item.quantity + 1,
                                              ),
                                        ),
                                      ],
                                    ),
                                    IconButton(
                                      icon: const Icon(
                                        Icons.delete_outline,
                                        color: Colors.red,
                                      ),
                                      onPressed: () async {
                                        final confirmed = await ConfirmDialog.show(
                                          context: context,
                                          title: 'Remove Item',
                                          message:
                                              'Remove "${item.product.title}" from your cart?',
                                          confirmText: 'Remove',
                                          isDangerous: true,
                                        );
                                        if (confirmed == true &&
                                            context.mounted) {
                                          context
                                              .readCartViewModel()
                                              .removeFromCart(item.product.id);
                                        }
                                      },
                                    ),
                                  ],
                                ),
                              ],
                            ),
                          ),
                        );
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
                                  Text(
                                    'Total (${state.itemCount} items)',
                                    style:
                                        Theme.of(context).textTheme.titleMedium,
                                  ),
                                  Text(
                                    '\$${state.totalPrice.toStringAsFixed(2)}',
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
                                      context.routes.checkout().push(),
                                  child: const Text('Proceed to Checkout'),
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
