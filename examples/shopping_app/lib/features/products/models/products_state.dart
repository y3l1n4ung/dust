import 'product.dart';

enum ProductsStatus { initial, loading, success, error }

enum ProductSortOption { featured, priceLow, priceHigh, ratingHigh }

class ProductsState {
  final List<Product> products;
  final ProductsStatus status;
  final String? errorMessage;
  final String? selectedCategory;
  final String searchQuery;
  final ProductSortOption sortOption;

  const ProductsState({
    this.products = const [],
    this.status = ProductsStatus.initial,
    this.errorMessage,
    this.selectedCategory,
    this.searchQuery = '',
    this.sortOption = ProductSortOption.featured,
  });

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

  List<String> get categories {
    final cats = products.map((p) => p.category).toSet().toList();
    cats.sort();
    return ['all', ...cats];
  }

  ProductsState copyWith({
    List<Product>? products,
    ProductsStatus? status,
    String? errorMessage,
    String? selectedCategory,
    String? searchQuery,
    ProductSortOption? sortOption,
  }) {
    return ProductsState(
      products: products ?? this.products,
      status: status ?? this.status,
      errorMessage: errorMessage ?? this.errorMessage,
      selectedCategory: selectedCategory ?? this.selectedCategory,
      searchQuery: searchQuery ?? this.searchQuery,
      sortOption: sortOption ?? this.sortOption,
    );
  }
}
