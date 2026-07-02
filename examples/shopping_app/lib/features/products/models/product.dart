import 'package:dust_dart/serde.dart';

import '../../../core/i18n/shop_i18n_keys.dart';

part 'product.g.dart';

/// Product model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Product with _$Product {
  /// Creates a [Product].
  const Product({
    required this.id,
    required this.title,
    required this.price,
    required this.description,
    required this.category,
    required this.image,
    required this.rating,
  });

  /// Unique identifier.
  final int id;

  /// Title.
  final String title;

  /// Price.
  final double price;

  /// Description.
  final String description;

  /// Category.
  final String category;

  /// Image.
  final String image;

  /// Rating.
  final Rating rating;

  /// Creates a [Product] from JSON.
  factory Product.fromJson(Map<String, Object?> json) =>
      _$ProductFromJson(json);

  /// Category translation key.
  String get categoryTranslationKey => shopCategoryKey(category);

  /// Category fallback label.
  String get categoryFallbackLabel => category.toUpperCase();

  /// Price label.
  String get priceLabel => price.toStringAsFixed(2);
}

/// Rating model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Rating with _$Rating {
  /// Creates a [Rating].
  const Rating({required this.rate, required this.count});

  /// Rate.
  final double rate;

  /// Count.
  final int count;

  /// Creates a [Rating] from JSON.
  factory Rating.fromJson(Map<String, Object?> json) => _$RatingFromJson(json);

  /// Rate label.
  String get rateLabel => rate.toStringAsFixed(1);
}
