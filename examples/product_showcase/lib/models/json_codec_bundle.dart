import 'package:dust_dart/serde.dart';

import 'json_profile.dart';

part 'json_codec_bundle.g.dart';

final class JsonPage<T> {
  const JsonPage({required this.items, required this.total});

  final List<T> items;
  final int total;
}

final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {
  const UnixEpochDateTimeCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) =>
      DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);
}

final class JsonProfilePageCodec
    implements SerDeCodec<JsonPage<JsonProfile>, Map<String, Object?>> {
  const JsonProfilePageCodec();

  @override
  Map<String, Object?> serialize(JsonPage<JsonProfile> value) => {
        'items': value.items.map((item) => item.toJson()).toList(),
        'total': value.total,
      };

  @override
  JsonPage<JsonProfile> deserialize(Map<String, Object?> value) => JsonPage(
        items: JsonHelper.decodeList(
          value['items'],
          'items',
          (item, key) => JsonProfile.fromJson(JsonHelper.asMap(item, key)),
        ),
        total: JsonHelper.as<int>(value['total'], 'total', 'int'),
      );
}

const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();
const jsonProfilePageCodec = JsonProfilePageCodec();

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonCodecBundle with _$JsonCodecBundle {
  const JsonCodecBundle({
    required this.createdAt,
    this.updatedAt,
    required this.profiles,
  });

  factory JsonCodecBundle.fromJson(Map<String, Object?> json) =>
      _$JsonCodecBundleFromJson(json);

  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime? updatedAt;

  @SerDe(using: jsonProfilePageCodec)
  final JsonPage<JsonProfile> profiles;
}
