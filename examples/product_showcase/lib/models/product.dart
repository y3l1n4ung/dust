import 'package:dust_dart/derive.dart';

import 'category.dart';
import 'price.dart';

part 'product.g.dart';

/// Product model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class Product with _$Product {
  /// Creates a [Product].
  const Product({
    required this.sku,
    required this.title,
    required this.price,
    required this.categories,
    required this.attributes,
    this.featured = false,
  });

  /// SKU.
  final String sku;

  /// Title.
  final String title;

  /// Price.
  final Price price;

  /// Categories.
  final List<Category> categories;

  /// Attributes.
  final Map<String, String> attributes;

  /// Featured.
  final bool featured;
}
