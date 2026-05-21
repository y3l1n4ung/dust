import 'product.dart';

enum ProductsStatus { initial, loading, success, error }

class ProductsState {
  final List<Product> products;
  final ProductsStatus status;
  final String? errorMessage;
  final String? selectedCategory;

  const ProductsState({
    this.products = const [],
    this.status = ProductsStatus.initial,
    this.errorMessage,
    this.selectedCategory,
  });

  List<Product> get filteredProducts {
    if (selectedCategory == null || selectedCategory == 'all') {
      return products;
    }
    return products.where((p) => p.category == selectedCategory).toList();
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
  }) {
    return ProductsState(
      products: products ?? this.products,
      status: status ?? this.status,
      errorMessage: errorMessage ?? this.errorMessage,
      selectedCategory: selectedCategory ?? this.selectedCategory,
    );
  }
}
