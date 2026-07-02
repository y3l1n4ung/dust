import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../models/demo_cart_state.dart';

part 'demo_cart_api_view_model.g.dart';

/// Demo cart API view model args model for the shopping app example.
final class DemoCartApiViewModelArgs extends ViewModelArgs {
  /// Creates a [DemoCartApiViewModelArgs].
  const DemoCartApiViewModelArgs({required this.repository, super.observer});

  /// Repository.
  final ShoppingRepository repository;
}

/// Demo cart API view model for the shopping app example.
@ViewModel(state: DemoCartState, args: DemoCartApiViewModelArgs)
class DemoCartApiViewModel extends $DemoCartApiViewModel {
  /// Creates a [DemoCartApiViewModel].
  DemoCartApiViewModel(super.args);

  @override
  Future<void> onInit() => loadUserCarts(1);

  /// Loads user carts.
  Future<void> loadUserCarts(int userId) async {
    emit(state.copyWith(status: DemoCartStatus.loading, errorMessage: null));
    try {
      final carts = await args.repository.getUserCarts(userId);
      emit(state.copyWith(status: DemoCartStatus.success, carts: carts));
    } catch (error) {
      emit(
        state.copyWith(
          status: DemoCartStatus.error,
          errorMessage: error.toString(),
        ),
      );
    }
  }
}
