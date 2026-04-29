import 'package:derive_annotation/derive_annotation.dart';

import 'audit.dart';
import 'price.dart';

part 'featured_product.g.dart';

@Derive([ToString(), Eq(), CopyWith()])
class FeaturedProduct extends CatalogNode with AuditStamp, _$FeaturedProductDust {
  const FeaturedProduct({
    required this.sku,
    required this.price,
    required this.tags,
    this.archived = false,
  });

  final String sku;
  final Price price;
  final Set<String> tags;
  final bool archived;
}
