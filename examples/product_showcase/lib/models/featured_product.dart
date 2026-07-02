import 'package:dust_dart/derive.dart';

import 'audit.dart';
import 'price.dart';

part 'featured_product.g.dart';

/// Featured product model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith()])
class FeaturedProduct extends CatalogNode with AuditStamp, _$FeaturedProduct {
  /// Creates a [FeaturedProduct].
  const FeaturedProduct({
    required this.sku,
    required this.price,
    required this.tags,
    this.archived = false,
  });

  /// SKU.
  final String sku;

  /// Price.
  final Price price;

  /// Tags.
  final Set<String> tags;

  /// Archived.
  final bool archived;
}
