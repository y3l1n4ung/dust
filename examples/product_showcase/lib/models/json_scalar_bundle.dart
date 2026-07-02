import 'package:dust_dart/serde.dart';

part 'json_scalar_bundle.g.dart';

/// JSON scalar bundle model for the product showcase example.
@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonScalarBundle with _$JsonScalarBundle {
  /// Creates a [JsonScalarBundle].
  const JsonScalarBundle({
    required this.createdAt,
    this.updatedAt,
    required this.website,
    required this.largeNumber,
    required this.endpoints,
    required this.checkpoints,
  });

  /// Creates a [JsonScalarBundle] from JSON.
  factory JsonScalarBundle.fromJson(Map<String, Object?> json) =>
      _$JsonScalarBundleFromJson(json);

  /// Created at.
  final DateTime createdAt;

  /// Updated at.
  final DateTime? updatedAt;

  /// Website.
  final Uri website;

  /// Large number.
  final BigInt largeNumber;

  /// Endpoints.
  final Set<Uri> endpoints;

  /// Checkpoints.
  final Map<String, DateTime> checkpoints;
}
