import 'package:dust_dart/serde.dart';

/// Audit stamp.
mixin AuditStamp {
  /// Returns the audit label.
  String auditLabel() => 'audited';
}

/// Generated node model for the benchmark example.
class GeneratedNode {
  /// Creates a [GeneratedNode].
  const GeneratedNode();
}

/// Shared status values for the benchmark example.
enum SharedStatus {
  /// Active shared status.
  active,

  /// Paused shared status.
  paused,

  /// Archived shared status.
  archived,
}

/// Unix epoch date time codec model for the benchmark example.
final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {
  /// Creates an [UnixEpochDateTimeCodec].
  const UnixEpochDateTimeCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) =>
      DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);
}

/// Shared status index codec for the benchmark example.
final class SharedStatusIndexCodec implements SerDeCodec<SharedStatus, int> {
  /// Creates a [SharedStatusIndexCodec].
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

/// Unix epoch date time codec.
const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();

/// Shared status index codec.
const sharedStatusIndexCodec = SharedStatusIndexCodec();
