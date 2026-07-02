import 'package:dust_dart/serde.dart';

import 'json_profile.dart';

part 'json_codec_bundle.g.dart';

/// JSON page.
final class JsonPage<T> {
  /// Creates a [JsonPage].
  const JsonPage({required this.items, required this.total});

  /// Items.
  final List<T> items;

  /// Total.
  final int total;
}

/// Unix epoch date time codec model for the product showcase example.
final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {
  /// Creates an [UnixEpochDateTimeCodec].
  const UnixEpochDateTimeCodec();

  @override
  int serialize(DateTime value) => value.millisecondsSinceEpoch;

  @override
  DateTime deserialize(int value) =>
      DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);
}

/// JSON profile page codec for the product showcase example.
final class JsonProfilePageCodec
    implements SerDeCodec<JsonPage<JsonProfile>, Map<String, Object?>> {
  /// Creates a [JsonProfilePageCodec].
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

/// Unix epoch date time codec.
const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();

/// JSON profile page codec.
const jsonProfilePageCodec = JsonProfilePageCodec();

/// JSON codec bundle model for the product showcase example.
@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonCodecBundle with _$JsonCodecBundle {
  /// Creates a [JsonCodecBundle].
  const JsonCodecBundle({
    required this.createdAt,
    this.updatedAt,
    required this.profiles,
  });

  /// Creates a [JsonCodecBundle] from JSON.
  factory JsonCodecBundle.fromJson(Map<String, Object?> json) =>
      _$JsonCodecBundleFromJson(json);

  /// Created at.
  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime createdAt;

  /// Updated at.
  @SerDe(using: unixEpochDateTimeCodec)
  final DateTime? updatedAt;

  /// Profiles.
  @SerDe(using: jsonProfilePageCodec)
  final JsonPage<JsonProfile> profiles;
}
