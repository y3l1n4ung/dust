import 'package:dust_flutter/state.dart';

import '../../../core/data/shopping_repository.dart';
import '../models/order_tracking_state.dart';

part 'order_tracking_view_model.g.dart';

/// Order tracking view model args model for the shopping app example.
final class OrderTrackingViewModelArgs extends ViewModelArgs {
  /// Creates an [OrderTrackingViewModelArgs].
  const OrderTrackingViewModelArgs({required this.repository, super.observer});

  /// Repository.
  final ShoppingRepository repository;
}

/// Order tracking view model for the shopping app example.
@ViewModel(state: OrderTrackingState, args: OrderTrackingViewModelArgs)
class OrderTrackingViewModel extends $OrderTrackingViewModel {
  /// Creates an [OrderTrackingViewModel].
  OrderTrackingViewModel(super.args);

  /// Loads.
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
