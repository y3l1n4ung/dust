import 'package:dust_dart/serde.dart';

part 'order_tracking.g.dart';

/// Tracking event model for the shopping app example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class TrackingEvent with _$TrackingEvent {
  /// Creates a [TrackingEvent].
  const TrackingEvent({
    required this.id,
    required this.orderId,
    required this.title,
    required this.description,
    required this.location,
    required this.occurredAt,
    required this.completed,
  });

  /// Unique identifier.
  final String id;

  /// Order ID.
  final String orderId;

  /// Title.
  final String title;

  /// Description.
  final String description;

  /// Location.
  final String location;

  /// Occurred at.
  final DateTime occurredAt;

  /// Completed.
  final bool completed;

  /// Creates a [TrackingEvent] from JSON.
  factory TrackingEvent.fromJson(Map<String, Object?> json) =>
      _$TrackingEventFromJson(json);
}
