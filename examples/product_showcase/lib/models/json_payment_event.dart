import 'package:dust_dart/serde.dart';

part 'json_payment_event.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(tag: 'type', renameAll: SerDeRename.snakeCase)
sealed class JsonPaymentEvent with _$JsonPaymentEvent {
  const JsonPaymentEvent();

  factory JsonPaymentEvent.fromJson(Map<String, Object?> json) =>
      _$JsonPaymentEventFromJson(json);

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
