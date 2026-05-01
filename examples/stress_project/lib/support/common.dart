import 'package:derive_serde_annotation/derive_serde_annotation.dart';

mixin AuditStamp {
  String auditLabel() => 'audited';
}

class GeneratedNode {
  const GeneratedNode();
}

enum SharedStatus { active, paused, archived }

final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {
  const UnixEpochDateTimeCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) =>
      DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);
}

final class SharedStatusIndexCodec implements SerDeCodec<SharedStatus, int> {
  const SharedStatusIndexCodec();

  @override
  int serialize(SharedStatus value) => switch (value) {
    SharedStatus.active => 0,
    SharedStatus.paused => 1,
    SharedStatus.archived => 2,
  };

  @override
  SharedStatus deserialize(int value) => switch (value) {
    0 => SharedStatus.active,
    1 => SharedStatus.paused,
    2 => SharedStatus.archived,
    _ => throw ArgumentError.value(
      value,
      'value',
      'unknown SharedStatus index',
    ),
  };
}

const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();
const sharedStatusIndexCodec = SharedStatusIndexCodec();
