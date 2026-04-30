import 'package:derive_serde_annotation/derive_serde_annotation.dart';

import 'json_profile.dart';

part 'json_account.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class JsonAccount with _$JsonAccountDust {
  const JsonAccount({
    required this.profile,
    required this.metrics,
    required this.archived,
  });

  factory JsonAccount.fromJson(Map<String, Object?> json) =>
      _$JsonAccountFromJson(json);

  final JsonProfile profile;
  final Map<String, List<int>> metrics;
  final bool archived;
}
