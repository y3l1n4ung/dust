import 'dart:convert';

import 'package:dust_flutter/state.dart';

import '../../../core/services/storage_service.dart';
import '../../products/models/product.dart';
import '../models/wishlist_item.dart';
import '../models/wishlist_state.dart';

part 'wishlist_view_model.g.dart';

/// Wishlist view model args model for the shopping app example.
final class WishlistViewModelArgs extends ViewModelArgs {
  /// Creates a [WishlistViewModelArgs].
  const WishlistViewModelArgs({required this.storage, super.observer});

  /// Storage.
  final StorageService storage;
}

/// Wishlist effect model for the shopping app example.
class WishlistEffect {
  /// Creates a [WishlistEffect].
  const WishlistEffect(this.message);

  /// Message.
  final String message;
}

/// Wishlist view model for the shopping app example.
@ViewModel(state: WishlistState, args: WishlistViewModelArgs)
class WishlistViewModel extends $WishlistViewModel {
  /// Creates a [WishlistViewModel].
  WishlistViewModel(super.args);

  static const _storageKey = 'wishlist_items';

  @override
  Future<void> onInit() => loadWishlist();

  /// Loads wishlist.
  Future<void> loadWishlist() async {
    emit(state.copyWith(isLoading: true));
    final raw = args.storage.getString(_storageKey);
    if (raw == null || raw.isEmpty) {
      emit(const WishlistState());
      return;
    }

    try {
      final decoded = jsonDecode(raw) as List<dynamic>;
      final items = decoded
          .map(
            (item) => WishlistItem.fromJson(
              Map<String, Object?>.from(item as Map<dynamic, dynamic>),
            ),
          )
          .toList();
      emit(WishlistState(items: items));
    } catch (_) {
      emit(const WishlistState());
    }
  }

  /// Toggles.
  Future<void> toggle(Product product) async {
    final exists = state.containsProduct(product.id);
    final nextItems = exists
        ? state.items
            .where((item) => item.product.id != product.id)
            .toList(growable: false)
        : [
            WishlistItem(product: product, savedAt: DateTime.now()),
            ...state.items,
          ];

    emit(state.copyWith(items: nextItems, isLoading: false));
    await _persist(nextItems);
    emitEffect(
      WishlistEffect(
        exists
            ? '${product.title} removed from wishlist'
            : '${product.title} saved to wishlist',
      ),
    );
  }

  /// Removes.
  Future<void> remove(int productId) async {
    final nextItems = state.items
        .where((item) => item.product.id != productId)
        .toList(growable: false);
    emit(state.copyWith(items: nextItems));
    await _persist(nextItems);
  }

  Future<void> _persist(List<WishlistItem> items) {
    final encoded = jsonEncode(items.map((item) => item.toJson()).toList());
    return args.storage.setString(_storageKey, encoded);
  }
}
