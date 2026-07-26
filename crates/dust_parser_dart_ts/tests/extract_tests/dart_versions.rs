use dust_parser_dart::ParsedTypeKind;

use crate::support::parse;

/// One Dart language-version fixture loaded by parser regression tests.
struct DartVersionFixture {
    /// Human-readable label for test failures.
    label: &'static str,
    /// Dart language version represented by the fixture.
    language_version: &'static str,
    /// Repository-relative fixture path.
    path: &'static str,
    /// Fixture source code.
    source: &'static str,
}

#[test]
fn parses_supported_dart_version_fixtures_without_diagnostics() {
    let fixtures = [
        DartVersionFixture {
            label: "Dart 2.12 null safety",
            language_version: "2.12",
            path: "crates/dust_parser_dart_ts/tests/fixtures/dart_versions/dart_2_12_null_safety.dart",
            source: include_str!("../fixtures/dart_versions/dart_2_12_null_safety.dart"),
        },
        DartVersionFixture {
            label: "Dart 2.17 super parameters",
            language_version: "2.17",
            path: "crates/dust_parser_dart_ts/tests/fixtures/dart_versions/dart_2_17_super_parameters.dart",
            source: include_str!("../fixtures/dart_versions/dart_2_17_super_parameters.dart"),
        },
        DartVersionFixture {
            label: "Dart 3.0 records, patterns, class modifiers",
            language_version: "3.0",
            path: "crates/dust_parser_dart_ts/tests/fixtures/dart_versions/dart_3_0_records_patterns_class_modifiers.dart",
            source: include_str!(
                "../fixtures/dart_versions/dart_3_0_records_patterns_class_modifiers.dart"
            ),
        },
        DartVersionFixture {
            label: "Dart 3.3 extension types",
            language_version: "3.3",
            path: "crates/dust_parser_dart_ts/tests/fixtures/dart_versions/dart_3_3_extension_types.dart",
            source: include_str!("../fixtures/dart_versions/dart_3_3_extension_types.dart"),
        },
        DartVersionFixture {
            label: "Dart 3.8 null-aware collection elements",
            language_version: "3.8",
            path: "crates/dust_parser_dart_ts/tests/fixtures/dart_versions/dart_3_8_null_aware_collections.dart",
            source: include_str!("../fixtures/dart_versions/dart_3_8_null_aware_collections.dart"),
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        let result = parse(100 + index as u32, fixture.source);

        assert!(
            result.diagnostics.is_empty(),
            "{} language={} path={} diagnostics: {:?}",
            fixture.label,
            fixture.language_version,
            fixture.path,
            result.diagnostics
        );
    }
}

#[test]
fn preserves_null_safety_and_record_type_surfaces() {
    let null_safety = parse(
        110,
        include_str!("../fixtures/dart_versions/dart_2_12_null_safety.dart"),
    );
    let user = &null_safety.library.classes[0];
    assert_eq!(user.name, "LegacyNullSafetyUser");
    assert_eq!(user.fields[0].type_source.as_deref(), Some("String"));
    assert_eq!(user.fields[1].type_source.as_deref(), Some("String?"));
    assert!(user.fields[1].parsed_type.as_ref().unwrap().nullable);
    assert_eq!(
        null_safety.library.typedefs[0]
            .aliased_type_source
            .as_deref(),
        Some("Map<String, List<int?>>")
    );

    let records = parse(
        111,
        include_str!("../fixtures/dart_versions/dart_3_0_records_patterns_class_modifiers.dart"),
    );
    let circle = &records.library.classes[1];
    assert_eq!(circle.name, "Circle");
    assert_eq!(
        circle.fields[0].type_source.as_deref(),
        Some("(double x, double y)")
    );
    assert_eq!(
        circle.fields[0].parsed_type.as_ref().map(|ty| &ty.kind),
        Some(&ParsedTypeKind::Record)
    );
    assert_eq!(records.library.functions[0].name, "describeRecord");
}

#[test]
fn preserves_super_parameter_and_extension_type_surfaces() {
    let super_params = parse(
        120,
        include_str!("../fixtures/dart_versions/dart_2_17_super_parameters.dart"),
    );
    let child = &super_params.library.classes[1];
    assert_eq!(child.name, "ChildRecord");
    assert_eq!(child.superclass_name.as_deref(), Some("ParentRecord"));
    assert_eq!(child.constructors[0].params[0].name, "id");
    assert_eq!(child.constructors[0].params[1].name, "title");
    assert_eq!(child.constructors[0].params[2].name, "enabled");

    let extension_types = parse(
        121,
        include_str!("../fixtures/dart_versions/dart_3_3_extension_types.dart"),
    );
    let user_id = &extension_types.library.extension_types[0];
    assert_eq!(user_id.name, "UserId");
    assert_eq!(user_id.representation_name, "value");
    assert_eq!(
        user_id.representation_type_source.as_deref(),
        Some("String")
    );
}

#[test]
fn reports_diagnostics_for_grammar_blocked_future_syntax() {
    let fixtures = [
        DartVersionFixture {
            label: "Dart 3.10 dot shorthands",
            language_version: "3.10",
            path: "crates/dust_parser_dart_ts/tests/fixtures/dart_versions/dart_3_10_dot_shorthands.dart",
            source: include_str!("../fixtures/dart_versions/dart_3_10_dot_shorthands.dart"),
        },
        DartVersionFixture {
            label: "Dart 3.13 primary constructors",
            language_version: "3.13",
            path: "crates/dust_parser_dart_ts/tests/fixtures/dart_versions/dart_3_13_primary_constructors.dart",
            source: include_str!("../fixtures/dart_versions/dart_3_13_primary_constructors.dart"),
        },
    ];

    for (index, fixture) in fixtures.iter().enumerate() {
        let result = parse(130 + index as u32, fixture.source);

        assert!(
            result.has_errors(),
            "{} language={} path={} should stay diagnostic-gated until tree-sitter Dart supports it",
            fixture.label,
            fixture.language_version,
            fixture.path
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("Dart syntax error"))
        );
    }
}

#[test]
fn parses_private_named_parameter_fixture_without_version_gating() {
    let result = parse(
        140,
        include_str!("../fixtures/dart_versions/dart_3_12_private_named_parameters.dart"),
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let constructor = &result.library.classes[0].constructors[0];
    assert_eq!(constructor.params[0].name, "value");
    assert_eq!(constructor.params[1].name, "_traceId");
}
