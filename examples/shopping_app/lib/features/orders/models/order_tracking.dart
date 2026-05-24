import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'order_tracking.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class TrackingEvent with _$TrackingEvent {
  const TrackingEvent({
    required this.id,
    required this.orderId,
    required this.title,
    required this.description,
    required this.location,
    required this.occurredAt,
    required this.completed,
  });

  final String id;
  final String orderId;
  final String title;
  final String description;
  final String location;
  final DateTime occurredAt;
  final bool completed;

  factory TrackingEvent.fromJson(Map<String, Object?> json) =>
      _$TrackingEventFromJson(json);
}
