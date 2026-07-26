use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{DustImport, generated_output, make_workspace, write_dust_file};

#[test]
fn build_writes_custom_serde_codec_outputs() {
    let workspace = make_workspace();
    write_dust_file(
        &workspace.path().join("lib/audit.dart"),
        &[DustImport::Derive],
        "part 'audit.g.dart';\n\
         final class UnixEpochDateTimeCodec implements SerDeCodec<DateTime, int> {\n\
           const UnixEpochDateTimeCodec();\n\
           @override\n\
           int serialize(DateTime value) => value.millisecondsSinceEpoch;\n\
           @override\n\
           DateTime deserialize(int value) => DateTime.fromMillisecondsSinceEpoch(value, isUtc: true);\n\
         }\n\
         const unixEpochDateTimeCodec = UnixEpochDateTimeCodec();\n\
         @Derive([Serialize(), Deserialize()])\n\
         class Audit {\n\
           const Audit({required this.createdAt, this.updatedAt});\n\
           @SerDe(using: unixEpochDateTimeCodec)\n\
           final DateTime createdAt;\n\
           @SerDe(using: unixEpochDateTimeCodec)\n\
           final DateTime? updatedAt;\n\
           factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    let output = fs::read_to_string(workspace.path().join("lib/audit.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'audit.dart';

mixin _$Audit implements Serializable {
  Map<String, Object?> serialize() => _$AuditSerialize(this as Audit);

  Map<String, Object?> toJson() => serialize();
}

final class $AuditSerializer implements Serializer<Audit, Map<String, Object?>> {
  const $AuditSerializer();

  @override
  Map<String, Object?> serialize(Audit value) => _$AuditSerialize(value);
}
final class $AuditDeserializer implements Deserializer<Audit, Map<String, Object?>> {
  const $AuditDeserializer();

  @override
  Audit deserialize(Map<String, Object?> json) => _$AuditDeserialize(json);
}

Map<String, Object?> _$AuditSerialize(Audit instance) {
  return <String, Object?>{
    'createdAt': JsonHelper.encodeWithCodec<DateTime, Object?>(
      unixEpochDateTimeCodec,
      instance.createdAt,
    ),
    'updatedAt': instance.updatedAt == null
        ? null
        : JsonHelper.encodeWithCodec<DateTime, Object?>(unixEpochDateTimeCodec, instance.updatedAt!),
  };
}

Map<String, Object?> _$AuditToJson(Audit instance) =>
    _$AuditSerialize(instance);

// factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);
Audit _$AuditDeserialize(Map<String, Object?> json) {
  final createdAtValue = JsonHelper.decodeWithCodec<DateTime, Object?>(
    unixEpochDateTimeCodec,
    json['createdAt'],
    'createdAt',
  );
  final updatedAtValue = json['updatedAt'] == null
      ? null
      : JsonHelper.decodeWithCodec<DateTime, Object?>(unixEpochDateTimeCodec, json['updatedAt'], 'updatedAt');

  return Audit(createdAt: createdAtValue, updatedAt: updatedAtValue);
}

Audit _$AuditFromJson(Map<String, Object?> json) =>
    _$AuditDeserialize(json);
"#
        )
    );
}
