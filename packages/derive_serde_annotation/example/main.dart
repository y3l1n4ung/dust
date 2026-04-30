import 'package:derive_serde_annotation/derive_serde_annotation.dart';

@Derive([Serialize(), Deserialize()])
@SerDe(renameAll: SerDeRename.snakeCase)
class AuditLog {
  @SerDe(rename: 'created_at')
  final String createdAt;

  const AuditLog(this.createdAt);
}

void main() {
  const log = AuditLog('2026-04-30T00:00:00Z');
  print(log.createdAt);
}
