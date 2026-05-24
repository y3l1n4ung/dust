import '../../products/models/product.dart';
import 'product_review.dart';

enum ProductDetailStatus { initial, loading, success, error }

class ProductDetailState {
  const ProductDetailState({
    this.productId,
    this.status = ProductDetailStatus.initial,
    this.reviews = const [],
    this.recommendations = const [],
    this.errorMessage,
  });

  final int? productId;
  final ProductDetailStatus status;
  final List<ProductReview> reviews;
  final List<Product> recommendations;
  final String? errorMessage;

  ProductDetailState copyWith({
    int? productId,
    ProductDetailStatus? status,
    List<ProductReview>? reviews,
    List<Product>? recommendations,
    String? errorMessage,
  }) {
    return ProductDetailState(
      productId: productId ?? this.productId,
      status: status ?? this.status,
      reviews: reviews ?? this.reviews,
      recommendations: recommendations ?? this.recommendations,
      errorMessage: errorMessage,
    );
  }
}
