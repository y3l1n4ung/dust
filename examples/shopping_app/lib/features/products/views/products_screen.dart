import 'package:flutter/material.dart' hide Route;

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

@Route(
  '/',
  name: 'products',
  guards: [],
  transition: FadeUpwardsPageTransitionsBuilder(),
)
class ProductsScreen extends StatefulWidget {
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
    final viewModel = context.productsViewModel;
    final state = viewModel.state;
    final cartState = context.cartViewModel.state;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Shop'),
        actions: [
          IconButton(
            tooltip: 'Wishlist',
            icon: const Icon(Icons.favorite_border),
            onPressed: () => context.routes.wishlist().push(),
          ),
          IconButton(
            tooltip: 'Support chat',
            icon: const Icon(Icons.support_agent),
            onPressed: () => context.routes.supportChat().push(),
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
              Text('Error: ${state.errorMessage}'),
              const SizedBox(height: 16),
              FilledButton.icon(
                onPressed: () => context.readProductsViewModel().loadProducts(),
                icon: const Icon(Icons.refresh),
                label: const Text('Retry'),
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
}

class _CartIconButton extends StatefulWidget {
  final int itemCount;

  const _CartIconButton({required this.itemCount});

  @override
  State<_CartIconButton> createState() => _CartIconButtonState();
}

class _CartIconButtonState extends State<_CartIconButton>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _scaleAnimation;
  int _previousCount = 0;

  @override
  void initState() {
    super.initState();
    _previousCount = widget.itemCount;
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 300),
    );
    _scaleAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 1.0, end: 1.3), weight: 1),
      TweenSequenceItem(tween: Tween(begin: 1.3, end: 1.0), weight: 1),
    ]).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));
  }

  @override
  void didUpdateWidget(_CartIconButton oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.itemCount > _previousCount) {
      _controller.forward(from: 0);
    }
    _previousCount = widget.itemCount;
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ScaleTransition(
      scale: _scaleAnimation,
      child: Stack(
        children: [
          IconButton(
            icon: const Icon(Icons.shopping_cart),
            onPressed: () => context.routes.cart().push(),
          ),
          if (widget.itemCount > 0)
            Positioned(
              right: 4,
              top: 4,
              child: Container(
                padding: const EdgeInsets.all(4),
                decoration: const BoxDecoration(
                  color: Colors.red,
                  shape: BoxShape.circle,
                ),
                constraints: const BoxConstraints(minWidth: 18, minHeight: 18),
                child: Text(
                  widget.itemCount > 99 ? '99+' : '${widget.itemCount}',
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                  ),
                  textAlign: TextAlign.center,
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _AppDrawer extends StatelessWidget {
  const _AppDrawer();

  @override
  Widget build(BuildContext context) {
    final authState = context.authViewModel.state;

    return Drawer(
      child: ListView(
        padding: EdgeInsets.zero,
        children: [
          DrawerHeader(
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primary,
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                const CircleAvatar(
                  radius: 30,
                  child: Icon(Icons.person, size: 30),
                ),
                const SizedBox(height: 12),
                Text(
                  authState.isAuthenticated
                      ? authState.user?.name.fullName ?? 'User'
                      : 'Guest',
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                if (authState.isAuthenticated)
                  Text(
                    authState.user?.email ?? '',
                    style: TextStyle(
                      color: Colors.white.withAlpha(200),
                      fontSize: 14,
                    ),
                  ),
              ],
            ),
          ),
          ListTile(
            leading: const Icon(Icons.home),
            title: const Text('Shop'),
            onTap: () {
              Navigator.pop(context);
              context.routes.products().go();
            },
          ),
          ListTile(
            leading: const Icon(Icons.shopping_cart),
            title: const Text('Cart'),
            onTap: () {
              Navigator.pop(context);
              context.routes.cart().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.favorite),
            title: const Text('Wishlist'),
            onTap: () {
              Navigator.pop(context);
              context.routes.wishlist().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.cloud_queue),
            title: const Text('FakeStore Carts'),
            onTap: () {
              Navigator.pop(context);
              context.routes.demoCarts().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.support_agent),
            title: const Text('Support Chat'),
            onTap: () {
              Navigator.pop(context);
              context.routes.supportChat().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.receipt_long),
            title: const Text('Orders'),
            onTap: () {
              Navigator.pop(context);
              context.routes.orders().push();
            },
          ),
          ListTile(
            leading: const Icon(Icons.person),
            title: const Text('Profile'),
            onTap: () {
              Navigator.pop(context);
              context.routes.profile().push();
            },
          ),
          const Divider(),
          if (authState.isAuthenticated)
            ListTile(
              leading: const Icon(Icons.logout, color: Colors.red),
              title: const Text('Logout', style: TextStyle(color: Colors.red)),
              onTap: () async {
                final confirmed = await ConfirmDialog.show(
                  context: context,
                  title: 'Logout',
                  message: 'Are you sure you want to logout?',
                  confirmText: 'Logout',
                  isDangerous: true,
                );
                if (confirmed == true && context.mounted) {
                  context.readAuthViewModel().logout();
                  Navigator.pop(context);
                }
              },
            )
          else
            ListTile(
              leading: const Icon(Icons.login),
              title: const Text('Sign In'),
              onTap: () {
                Navigator.pop(context);
                context.routes.login().go();
              },
            ),
        ],
      ),
    );
  }
}

class _CategoryFilter extends StatelessWidget {
  final List<String> categories;
  final String selectedCategory;
  final ValueChanged<String> onCategorySelected;

  const _CategoryFilter({
    required this.categories,
    required this.selectedCategory,
    required this.onCategorySelected,
  });

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 50,
      child: ListView.builder(
        scrollDirection: Axis.horizontal,
        padding: const EdgeInsets.symmetric(horizontal: 8),
        itemCount: categories.length,
        itemBuilder: (context, index) {
          final category = categories[index];
          final isSelected = category == selectedCategory;
          return Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4),
            child: FilterChip(
              label: Text(category.toUpperCase()),
              selected: isSelected,
              onSelected: (_) => onCategorySelected(category),
            ),
          );
        },
      ),
    );
  }
}

class _SearchAndSortBar extends StatelessWidget {
  const _SearchAndSortBar({
    required this.query,
    required this.sortOption,
    required this.onSearch,
    required this.onSort,
  });

  final String query;
  final ProductSortOption sortOption;
  final ValueChanged<String> onSearch;
  final ValueChanged<ProductSortOption> onSort;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
      child: Row(
        children: [
          Expanded(
            child: TextFormField(
              initialValue: query,
              decoration: const InputDecoration(
                prefixIcon: Icon(Icons.search),
                hintText: 'Search products',
                border: OutlineInputBorder(),
                isDense: true,
              ),
              onChanged: onSearch,
            ),
          ),
          const SizedBox(width: 12),
          DropdownButton<ProductSortOption>(
            value: sortOption,
            onChanged: (value) {
              if (value != null) onSort(value);
            },
            items: const [
              DropdownMenuItem(
                value: ProductSortOption.featured,
                child: Text('Featured'),
              ),
              DropdownMenuItem(
                value: ProductSortOption.priceLow,
                child: Text('Price ↑'),
              ),
              DropdownMenuItem(
                value: ProductSortOption.priceHigh,
                child: Text('Price ↓'),
              ),
              DropdownMenuItem(
                value: ProductSortOption.ratingHigh,
                child: Text('Rating'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _ProductsGrid extends StatelessWidget {
  final List<Product> products;

  const _ProductsGrid({required this.products});

  @override
  Widget build(BuildContext context) {
    if (products.isEmpty) {
      return const Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(Icons.search_off, size: 72, color: Colors.grey),
            SizedBox(height: 12),
            Text('No products match your filters'),
          ],
        ),
      );
    }

    return GridView.builder(
      padding: const EdgeInsets.all(16),
      gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
        maxCrossAxisExtent: 220,
        childAspectRatio: 0.65,
        crossAxisSpacing: 16,
        mainAxisSpacing: 16,
      ),
      itemCount: products.length,
      itemBuilder: (context, index) {
        final product = products[index];
        return AnimatedGridItem(
          index: index,
          child: _ProductCard(product: product),
        );
      },
    );
  }
}

class _ProductCard extends StatelessWidget {
  final Product product;

  const _ProductCard({required this.product});

  @override
  Widget build(BuildContext context) {
    final isSaved = context.wishlistViewModel.state.containsProduct(product.id);

    return AnimatedCard(
      onTap: () => context.routes.productDetail(productId: product.id).push(),
      onLongPress: () {
        ProductQuickView.show(
          context: context,
          product: product,
          onAddToCart: () {
            context.readCartViewModel().addToCart(product);
            AppSnackbar.success(
              context,
              '${product.title} added to cart',
              actionLabel: 'View',
              onAction: () => context.routes.cart().push(),
            );
          },
          onViewDetails: () =>
              context.routes.productDetail(productId: product.id).push(),
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
                    tooltip: isSaved ? 'Saved' : 'Save',
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
                  const Spacer(),
                  Row(
                    children: [
                      Text(
                        '\$${product.price.toStringAsFixed(2)}',
                        style: Theme.of(context).textTheme.titleMedium
                            ?.copyWith(fontWeight: FontWeight.bold),
                      ),
                      const Spacer(),
                      Icon(Icons.star, size: 16, color: Colors.amber[700]),
                      Text(
                        product.rating.rate.toStringAsFixed(1),
                        style: Theme.of(context).textTheme.bodySmall,
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
