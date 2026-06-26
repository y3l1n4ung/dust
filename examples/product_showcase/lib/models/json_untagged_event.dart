import 'package:dust_dart/serde.dart';

part 'json_untagged_event.g.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(untagged: true)
sealed class JsonUntaggedEvent with _$JsonUntaggedEvent {
  const JsonUntaggedEvent();

  factory JsonUntaggedEvent.fromJson(Map<String, Object?> json) =>
      _$JsonUntaggedEventFromJson(json);

  factory JsonUntaggedEvent.signup({
    required String id,
    required String email,
  }) = JsonSignupEvent;

  factory JsonUntaggedEvent.archive({
    required String id,
    required String reason,
  }) = JsonArchiveEvent;
}
