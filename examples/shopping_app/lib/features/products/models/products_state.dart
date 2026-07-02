import 'package:dust_dart/derive.dart';
import 'product.dart';

part 'products_state.g.dart';

/// Products status values for the shopping app example.
enum ProductsStatus {
  /// Initial products status.
  initial,

  /// Loading products status.
  loading,

  /// Success products status.
  success,

  /// Error products status.
  error,
}

/// Product sort option values for the shopping app example.
enum ProductSortOption {
  /// Featured product sort option.
  featured,

  /// Price low product sort option.
  priceLow,

  /// Price high product sort option.
  priceHigh,

  /// Rating high product sort option.
  ratingHigh,
}

/// Products state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class ProductsState with _$ProductsState {
  /// Products.
  final List<Product> products;

  /// Status.
  final ProductsStatus status;

  /// Error message.
  final String? errorMessage;

  /// Selected category.
  final String? selectedCategory;

  /// Search query.
  final String searchQuery;

  /// Sort option.
  final ProductSortOption sortOption;

  /// Creates a [ProductsState].
  const ProductsState({
    this.products = const [],
    this.status = ProductsStatus.initial,
    this.errorMessage,
    this.selectedCategory,
    this.searchQuery = '',
    this.sortOption = ProductSortOption.featured,
  });

  /// Filtered products.
  List<Product> get filteredProducts {
    var result = selectedCategory == null || selectedCategory == 'all'
        ? products
        : products.where((p) => p.category == selectedCategory).toList();

    final query = searchQuery.trim().toLowerCase();
    if (query.isNotEmpty) {
      result = result.where((product) {
        return product.title.toLowerCase().contains(query) ||
            product.description.toLowerCase().contains(query) ||
            product.category.toLowerCase().contains(query);
      }).toList();
    }

    result = [...result];
    switch (sortOption) {
      case ProductSortOption.featured:
        break;
      case ProductSortOption.priceLow:
        result.sort((a, b) => a.price.compareTo(b.price));
      case ProductSortOption.priceHigh:
        result.sort((a, b) => b.price.compareTo(a.price));
      case ProductSortOption.ratingHigh:
        result.sort((a, b) => b.rating.rate.compareTo(a.rating.rate));
    }
    return result;
  }

  /// Categories.
  List<String> get categories {
    final cats = products.map((p) => p.category).toSet().toList();
    cats.sort();
    return ['all', ...cats];
  }
}
