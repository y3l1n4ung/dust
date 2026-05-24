import 'wishlist_item.dart';

class WishlistState {
  const WishlistState({this.items = const [], this.isLoading = false});

  final List<WishlistItem> items;
  final bool isLoading;

  bool containsProduct(int productId) =>
      items.any((item) => item.product.id == productId);

  WishlistState copyWith({List<WishlistItem>? items, bool? isLoading}) {
    return WishlistState(
      items: items ?? this.items,
      isLoading: isLoading ?? this.isLoading,
    );
  }
}
