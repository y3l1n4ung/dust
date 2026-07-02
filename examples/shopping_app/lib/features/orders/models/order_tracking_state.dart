import 'order_tracking.dart';

/// Order tracking status values for the shopping app example.
enum OrderTrackingStatus {
  /// Initial order tracking status.
  initial,

  /// Loading order tracking status.
  loading,

  /// Success order tracking status.
  success,

  /// Error order tracking status.
  error,
}

/// Order tracking state for the shopping app example.
class OrderTrackingState {
  /// Creates an [OrderTrackingState].
  const OrderTrackingState({
    this.orderId,
    this.status = OrderTrackingStatus.initial,
    this.events = const [],
    this.errorMessage,
  });

  /// Order ID.
  final String? orderId;

  /// Status.
  final OrderTrackingStatus status;

  /// Events.
  final List<TrackingEvent> events;

  /// Error message.
  final String? errorMessage;

  /// Copy with order tracking status.
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
