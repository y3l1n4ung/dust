import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'json_codec_bundle.g.dart';

final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {
  const UnixEpochDateTimeCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) =>
      DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);
}

const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonCodecBundle with _$JsonCodecBundleDust {
  const JsonCodecBundle({required this.createdAt, this.updatedAt});

  factory JsonCodecBundle.fromJson(Map<String, Object?> json) =>
      _$JsonCodecBundleFromJson(json);

  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime? updatedAt;
}
