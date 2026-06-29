import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../route.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../cart/view_models/cart_view_model.dart';
import '../models/wishlist_item.dart';
import '../view_models/wishlist_view_model.dart';

@AppRoute('/wishlist', name: 'wishlist', guards: [])
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
        appBar: AppBar(
          title: const TranslatedText('shop_wishlist', defaultText: 'Wishlist'),
        ),
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
                    const TranslatedText(
                      'shop_wishlist_empty',
                      defaultText: 'No saved products yet',
                    ),
                    const SizedBox(height: 16),
                    FilledButton.icon(
                      onPressed: () => context.navigator.products().go(),
                      icon: const Icon(Icons.storefront),
                      label: const TranslatedText(
                        'shop_browse_products',
                        defaultText: 'Browse products',
                      ),
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
        subtitle: TranslatedText(
          'shop_product_price',
          defaultText: r'${price}',
          args: {'price': product.priceLabel},
        ),
        onTap: () =>
            context.navigator.productDetail(productId: product.id).push(),
        trailing: Wrap(
          spacing: 4,
          children: [
            IconButton(
              tooltip:
                  context.tr('shop_add_to_cart', defaultText: 'Add to Cart'),
              icon: const Icon(Icons.add_shopping_cart),
              onPressed: () {
                context.readCartViewModel().addToCart(product);
                AppSnackbar.success(
                  context,
                  context.tr(
                    'shop_added_to_cart',
                    defaultText: '{name} added to cart',
                    args: {'name': product.title},
                  ),
                );
              },
            ),
            IconButton(
              tooltip: context.tr('shop_remove', defaultText: 'Remove'),
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
