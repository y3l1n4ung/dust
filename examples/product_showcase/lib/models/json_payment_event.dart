import 'package:dust_dart/serde.dart';

part 'json_payment_event.g.dart';

@SerDe(tag: 'type', content: 'payload', renameAll: SerDeRename.snakeCase)
sealed class JsonPaymentEvent {
  const JsonPaymentEvent();

  @SerDe(rename: 'payment_success')
  factory JsonPaymentEvent.success({
    required String id,
    required int cents,
    required String currency,
  }) = JsonPaymentSuccess;

  factory JsonPaymentEvent.failed({
    required String id,
    required String reason,
    required bool retryable,
  }) = JsonPaymentFailed;
}

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
final class JsonPaymentSuccess extends JsonPaymentEvent
    with _$JsonPaymentSuccess {
  const JsonPaymentSuccess({
    required this.id,
    required this.cents,
    required this.currency,
  }) : super();

  factory JsonPaymentSuccess.fromJson(Map<String, Object?> json) =>
      _$JsonPaymentSuccessFromJson(json);

  final String id;
  final int cents;
  final String currency;
}

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
final class JsonPaymentFailed extends JsonPaymentEvent
    with _$JsonPaymentFailed {
  const JsonPaymentFailed({
    required this.id,
    required this.reason,
    required this.retryable,
  }) : super();

  factory JsonPaymentFailed.fromJson(Map<String, Object?> json) =>
      _$JsonPaymentFailedFromJson(json);

  final String id;
  final String reason;
  final bool retryable;
}
