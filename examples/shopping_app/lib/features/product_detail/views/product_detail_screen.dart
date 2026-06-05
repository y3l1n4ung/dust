import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../cart/view_models/cart_view_model.dart';
import '../../products/models/product.dart';
import '../../products/models/products_state.dart';
import '../../products/view_models/products_view_model.dart';
import '../../wishlist/view_models/wishlist_view_model.dart';
import '../models/product_detail_state.dart';
import '../models/product_review.dart';
import '../view_models/product_detail_view_model.dart';

@Route(
  '/product/:productId',
  name: 'productDetail',
  guards: [],
  transition: CupertinoPageTransitionsBuilder(),
)
class ProductDetailScreen extends StatefulWidget {
  final int productId;

  const ProductDetailScreen({super.key, required this.productId});

  @override
  State<ProductDetailScreen> createState() => _ProductDetailScreenState();
}

class _ProductDetailScreenState extends State<ProductDetailScreen> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) context.readProductDetailViewModel().load(widget.productId);
    });
  }

  @override
  void didUpdateWidget(ProductDetailScreen oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.productId != widget.productId) {
      context.readProductDetailViewModel().load(widget.productId);
    }
  }

  @override
  Widget build(BuildContext context) {
    final productsState = context.watchProductsViewModel().value;
    Product? product;
    for (final candidate in productsState.products) {
      if (candidate.id == widget.productId) {
        product = candidate;
        break;
      }
    }

    if (product == null) {
      return Scaffold(
        appBar: AppBar(title: const Text('Product Details')),
        body: Center(
          child: switch (productsState.status) {
            ProductsStatus.initial ||
            ProductsStatus.loading => const CircularProgressIndicator(),
            _ => Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Icon(Icons.search_off, size: 72, color: Colors.grey),
                const SizedBox(height: 16),
                Text('Product #${widget.productId} was not found.'),
                const SizedBox(height: 16),
                FilledButton.icon(
                  onPressed: () => context.navigator.products().go(),
                  icon: const Icon(Icons.storefront),
                  label: const Text('Back to shop'),
                ),
              ],
            ),
          },
        ),
      );
    }

    final selectedProduct = product;
    final detailState = context.watchProductDetailViewModel().value;
    final wishlistState = context.watchWishlistViewModel().value;
    final isSaved = wishlistState.containsProduct(selectedProduct.id);

    return WishlistViewModelListener(
      listener: (context, effect) {
        if (effect is WishlistEffect) AppSnackbar.info(context, effect.message);
      },
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Product Details'),
          actions: [
            IconButton(
              icon: Icon(isSaved ? Icons.favorite : Icons.favorite_border),
              onPressed: () =>
                  context.readWishlistViewModel().toggle(selectedProduct),
            ),
            IconButton(
              icon: const Icon(Icons.shopping_cart),
              onPressed: () => context.navigator.cart().push(),
            ),
          ],
        ),
        body: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Hero(
                tag: 'product-image-${selectedProduct.id}',
                child: Container(
                  width: double.infinity,
                  height: 300,
                  color: Colors.white,
                  padding: const EdgeInsets.all(24),
                  child: Image.network(
                    selectedProduct.image,
                    fit: BoxFit.contain,
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    _ProductHeader(product: selectedProduct),
                    const SizedBox(height: 24),
                    Text(
                      'Description',
                      style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(selectedProduct.description),
                    const SizedBox(height: 28),
                    _ReviewsSection(state: detailState),
                    const SizedBox(height: 28),
                    _RecommendationsSection(
                      products: detailState.recommendations,
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        bottomNavigationBar: Container(
          padding: const EdgeInsets.all(16),
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
          child: SafeArea(child: _AddToCartButton(product: selectedProduct)),
        ),
      ),
    );
  }
}

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
          child: Text(
            product.category.toUpperCase(),
            style: Theme.of(
              context,
            ).textTheme.labelSmall?.copyWith(color: Colors.deepPurple),
          ),
        ),
        const SizedBox(height: 12),
        Text(product.title, style: Theme.of(context).textTheme.headlineSmall),
        const SizedBox(height: 8),
        Row(
          children: [
            Icon(Icons.star, color: Colors.amber[700], size: 20),
            const SizedBox(width: 4),
            Text(
              '${product.rating.rate}',
              style: Theme.of(
                context,
              ).textTheme.bodyLarge?.copyWith(fontWeight: FontWeight.bold),
            ),
            const SizedBox(width: 8),
            Text(
              '(${product.rating.count} reviews)',
              style: Theme.of(
                context,
              ).textTheme.bodyMedium?.copyWith(color: Colors.grey),
            ),
          ],
        ),
        const SizedBox(height: 16),
        Text(
          '\$${product.price.toStringAsFixed(2)}',
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
        Text('Reviews', style: Theme.of(context).textTheme.titleLarge),
        const SizedBox(height: 12),
        switch (state.status) {
          ProductDetailStatus.initial || ProductDetailStatus.loading =>
            const Center(child: CircularProgressIndicator()),
          ProductDetailStatus.error => Text(
            state.errorMessage ?? 'Failed to load reviews.',
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
                child: Text(
                  'Verified purchase',
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
        Text(
          'You may also like',
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
          '${product.title} added to cart',
          actionLabel: 'View Cart',
          onAction: () => context.navigator.cart().push(),
        );
      },
      icon: Icon(inCart ? Icons.add_shopping_cart : Icons.shopping_cart),
      label: Text(inCart ? 'Add Another' : 'Add to Cart'),
    );
  }
}
