import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'product.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Product with _$Product {
  const Product({
    required this.id,
    required this.title,
    required this.price,
    required this.description,
    required this.category,
    required this.image,
    required this.rating,
  });

  final int id;
  final String title;
  final double price;
  final String description;
  final String category;
  final String image;
  final Rating rating;

  factory Product.fromJson(Map<String, Object?> json) =>
      _$ProductFromJson(json);
}

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class Rating with _$Rating {
  const Rating({required this.rate, required this.count});

  final double rate;
  final int count;

  factory Rating.fromJson(Map<String, Object?> json) => _$RatingFromJson(json);
}
