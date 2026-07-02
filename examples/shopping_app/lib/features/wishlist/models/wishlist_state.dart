import 'package:dust_dart/derive.dart';
import 'wishlist_item.dart';

part 'wishlist_state.g.dart';

/// Wishlist state for the shopping app example.
@Derive([ToString(), CopyWith(), Eq()])
class WishlistState with _$WishlistState {
  /// Creates a [WishlistState].
  const WishlistState({this.items = const [], this.isLoading = false});

  /// Items.
  final List<WishlistItem> items;

  /// Is loading.
  final bool isLoading;

  /// Checks whether the product is present.
  bool containsProduct(int productId) =>
      items.any((item) => item.product.id == productId);
}
