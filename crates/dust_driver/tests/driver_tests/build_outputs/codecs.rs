use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{generated_output, make_workspace, write_file};

#[test]
fn build_writes_custom_serde_codec_outputs() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/audit.dart"),
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

Never _jsonTypeError(Object? value, String key, String expected) =>
    throw ArgumentError.value(value, key, 'expected $expected');
T _jsonAs<T>(Object? value, String key, String expected) =>
    value is T ? value : _jsonTypeError(value, key, expected);
T _jsonParseString<T>(
  Object? value,
  String key,
  String expected,
  T? Function(String value) parse,
) =>
    parse(_jsonAs<String>(value, key, 'String')) ??
    _jsonTypeError(value, key, expected);
List<Object?> _jsonAsList(Object? value, String key) =>
    _jsonAs<List>(value, key, 'List<Object?>').cast<Object?>();

Map<String, Object?> _jsonAsMap(Object? value, String key) {
  final map = _jsonAs<Map>(value, key, 'Map<String, Object?>');
  try {
    return Map<String, Object?>.from(map);
  } on TypeError {
    _jsonTypeError(value, key, 'Map<String, Object?>');
  }
}

DateTime _jsonAsDateTime(Object? value, String key) =>
    _jsonParseString(value, key, 'ISO-8601 DateTime string', DateTime.tryParse);
Uri _jsonAsUri(Object? value, String key) =>
    _jsonParseString(value, key, 'Uri string', Uri.tryParse);
BigInt _jsonAsBigInt(Object? value, String key) =>
    _jsonParseString(value, key, 'BigInt string', BigInt.tryParse);
T _jsonDecodeWithCodec<T>(dynamic codec, Object? value, String key) {
  if (value == null) {
    throw ArgumentError.value(value, key, 'expected value for SerDeCodec');
  }
  try {
    return codec.deserialize(value as dynamic) as T;
  } catch (error) {
    throw ArgumentError.value(value, key, 'failed SerDeCodec decode: $error');
  }
}

mixin _$Audit {
  Map<String, Object?> toJson() => _$AuditToJson(this as Audit);
}

Map<String, Object?> _$AuditToJson(Audit instance) {
  return <String, Object?>{
    'createdAt': unixEpochDateTimeCodec.serialize(instance.createdAt),
    'updatedAt': instance.updatedAt == null
        ? null
        : unixEpochDateTimeCodec.serialize(instance.updatedAt!),
  };
}
// factory Audit.fromJson(Map<String, Object?> json) => _$AuditFromJson(json);
Audit _$AuditFromJson(Map<String, Object?> json) {
  final createdAtValue = _jsonDecodeWithCodec<DateTime>(
    unixEpochDateTimeCodec,
    json['createdAt'],
    'createdAt',
  );
  final updatedAtValue = json['updatedAt'] == null
      ? null
      : _jsonDecodeWithCodec<DateTime>(unixEpochDateTimeCodec, json['updatedAt'], 'updatedAt');

  return Audit(createdAt: createdAtValue, updatedAt: updatedAtValue);
}
"#
        )
    );
}
