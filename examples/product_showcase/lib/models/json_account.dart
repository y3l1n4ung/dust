import 'package:dust_dart/serde.dart';

import 'json_profile.dart';

part 'json_account.g.dart';

/// JSON account model for the product showcase example.
@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class JsonAccount with _$JsonAccount {
  /// Creates a [JsonAccount].
  const JsonAccount({
    required this.profile,
    required this.metrics,
    required this.archived,
  });

  /// Creates a [JsonAccount] from JSON.
  factory JsonAccount.fromJson(Map<String, Object?> json) =>
      _$JsonAccountFromJson(json);

  /// Profile.
  final JsonProfile profile;

  /// Metrics.
  final Map<String, List<int>> metrics;

  /// Archived.
  final bool archived;
}
