part of 'products_screen.dart';

// One product's card in the grid.

class _ProductCard extends StatelessWidget {
  final Product product;

  const _ProductCard({required this.product});

  @override
  Widget build(BuildContext context) {
    final isSaved = context.watchWishlistViewModel().value.containsProduct(
          product.id,
        );

    return AnimatedCard(
      onTap: () =>
          context.navigator.productDetail(productId: product.id).push(),
      onLongPress: () {
        ProductQuickView.show(
          context: context,
          product: product,
          onAddToCart: () {
            context.readCartViewModel().addToCart(product);
            AppSnackbar.success(
              context,
              context.tr(
                'shop_added_to_cart',
                defaultText: '{name} added to cart',
                args: {'name': product.title},
              ),
              actionLabel:
                  context.tr('shop_view_cart', defaultText: 'View Cart'),
              onAction: () => context.navigator.cart().push(),
            );
          },
          onViewDetails: () =>
              context.navigator.productDetail(productId: product.id).push(),
        );
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Expanded(
            flex: 3,
            child: Stack(
              children: [
                Positioned.fill(
                  child: Hero(
                    tag: 'product-image-${product.id}',
                    child: Container(
                      width: double.infinity,
                      color: Colors.white,
                      padding: const EdgeInsets.all(8),
                      child: Image.network(
                        product.image,
                        fit: BoxFit.contain,
                        errorBuilder: (_, __, ___) =>
                            const Icon(Icons.image_not_supported),
                      ),
                    ),
                  ),
                ),
                Positioned(
                  right: 4,
                  top: 4,
                  child: IconButton.filledTonal(
                    tooltip: isSaved
                        ? context.tr('shop_saved', defaultText: 'Saved')
                        : context.tr('shop_save', defaultText: 'Save'),
                    icon: Icon(
                      isSaved ? Icons.favorite : Icons.favorite_border,
                    ),
                    onPressed: () =>
                        context.readWishlistViewModel().toggle(product),
                  ),
                ),
              ],
            ),
          ),
          Expanded(
            flex: 2,
            child: Padding(
              padding: const EdgeInsets.all(8),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    product.title,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                  TranslatedText.dynamic(
                    product.categoryTranslationKey,
                    fallback: product.categoryFallbackLabel,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Theme.of(context).textTheme.labelSmall,
                  ),
                  const Spacer(),
                  Row(
                    children: [
                      Expanded(
                        child: TranslatedText(
                          'shop_product_price',
                          defaultText: r'${price}',
                          args: {'price': product.priceLabel},
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: Theme.of(context)
                              .textTheme
                              .titleMedium
                              ?.copyWith(fontWeight: FontWeight.bold),
                        ),
                      ),
                      const SizedBox(width: 4),
                      Icon(Icons.star, size: 16, color: Colors.amber[700]),
                      const SizedBox(width: 2),
                      Flexible(
                        child: TranslatedText(
                          'shop_rating_summary',
                          defaultText: '{rating} ({count})',
                          args: {
                            'rating': product.rating.rateLabel,
                            'count': product.rating.count,
                          },
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}
