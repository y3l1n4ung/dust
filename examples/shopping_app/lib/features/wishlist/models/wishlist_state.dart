import 'package:derive_annotation/derive_annotation.dart';
import 'wishlist_item.dart';

part 'wishlist_state.g.dart';

@Derive([ToString(), CopyWith(), Eq()])
class WishlistState with _$WishlistState {
  const WishlistState({this.items = const [], this.isLoading = false});

  final List<WishlistItem> items;
  final bool isLoading;

  bool containsProduct(int productId) =>
      items.any((item) => item.product.id == productId);
}
