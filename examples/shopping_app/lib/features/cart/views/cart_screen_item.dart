part of 'cart_screen.dart';

/// One line in the cart: the product, its quantity, and the controls that
/// change or remove it.
///
/// Extracted from `CartScreen.build`, which held the whole tree inline.
class _CartItemCard extends StatelessWidget {
  const _CartItemCard({required this.item});

  /// The cart line this card renders.
  final CartItem item;

  @override
  Widget build(BuildContext context) {
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
                crossAxisAlignment: CrossAxisAlignment.start,
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
                  TranslatedText(
                    'shop_product_price',
                    defaultText: r'${price}',
                    args: {
                      'price': item.product.priceLabel,
                    },
                    style: Theme.of(context).textTheme.titleSmall?.copyWith(
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
                      onPressed: () =>
                          context.readCartViewModel().updateQuantity(
                                item.product.id,
                                item.quantity - 1,
                              ),
                    ),
                    Text('${item.quantity}'),
                    IconButton(
                      icon: const Icon(
                        Icons.add_circle_outline,
                      ),
                      onPressed: () =>
                          context.readCartViewModel().updateQuantity(
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
                      title: context.tr(
                        'shop_remove_item',
                        defaultText: 'Remove Item',
                      ),
                      message: context.tr(
                        'shop_remove_item_message',
                        defaultText: 'Remove "{name}" from your cart?',
                        args: {'name': item.product.title},
                      ),
                      confirmText: context.tr(
                        'shop_remove',
                        defaultText: 'Remove',
                      ),
                      cancelText: context.tr(
                        'shop_cancel',
                        defaultText: 'Cancel',
                      ),
                      isDangerous: true,
                    );
                    if (confirmed == true && context.mounted) {
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
  }
}
