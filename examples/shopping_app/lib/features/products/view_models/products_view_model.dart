import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../../../core/logging/logger.dart';
import '../models/product.dart';
import '../models/products_state.dart';

part 'products_view_model.g.dart';

/// Products view model args model for the shopping app example.
final class ProductsViewModelArgs extends ViewModelArgs {
  /// Creates a [ProductsViewModelArgs].
  const ProductsViewModelArgs({required this.repository, super.observer});

  /// Repository.
  final ShoppingRepository repository;
}

/// Products view model for the shopping app example.
@ViewModel(state: ProductsState, args: ProductsViewModelArgs)
class ProductsViewModel extends $ProductsViewModel {
  /// Creates a [ProductsViewModel].
  ProductsViewModel(super.args);

  static const Object _loadProductsAction = Object();

  @override
  Future<void> onInit() => loadProducts();

  /// Loads products.
  Future<void> loadProducts() async {
    await runAction<List<Product>>(
      _loadProductsAction,
      onStart: () {
        logger.info('PRODUCTS', 'Loading products...');
        emit(state.copyWith(status: ProductsStatus.loading));
      },
      run: args.repository.getProducts,
      onSuccess: (products) {
        emit(
          state.copyWith(products: products, status: ProductsStatus.success),
        );
        logger.info(
          'PRODUCTS',
          'Loaded ${products.length} products successfully',
        );
      },
      onError: (error, _) {
        logger.error('PRODUCTS', 'Failed to load products', error);
        emit(
          state.copyWith(
            status: ProductsStatus.error,
            errorMessage: error.toString(),
          ),
        );
      },
    );
  }

  /// Selects category.
  void selectCategory(String? category) {
    logger.userAction('select_category', {'category': category ?? 'all'});
    emit(state.copyWith(selectedCategory: category));
    logger.debug(
      'PRODUCTS',
      'Filtered to category: ${category ?? 'all'}, showing ${state.filteredProducts.length} products',
    );
  }

  /// Searches products.
  void search(String query) {
    logger.userAction('search_products', {'query': query});
    emit(state.copyWith(searchQuery: query));
  }

  /// Sorts products.
  void sort(ProductSortOption option) {
    logger.userAction('sort_products', {'option': option.name});
    emit(state.copyWith(sortOption: option));
  }
}
