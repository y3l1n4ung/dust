import 'package:dust_dart/serde.dart';

import 'json_profile.dart';

part 'json_account.g.dart';

@Derive([ToString(), Eq(), CopyWith(), Serialize(), Deserialize()])
class JsonAccount with _$JsonAccount {
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
