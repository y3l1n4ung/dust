import 'package:derive_serde_annotation/derive_serde_annotation.dart';

part 'json_scalar_bundle.g.dart';

@Derive([ToString(), Eq(), Serialize(), Deserialize()])
class JsonScalarBundle with _$JsonScalarBundleDust {
  const JsonScalarBundle({
    required this.createdAt,
    this.updatedAt,
    required this.website,
    required this.largeNumber,
    required this.endpoints,
    required this.checkpoints,
  });

  factory JsonScalarBundle.fromJson(Map<String, Object?> json) =>
      _$JsonScalarBundleFromJson(json);

  final DateTime createdAt;
  final DateTime? updatedAt;
  final Uri website;
  final BigInt largeNumber;
  final Set<Uri> endpoints;
  final Map<String, DateTime> checkpoints;
}
