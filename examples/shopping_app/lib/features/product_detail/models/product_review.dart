import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'product_review.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ProductReview with _$ProductReview {
  const ProductReview({
    required this.id,
    required this.productId,
    required this.authorName,
    required this.rating,
    required this.comment,
    required this.createdAt,
    required this.verifiedPurchase,
  });

  final String id;
  final int productId;
  final String authorName;
  final double rating;
  final String comment;
  final DateTime createdAt;
  final bool verifiedPurchase;

  factory ProductReview.fromJson(Map<String, Object?> json) =>
      _$ProductReviewFromJson(json);
}
