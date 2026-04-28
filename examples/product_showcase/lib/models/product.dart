import 'package:derive_annotation/derive_annotation.dart';

import 'category.dart';
import 'price.dart';

part 'product.g.dart';

@Derive([Debug(), Eq(), CopyWith()])
class Product with _$ProductDust {
  const Product({
    required this.sku,
    required this.title,
    required this.price,
    required this.categories,
    required this.attributes,
    this.featured = false,
  });

  final String sku;
  final String title;
  final Price price;
  final List<Category> categories;
  final Map<String, String> attributes;
  final bool featured;
}
