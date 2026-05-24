import 'order_tracking.dart';

enum OrderTrackingStatus { initial, loading, success, error }

class OrderTrackingState {
  const OrderTrackingState({
    this.orderId,
    this.status = OrderTrackingStatus.initial,
    this.events = const [],
    this.errorMessage,
  });

  final String? orderId;
  final OrderTrackingStatus status;
  final List<TrackingEvent> events;
  final String? errorMessage;

  OrderTrackingState copyWith({
    String? orderId,
    OrderTrackingStatus? status,
    List<TrackingEvent>? events,
    String? errorMessage,
  }) {
    return OrderTrackingState(
      orderId: orderId ?? this.orderId,
      status: status ?? this.status,
      events: events ?? this.events,
      errorMessage: errorMessage,
    );
  }
}
