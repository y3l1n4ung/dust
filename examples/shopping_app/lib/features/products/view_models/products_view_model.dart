import 'package:dust_state/dust_state.dart';

import '../../../core/data/shopping_repository.dart';
import '../../../core/logging/logger.dart';
import '../models/products_state.dart';

part 'products_view_model.g.dart';

final class ProductsViewModelArgs extends ViewModelArgs {
  const ProductsViewModelArgs({required this.repository, super.observer});

  final ShoppingRepository repository;
}

@ViewModel(state: ProductsState, args: ProductsViewModelArgs)
class ProductsViewModel extends $ProductsViewModel {
  ProductsViewModel(super.args);

  @override
  Future<void> onInit() => loadProducts();

  Future<void> loadProducts() async {
    logger.info('PRODUCTS', 'Loading products...');
    emit(state.copyWith(status: ProductsStatus.loading));

    try {
      final products = await args.repository.getProducts();
      emit(state.copyWith(products: products, status: ProductsStatus.success));
      logger.info(
        'PRODUCTS',
        'Loaded ${products.length} products successfully',
      );
    } catch (e) {
      logger.error('PRODUCTS', 'Failed to load products', e);
      emit(
        state.copyWith(
          status: ProductsStatus.error,
          errorMessage: e.toString(),
        ),
      );
    }
  }

  void selectCategory(String? category) {
    logger.userAction('select_category', {'category': category ?? 'all'});
    emit(state.copyWith(selectedCategory: category));
    logger.debug(
      'PRODUCTS',
      'Filtered to category: ${category ?? 'all'}, showing ${state.filteredProducts.length} products',
    );
  }

  void search(String query) {
    logger.userAction('search_products', {'query': query});
    emit(state.copyWith(searchQuery: query));
  }

  void sort(ProductSortOption option) {
    logger.userAction('sort_products', {'option': option.name});
    emit(state.copyWith(sortOption: option));
  }
}
