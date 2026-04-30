#![cfg(test)]

use super::lower_class;
use dust_ir::{ClassKindIr, ConfigApplicationIr, SerdeRenameRuleIr, SpanIr, SymbolId};
use dust_resolver::{ResolvedClass, ResolvedField};
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(99), TextRange::new(start, end))
}

#[test]
fn lowers_serde_configs_into_ir() {
    let class = ResolvedClass {
        kind: ClassKindIr::Class,
        name: "User".to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields: vec![ResolvedField {
            name: "name".to_owned(),
            type_source: Some("String".to_owned()),
            has_default: false,
            span: span(20, 30),
            configs: vec![ConfigApplicationIr {
                symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                arguments_source: Some(
                    "(rename: 'full_name', aliases: ['fullName'], using: const NameCodec(), defaultValue: 'guest')"
                        .to_owned(),
                ),
                span: span(18, 30),
            }],
        }],
        constructors: Vec::new(),
        traits: Vec::new(),
        configs: vec![ConfigApplicationIr {
            symbol: SymbolId::new("derive_serde_annotation::SerDe"),
            arguments_source: Some(
                "(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)".to_owned(),
            ),
            span: span(1, 10),
        }],
    };

    let outcome = lower_class(&class);
    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    assert_eq!(
        outcome
            .value
            .serde
            .as_ref()
            .and_then(|serde| serde.rename_all),
        Some(SerdeRenameRuleIr::SnakeCase)
    );
    assert_eq!(
        outcome.value.fields[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.rename.as_deref()),
        Some("full_name")
    );
    assert_eq!(
        outcome.value.fields[0]
            .serde
            .as_ref()
            .map(|serde| serde.aliases.clone()),
        Some(vec!["fullName".to_owned()])
    );
    assert_eq!(
        outcome.value.fields[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.codec_source.as_deref()),
        Some("const NameCodec()")
    );
    assert_eq!(
        outcome.value.fields[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.default_value_source.as_deref()),
        Some("'guest'")
    );
}

#[test]
fn invalid_serde_options_produce_lowering_diagnostics() {
    let class = ResolvedClass {
        kind: ClassKindIr::Class,
        name: "User".to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields: vec![ResolvedField {
            name: "name".to_owned(),
            type_source: Some("String".to_owned()),
            has_default: false,
            span: span(20, 30),
            configs: vec![ConfigApplicationIr {
                symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                arguments_source: Some("(renameAll: SerDeRename.snakeCase)".to_owned()),
                span: span(18, 30),
            }],
        }],
        constructors: Vec::new(),
        traits: Vec::new(),
        configs: vec![ConfigApplicationIr {
            symbol: SymbolId::new("derive_serde_annotation::SerDe"),
            arguments_source: Some("(aliases: ['legacy'], using: const NameCodec())".to_owned()),
            span: span(1, 10),
        }],
    };

    let outcome = lower_class(&class);
    assert_eq!(outcome.diagnostics.len(), 3);
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("class `User` does not support `SerDe(aliases: ...)`")
    }));
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("class `User` does not support `SerDe(using: ...)`")
    }));
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("field `name` does not support `SerDe(renameAll: ...)`")
    }));
}

#[test]
fn invalid_serde_using_values_produce_lowering_diagnostics() {
    let class = ResolvedClass {
        kind: ClassKindIr::Class,
        name: "User".to_owned(),
        is_abstract: false,
        superclass_name: None,
        span: span(0, 100),
        fields: vec![
            ResolvedField {
                name: "emptyCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                has_default: false,
                span: span(20, 30),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some("(using: )".to_owned()),
                    span: span(18, 30),
                }],
            },
            ResolvedField {
                name: "stringCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                has_default: false,
                span: span(31, 40),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some("(using: 'codec')".to_owned()),
                    span: span(31, 40),
                }],
            },
            ResolvedField {
                name: "nullCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                has_default: false,
                span: span(41, 50),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some("(using: null)".to_owned()),
                    span: span(41, 50),
                }],
            },
            ResolvedField {
                name: "lambdaCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                has_default: false,
                span: span(51, 60),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some("(using: () => const DateTimeCodec())".to_owned()),
                    span: span(51, 60),
                }],
            },
            ResolvedField {
                name: "typeCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                has_default: false,
                span: span(61, 70),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some("(using: DateTimeCodec)".to_owned()),
                    span: span(61, 70),
                }],
            },
            ResolvedField {
                name: "validCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                has_default: false,
                span: span(71, 80),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some("(using: const DateTimeCodec())".to_owned()),
                    span: span(71, 80),
                }],
            },
        ],
        constructors: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
    };

    let outcome = lower_class(&class);

    assert_eq!(outcome.diagnostics.len(), 5);
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("field `emptyCodec` uses empty `SerDe(using: ...)` value")
    }));
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("field `stringCodec` uses invalid `SerDe(using: ...)` value `'codec'`")
    }));
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("field `nullCodec` uses invalid `SerDe(using: ...)` value `null`")
    }));
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "field `lambdaCodec` uses invalid `SerDe(using: ...)` value `() => const DateTimeCodec()`",
        )
    }));
    assert!(outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.contains(
            "field `typeCodec` uses suspicious `SerDe(using: ...)` type reference `DateTimeCodec`",
        )
    }));
    assert_eq!(
        outcome.value.fields[5]
            .serde
            .as_ref()
            .and_then(|serde| serde.codec_source.as_deref()),
        Some("const DateTimeCodec()")
    );
    for field in &outcome.value.fields[..5] {
        assert_eq!(
            field
                .serde
                .as_ref()
                .and_then(|serde| serde.codec_source.as_deref()),
            None
        );
    }
    assert!(outcome.diagnostics.iter().all(|diagnostic| {
        diagnostic.notes.iter().any(|note| {
            note.contains("Use a codec object such as `const UnixEpochDateTimeCodec()`")
        })
    }));
}
