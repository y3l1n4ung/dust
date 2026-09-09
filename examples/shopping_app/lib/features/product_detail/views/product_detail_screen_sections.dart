part of 'product_detail_screen.dart';

// The header, reviews and recommendation sections.

class _ProductHeader extends StatelessWidget {
  const _ProductHeader({required this.product});

  final Product product;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            color: Colors.deepPurple.withAlpha(25),
            borderRadius: BorderRadius.circular(4),
          ),
          child: TranslatedText.dynamic(
            product.categoryTranslationKey,
            fallback: product.categoryFallbackLabel,
            style: Theme.of(
              context,
            ).textTheme.labelSmall?.copyWith(color: Colors.deepPurple),
          ),
        ),
        const SizedBox(height: 12),
        Text(
          product.title,
          style: Theme.of(context).textTheme.headlineSmall,
        ),
        const SizedBox(height: 8),
        Row(
          children: [
            Icon(Icons.star, color: Colors.amber[700], size: 20),
            const SizedBox(width: 4),
            Text(product.rating.rateLabel),
            const SizedBox(width: 8),
            TranslatedText(
              'shop_review_count',
              defaultText: '({count} reviews)',
              args: {'count': product.rating.count},
              style: Theme.of(
                context,
              ).textTheme.bodyMedium?.copyWith(color: Colors.grey),
            ),
          ],
        ),
        const SizedBox(height: 16),
        TranslatedText(
          'shop_product_price',
          defaultText: r'${price}',
          args: {'price': product.priceLabel},
          style: Theme.of(context).textTheme.headlineMedium?.copyWith(
                fontWeight: FontWeight.bold,
                color: Colors.deepPurple,
              ),
        ),
      ],
    );
  }
}

class _ReviewsSection extends StatelessWidget {
  const _ReviewsSection({required this.state});

  final ProductDetailState state;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TranslatedText(
          'shop_reviews',
          defaultText: 'Reviews',
          style: Theme.of(context).textTheme.titleLarge,
        ),
        const SizedBox(height: 12),
        switch (state.status) {
          ProductDetailStatus.initial ||
          ProductDetailStatus.loading =>
            const Center(child: CircularProgressIndicator()),
          ProductDetailStatus.error => Text(
              state.errorMessage ??
                  context.tr(
                    'shop_reviews_failed',
                    defaultText: 'Failed to load reviews.',
                  ),
              style: const TextStyle(color: Colors.red),
            ),
          ProductDetailStatus.success => Column(
              children: state.reviews
                  .map((review) => _ReviewTile(review: review))
                  .toList(),
            ),
        },
      ],
    );
  }
}

class _ReviewTile extends StatelessWidget {
  const _ReviewTile({required this.review});

  final ProductReview review;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.only(bottom: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  review.authorName,
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const Spacer(),
                const Icon(Icons.star, color: Colors.amber, size: 16),
                Text(review.rating.toStringAsFixed(1)),
              ],
            ),
            const SizedBox(height: 6),
            Text(review.comment),
            if (review.verifiedPurchase)
              Padding(
                padding: const EdgeInsets.only(top: 6),
                child: TranslatedText(
                  'shop_verified_purchase',
                  defaultText: 'Verified purchase',
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: Colors.green,
                        fontWeight: FontWeight.bold,
                      ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _RecommendationsSection extends StatelessWidget {
  const _RecommendationsSection({required this.products});

  final List<Product> products;

  @override
  Widget build(BuildContext context) {
    if (products.isEmpty) return const SizedBox.shrink();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        TranslatedText(
          'shop_recommendations',
          defaultText: 'You may also like',
          style: Theme.of(context).textTheme.titleLarge,
        ),
        const SizedBox(height: 12),
        SizedBox(
          height: 150,
          child: ListView.separated(
            scrollDirection: Axis.horizontal,
            itemCount: products.length,
            separatorBuilder: (_, __) => const SizedBox(width: 12),
            itemBuilder: (context, index) {
              final product = products[index];
              return SizedBox(
                width: 140,
                child: InkWell(
                  onTap: () => context.navigator
                      .productDetail(productId: product.id)
                      .replace(),
                  child: Card(
                    child: Padding(
                      padding: const EdgeInsets.all(8),
                      child: Column(
                        children: [
                          Expanded(
                            child: Image.network(
                              product.image,
                              fit: BoxFit.contain,
                            ),
                          ),
                          Text(
                            product.title,
                            maxLines: 2,
                            overflow: TextOverflow.ellipsis,
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      ],
    );
  }
}

class _AddToCartButton extends StatelessWidget {
  final Product product;

  const _AddToCartButton({required this.product});

  @override
  Widget build(BuildContext context) {
    final cartState = context.watchCartViewModel().value;
    final inCart = cartState.items.any((item) => item.product.id == product.id);

    return FilledButton.icon(
      onPressed: () {
        context.readCartViewModel().addToCart(product);
        AppSnackbar.success(
          context,
          context.tr(
            'shop_added_to_cart',
            defaultText: '{name} added to cart',
            args: {'name': product.title},
          ),
          actionLabel: context.tr('shop_view_cart', defaultText: 'View Cart'),
          onAction: () => context.navigator.cart().push(),
        );
      },
      icon: Icon(inCart ? Icons.add_shopping_cart : Icons.shopping_cart),
      label: inCart
          ? const TranslatedText(
              'shop_add_another',
              defaultText: 'Add Another',
            )
          : const TranslatedText(
              'shop_add_to_cart',
              defaultText: 'Add to Cart',
            ),
    );
  }
}
