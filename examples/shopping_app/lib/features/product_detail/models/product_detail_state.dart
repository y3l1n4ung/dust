import 'package:derive_annotation/derive_annotation.dart';
import '../../products/models/product.dart';
import 'product_review.dart';

part 'product_detail_state.g.dart';

enum ProductDetailStatus { initial, loading, success, error }

@Derive([ToString(), CopyWith(), Eq()])
class ProductDetailState with _$ProductDetailState {
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
}
