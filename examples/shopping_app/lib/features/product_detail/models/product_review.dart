import 'package:dust_dart/serde.dart';

part 'product_review.g.dart';

/// Product review model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class ProductReview with _$ProductReview {
  /// Creates a [ProductReview].
  const ProductReview({
    required this.id,
    required this.productId,
    required this.authorName,
    required this.rating,
    required this.comment,
    required this.createdAt,
    required this.verifiedPurchase,
  });

  /// Unique identifier.
  final String id;

  /// Product ID.
  final int productId;

  /// Author name.
  final String authorName;

  /// Rating.
  final double rating;

  /// Comment.
  final String comment;

  /// Created at.
  final DateTime createdAt;

  /// Verified purchase.
  final bool verifiedPurchase;

  /// Creates a [ProductReview] from JSON.
  factory ProductReview.fromJson(Map<String, Object?> json) =>
      _$ProductReviewFromJson(json);
}
