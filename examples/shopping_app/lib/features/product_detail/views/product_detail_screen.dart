import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';

import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../cart/view_models/cart_view_model.dart';
import '../../products/models/product.dart';
import '../../products/models/products_state.dart';
import '../../products/view_models/products_view_model.dart';

@Route(
  '/product/:productId',
  name: 'productDetail',
  guards: [],
  transition: CupertinoPageTransitionsBuilder(),
)
class ProductDetailScreen extends StatelessWidget {
  final int productId;

  const ProductDetailScreen({super.key, required this.productId});

  @override
  Widget build(BuildContext context) {
    final productsState = context.productsViewModel.state;
    Product? product;
    for (final candidate in productsState.products) {
      if (candidate.id == productId) {
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
                Text('Product #$productId was not found.'),
                const SizedBox(height: 16),
                FilledButton.icon(
                  onPressed: () => context.routes.products().go(),
                  icon: const Icon(Icons.storefront),
                  label: const Text('Back to shop'),
                ),
              ],
            ),
          },
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Product Details'),
        actions: [
          IconButton(
            icon: const Icon(Icons.shopping_cart),
            onPressed: () => context.routes.cart().push(),
          ),
        ],
      ),
      body: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Hero(
              tag: 'product-image-${product.id}',
              child: Container(
                width: double.infinity,
                height: 300,
                color: Colors.white,
                padding: const EdgeInsets.all(24),
                child: Image.network(product.image, fit: BoxFit.contain),
              ),
            ),
            Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 8,
                      vertical: 4,
                    ),
                    decoration: BoxDecoration(
                      color: Colors.deepPurple.withAlpha(25),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      product.category.toUpperCase(),
                      style: Theme.of(context).textTheme.labelSmall?.copyWith(
                        color: Colors.deepPurple,
                      ),
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
                      Text(
                        '${product.rating.rate}',
                        style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                          fontWeight: FontWeight.bold,
                        ),
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
                  const SizedBox(height: 24),
                  Text(
                    'Description',
                    style: Theme.of(context).textTheme.titleMedium?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    product.description,
                    style: Theme.of(context).textTheme.bodyMedium,
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
        child: SafeArea(child: _AddToCartButton(product: product)),
      ),
    );
  }
}

class _AddToCartButton extends StatelessWidget {
  final Product product;

  const _AddToCartButton({required this.product});

  @override
  Widget build(BuildContext context) {
    final cartState = context.cartViewModel.state;
    final inCart = cartState.items.any((item) => item.product.id == product.id);

    return FilledButton.icon(
      onPressed: () {
        context.readCartViewModel().addToCart(product);
        AppSnackbar.success(
          context,
          '${product.title} added to cart',
          actionLabel: 'View Cart',
          onAction: () => context.routes.cart().push(),
        );
      },
      icon: Icon(inCart ? Icons.add_shopping_cart : Icons.shopping_cart),
      label: Text(inCart ? 'Add Another' : 'Add to Cart'),
    );
  }
}
