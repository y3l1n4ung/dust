import 'package:dust_dart/serde.dart';

import '../../products/models/product.dart';

part 'wishlist_item.g.dart';

/// Wishlist item model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class WishlistItem with _$WishlistItem {
  /// Creates a [WishlistItem].
  const WishlistItem({required this.product, required this.savedAt});

  /// Product.
  final Product product;

  /// Saved at.
  final DateTime savedAt;

  /// Creates a [WishlistItem] from JSON.
  factory WishlistItem.fromJson(Map<String, Object?> json) =>
      _$WishlistItemFromJson(json);
}
