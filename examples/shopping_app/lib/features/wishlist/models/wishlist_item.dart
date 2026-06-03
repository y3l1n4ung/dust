import 'package:dust_dart/serde.dart';

import '../../products/models/product.dart';

part 'wishlist_item.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class WishlistItem with _$WishlistItem {
  const WishlistItem({required this.product, required this.savedAt});

  final Product product;
  final DateTime savedAt;

  factory WishlistItem.fromJson(Map<String, Object?> json) =>
      _$WishlistItemFromJson(json);
}
