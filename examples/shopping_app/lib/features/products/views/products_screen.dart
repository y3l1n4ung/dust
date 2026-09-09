import 'package:dust_flutter/i18n.dart';
import 'package:flutter/material.dart';

import '../../../core/i18n/shop_i18n_keys.dart';
import '../../../route.dart';

import '../../../shared/animations/stagger_animation.dart';
import '../../../shared/widgets/bottom_sheets/product_quick_view.dart';
import '../../../shared/widgets/cards/animated_card.dart';
import '../../../shared/widgets/dialogs/confirm_dialog.dart';
import '../../../shared/widgets/loaders/product_skeleton.dart';
import '../../../shared/widgets/snackbars/app_snackbar.dart';
import '../../auth/view_models/auth_view_model.dart';
import '../../cart/view_models/cart_view_model.dart';
import '../../wishlist/view_models/wishlist_view_model.dart';
import '../models/product.dart';
import '../models/products_state.dart';
import '../view_models/products_view_model.dart';

part 'products_screen_chrome.dart';
part 'products_screen_list.dart';
part 'products_screen_card.dart';

/// Products screen.
@AppRoute(
  '/',
  name: 'products',
  guards: [],
  transition: FadeUpwardsPageTransitionsBuilder(),
)
class ProductsScreen extends StatefulWidget {
  /// Creates a [ProductsScreen].
  const ProductsScreen({super.key});

  @override
  State<ProductsScreen> createState() => _ProductsScreenState();
}

class _ProductsScreenState extends State<ProductsScreen> {
  Future<void> _onRefresh() async {
    await context.readProductsViewModel().loadProducts();
  }

  @override
  Widget build(BuildContext context) {
    final state = context.watchProductsViewModel().value;
    final cartState = context.watchCartViewModel().value;

    return Scaffold(
      appBar: AppBar(
        title: const TranslatedText('shop_title', defaultText: 'Shop'),
        actions: [
          const _LanguageToggleButton(),
          IconButton(
            tooltip: context.tr('shop_wishlist', defaultText: 'Wishlist'),
            icon: const Icon(Icons.favorite_border),
            onPressed: () => context.navigator.wishlist().push(),
          ),
          IconButton(
            tooltip: context.tr(
              'shop_support_chat',
              defaultText: 'Support Chat',
            ),
            icon: const Icon(Icons.support_agent),
            onPressed: _openSupportChat,
          ),
          _CartIconButton(itemCount: cartState.itemCount),
        ],
      ),
      drawer: const _AppDrawer(),
      body: Column(
        children: [
          if (state.status == ProductsStatus.success)
            _SearchAndSortBar(
              query: state.searchQuery,
              sortOption: state.sortOption,
              onSearch: context.readProductsViewModel().search,
              onSort: context.readProductsViewModel().sort,
            ),
          if (state.status == ProductsStatus.success)
            _CategoryFilter(
              categories: state.categories,
              selectedCategory: state.selectedCategory ?? 'all',
              onCategorySelected: (cat) =>
                  context.readProductsViewModel().selectCategory(cat),
            ),
          Expanded(child: _buildBody(state)),
        ],
      ),
    );
  }

  Widget _buildBody(ProductsState state) {
    switch (state.status) {
      case ProductsStatus.initial:
      case ProductsStatus.loading:
        return const ProductGridSkeleton();
      case ProductsStatus.error:
        return Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.error_outline, size: 64, color: Colors.red),
              const SizedBox(height: 16),
              Text(
                context.tr(
                  'shop_error_message',
                  defaultText: 'Error: {message}',
                  args: {'message': state.errorMessage ?? ''},
                ),
              ),
              const SizedBox(height: 16),
              FilledButton.icon(
                onPressed: () => context.readProductsViewModel().loadProducts(),
                icon: const Icon(Icons.refresh),
                label: const TranslatedText('shop_retry', defaultText: 'Retry'),
              ),
            ],
          ),
        );
      case ProductsStatus.success:
        return RefreshIndicator(
          onRefresh: _onRefresh,
          child: _ProductsGrid(products: state.filteredProducts),
        );
    }
  }

  Future<void> _openSupportChat() async {
    final sentMessage = await context.navigator.supportChat().push();
    if (!mounted || sentMessage != true) return;
    AppSnackbar.success(
      context,
      context.tr(
        'shop_support_message_sent',
        defaultText: 'Support message sent',
      ),
    );
  }
}
