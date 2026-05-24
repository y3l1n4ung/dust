import 'dart:convert';

import 'package:dust_state/dust_state.dart';

import '../../../core/services/storage_service.dart';
import '../../products/models/product.dart';
import '../models/wishlist_item.dart';
import '../models/wishlist_state.dart';

part 'wishlist_view_model.g.dart';

final class WishlistViewModelArgs extends ViewModelArgs {
  const WishlistViewModelArgs({required this.storage, super.observer});

  final StorageService storage;
}

class WishlistEffect {
  const WishlistEffect(this.message);

  final String message;
}

@ViewModel(state: WishlistState, args: WishlistViewModelArgs)
class WishlistViewModel extends $WishlistViewModel {
  WishlistViewModel(super.args);

  static const _storageKey = 'wishlist_items';

  @override
  Future<void> onInit() => loadWishlist();

  Future<void> loadWishlist() async {
    emit(state.copyWith(isLoading: true));
    final raw = storage.getString(_storageKey);
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

  Future<void> remove(int productId) async {
    final nextItems = state.items
        .where((item) => item.product.id != productId)
        .toList(growable: false);
    emit(state.copyWith(items: nextItems));
    await _persist(nextItems);
  }

  Future<void> _persist(List<WishlistItem> items) {
    final encoded = jsonEncode(items.map((item) => item.toJson()).toList());
    return storage.setString(_storageKey, encoded);
  }
}
