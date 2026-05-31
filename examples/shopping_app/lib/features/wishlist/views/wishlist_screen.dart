import 'package:flutter/material.dart' hide Route;

import '../../../route.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../cart/view_models/cart_view_model.dart';
import '../models/wishlist_item.dart';
import '../view_models/wishlist_view_model.dart';

@Route('/wishlist', name: 'wishlist', guards: [])
class WishlistScreen extends StatelessWidget {
  const WishlistScreen({super.key});

  @override
  Widget build(BuildContext context) {
    final state = context.watchWishlistViewModel().value;

    return WishlistViewModelListener(
      listener: (context, effect) {
        if (effect is WishlistEffect) {
          AppSnackbar.info(context, effect.message);
        }
      },
      child: Scaffold(
        appBar: AppBar(title: const Text('Wishlist')),
        body: state.items.isEmpty
            ? Center(
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    const Icon(
                      Icons.favorite_border,
                      size: 80,
                      color: Colors.grey,
                    ),
                    const SizedBox(height: 16),
                    const Text('No saved products yet'),
                    const SizedBox(height: 16),
                    FilledButton.icon(
                      onPressed: () => context.routes.products().go(),
                      icon: const Icon(Icons.storefront),
                      label: const Text('Browse products'),
                    ),
                  ],
                ),
              )
            : GridView.builder(
                padding: const EdgeInsets.all(16),
                gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                  maxCrossAxisExtent: 400,
                  mainAxisExtent: 100,
                  crossAxisSpacing: 16,
                  mainAxisSpacing: 16,
                ),
                itemCount: state.items.length,
                itemBuilder: (context, index) =>
                    _WishlistTile(item: state.items[index]),
              ),
      ),
    );
  }
}

class _WishlistTile extends StatelessWidget {
  const _WishlistTile({required this.item});

  final WishlistItem item;

  @override
  Widget build(BuildContext context) {
    final product = item.product;
    return Card(
      margin: const EdgeInsets.only(bottom: 12),
      child: ListTile(
        contentPadding: const EdgeInsets.all(12),
        leading: SizedBox(
          width: 56,
          height: 56,
          child: Image.network(
            product.image,
            fit: BoxFit.contain,
            errorBuilder: (_, __, ___) => const Icon(Icons.image_not_supported),
          ),
        ),
        title: Text(
          product.title,
          maxLines: 2,
          overflow: TextOverflow.ellipsis,
        ),
        subtitle: Text('\$${product.price.toStringAsFixed(2)}'),
        onTap: () => context.routes.productDetail(productId: product.id).push(),
        trailing: Wrap(
          spacing: 4,
          children: [
            IconButton(
              tooltip: 'Add to cart',
              icon: const Icon(Icons.add_shopping_cart),
              onPressed: () {
                context.readCartViewModel().addToCart(product);
                AppSnackbar.success(context, '${product.title} added to cart');
              },
            ),
            IconButton(
              tooltip: 'Remove',
              icon: const Icon(Icons.delete_outline),
              onPressed: () =>
                  context.readWishlistViewModel().remove(product.id),
            ),
          ],
        ),
      ),
    );
  }
}
