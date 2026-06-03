import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../models/demo_cart_state.dart';

part 'demo_cart_api_view_model.g.dart';

final class DemoCartApiViewModelArgs extends ViewModelArgs {
  const DemoCartApiViewModelArgs({required this.repository, super.observer});

  final ShoppingRepository repository;
}

@ViewModel(state: DemoCartState, args: DemoCartApiViewModelArgs)
class DemoCartApiViewModel extends $DemoCartApiViewModel {
  DemoCartApiViewModel(super.args);

  @override
  Future<void> onInit() => loadUserCarts(1);

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
