import 'package:dust_dart/derive.dart';
import '../../products/models/product.dart';
import 'product_review.dart';

part 'product_detail_state.g.dart';

/// Product detail status values for the shopping app example.
enum ProductDetailStatus {
  /// Initial product detail status.
  initial,

  /// Loading product detail status.
  loading,

  /// Success product detail status.
  success,

  /// Error product detail status.
  error,
}

/// Product detail state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class ProductDetailState with _$ProductDetailState {
  /// Creates a [ProductDetailState].
  const ProductDetailState({
    this.productId,
    this.status = ProductDetailStatus.initial,
    this.reviews = const [],
    this.recommendations = const [],
    this.errorMessage,
  });

  /// Product ID.
  final int? productId;

  /// Status.
  final ProductDetailStatus status;

  /// Reviews.
  final List<ProductReview> reviews;

  /// Recommendations.
  final List<Product> recommendations;

  /// Error message.
  final String? errorMessage;
}
