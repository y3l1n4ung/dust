use std::fs;

use dust_driver::{BuildRequest, run_build};

use crate::support::{DustImport, generated_output, make_workspace, write_dust_file, write_file};

#[test]
fn build_accepts_supported_dart_version_surfaces_while_generating_outputs() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("lib/legacy_null_safety.dart"),
        "// @dart=2.12\n\
         \n\
         class LegacyNullSafetyUser {\n\
           const LegacyNullSafetyUser({required this.id, this.nickname});\n\
           final String id;\n\
           final String? nickname;\n\
         }\n\
         \n\
         typedef LegacyMapper = Map<String, List<int?>>;\n",
    );
    write_file(
        &workspace.path().join("lib/modern_surfaces.dart"),
        "typedef Location = (double lat, double lng);\n\
         \n\
         sealed class Shape {\n\
           const Shape();\n\
         }\n\
         \n\
         final class Circle extends Shape {\n\
           const Circle(this.center);\n\
           final Location center;\n\
         }\n\
         \n\
         extension type UserId(String value) {\n\
           bool get isEmpty => value.isEmpty;\n\
         }\n\
         \n\
         const String? maybeLabel = null;\n\
         const List<String>? maybeAliases = null;\n\
         final labels = ['stable', ?maybeLabel, ...?maybeAliases];\n",
    );
    write_file(
        &workspace.path().join("lib/compat_model.dart"),
        "import 'package:dust_dart/derive.dart';\n\
         \n\
         import 'legacy_null_safety.dart';\n\
         import 'modern_surfaces.dart';\n\
         \n\
         part 'compat_model.g.dart';\n\
         \n\
         @Derive([ToString()])\n\
         class CompatProfile {\n\
           const CompatProfile({\n\
             required this.id,\n\
             required this.displayName,\n\
             this.nickname,\n\
             required this.location,\n\
           });\n\
         \n\
           final String id;\n\
           final String displayName;\n\
           final String? nickname;\n\
           final Location location;\n\
         \n\
           LegacyMapper get legacyMapper => const {};\n\
           UserId get userId => UserId(id);\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    let output = fs::read_to_string(workspace.path().join("lib/compat_model.g.dart")).unwrap();

    assert!(!result.has_errors(), "{:?}", result.diagnostics);
    assert_eq!(result.build_artifacts.len(), 1);
    assert_eq!(
        output,
        generated_output(
            r#"part of 'compat_model.dart';

mixin _$CompatProfile {
  @override
  String toString() {
    final self = this as CompatProfile;
    return 'CompatProfile('
        'id: ${self.id}, '
        'displayName: ${self.displayName}, '
        'nickname: ${self.nickname}, '
        'location: ${self.location}'
        ')';
  }
}
"#
        )
    );
}

#[test]
fn build_uses_pubspec_sdk_lower_bound_for_language_gates() {
    let workspace = make_workspace();
    write_file(
        &workspace.path().join("pubspec.yaml"),
        "name: dust_test\nenvironment:\n  sdk: '>=3.0.0 <4.0.0'\n",
    );
    write_dust_file(
        &workspace.path().join("lib/private_param.dart"),
        &[DustImport::Derive],
        "part 'private_param.g.dart';\n\
         \n\
         @Derive([ToString()])\n\
         class PrivateNamedParameter {\n\
           const PrivateNamedParameter({String? _traceId});\n\
         }\n",
    );

    let result = run_build(BuildRequest {
        cwd: workspace.path().to_path_buf(),
        fail_fast: false,
        jobs: None,
        db: Default::default(),
    });

    assert!(result.has_errors());
    assert!(result.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("private named parameters require Dart 3.12")
    }));
    assert_eq!(result.diagnostic_files.len(), 1);
}
