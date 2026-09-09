import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

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

part 'product_detail_screen_sections.dart';

/// Product detail screen.
@AppRoute('/product/:productId', name: 'productDetail', guards: [])
class ProductDetailScreen extends StatefulWidget {
  /// Product ID.
  final int productId;

  /// Creates a [ProductDetailScreen].
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
        appBar: AppBar(
          title: const TranslatedText(
            'shop_product_details',
            defaultText: 'Product Details',
          ),
        ),
        body: Center(
          child: switch (productsState.status) {
            ProductsStatus.initial ||
            ProductsStatus.loading =>
              const CircularProgressIndicator(),
            _ => Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  const Icon(Icons.search_off, size: 72, color: Colors.grey),
                  const SizedBox(height: 16),
                  Text(
                    context.tr(
                      'shop_product_not_found',
                      defaultText: 'Product #{id} was not found.',
                      args: {'id': widget.productId},
                    ),
                  ),
                  const SizedBox(height: 16),
                  FilledButton.icon(
                    onPressed: () => context.navigator.products().go(),
                    icon: const Icon(Icons.storefront),
                    label: const TranslatedText(
                      'shop_back_to_shop',
                      defaultText: 'Back to shop',
                    ),
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
          title: const TranslatedText(
            'shop_product_details',
            defaultText: 'Product Details',
          ),
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
                    TranslatedText(
                      'shop_description',
                      defaultText: 'Description',
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
