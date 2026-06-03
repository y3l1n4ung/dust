import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../models/order_tracking_state.dart';

part 'order_tracking_view_model.g.dart';

final class OrderTrackingViewModelArgs extends ViewModelArgs {
  const OrderTrackingViewModelArgs({required this.repository, super.observer});

  final ShoppingRepository repository;
}

@ViewModel(state: OrderTrackingState, args: OrderTrackingViewModelArgs)
class OrderTrackingViewModel extends $OrderTrackingViewModel {
  OrderTrackingViewModel(super.args);

  Future<void> load(String orderId) async {
    if (state.orderId == orderId &&
        state.status == OrderTrackingStatus.success) {
      return;
    }

    emit(
      state.copyWith(
        orderId: orderId,
        status: OrderTrackingStatus.loading,
        errorMessage: null,
      ),
    );

    try {
      final events = await args.repository.getOrderTracking(orderId);
      emit(
        state.copyWith(
          orderId: orderId,
          status: OrderTrackingStatus.success,
          events: events,
        ),
      );
    } catch (error) {
      emit(
        state.copyWith(
          orderId: orderId,
          status: OrderTrackingStatus.error,
          errorMessage: error.toString(),
        ),
      );
    }
  }
}
