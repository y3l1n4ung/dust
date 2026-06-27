#![cfg(test)]

use std::collections::BTreeMap;

use super::{lower_class, lower_library};
use dust_ir::{
    AnnotationValueIr, ClassKindIr, ConfigApplicationIr, SerdeRenameRuleIr, SpanIr, SymbolId,
};
use dust_parser_dart::{ParameterKind, ParsedConstructorParamSurface, ParsedConstructorSurface};
use dust_resolver::{
    ResolvedClass, ResolvedConstructor, ResolvedEnum, ResolvedEnumVariant, ResolvedField,
    ResolvedLibrary,
};
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(99), TextRange::new(start, end))
}

fn serde_config(args: &str, start: u32, end: u32) -> ConfigApplicationIr {
    ConfigApplicationIr::new(
        SymbolId::new("dust_dart::SerDe"),
        Some(args.to_owned()),
        span(start, end),
    )
}

fn serde_config_with_default(args: &str, default_value: AnnotationValueIr) -> ConfigApplicationIr {
    ConfigApplicationIr::with_arguments(
        SymbolId::new("dust_dart::SerDe"),
        Some(args.to_owned()),
        Vec::new(),
        BTreeMap::from([("defaultValue".to_owned(), default_value)]),
        span(18, 30),
    )
}

fn empty_class(name: &str, kind: ClassKindIr) -> ResolvedClass {
    ResolvedClass {
        kind,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(0, 100),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
    }
}

fn factory_constructor(
    name: &str,
    target_class_name: &str,
    configs: Vec<ConfigApplicationIr>,
) -> ResolvedConstructor {
    factory_constructor_with_params(name, target_class_name, configs, Vec::new())
}

fn factory_constructor_with_params(
    name: &str,
    target_class_name: &str,
    configs: Vec<ConfigApplicationIr>,
    params: Vec<ParsedConstructorParamSurface>,
) -> ResolvedConstructor {
    ResolvedConstructor {
        surface: ParsedConstructorSurface {
            name: Some(name.to_owned()),
            is_factory: true,
            annotations: Vec::new(),
            redirected_target_source: Some(target_class_name.to_owned()),
            redirected_target_name: Some(target_class_name.to_owned()),
            params,
            span: TextRange::new(0_u32, 10_u32),
        },
        configs,
    }
}

fn named_constructor_param(name: &str, ty: &str) -> ParsedConstructorParamSurface {
    ParsedConstructorParamSurface {
        name: name.to_owned(),
        annotations: Vec::new(),
        type_source: Some(ty.to_owned()),
        parsed_type: None,
        kind: ParameterKind::Named,
        has_default: false,
        default_value_source: None,
        span: TextRange::new(0_u32, 10_u32),
    }
}

fn library(classes: Vec<ResolvedClass>) -> ResolvedLibrary {
    ResolvedLibrary {
        source_path: "lib/payment_status.dart".to_owned(),
        output_path: "lib/payment_status.g.dart".to_owned(),
        span: span(0, 100),
        directives: Vec::new(),
        part_uri: None,
        classes,
        enums: Vec::new(),
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        query_calls: Vec::new(),
    }
}

fn enum_library(enums: Vec<ResolvedEnum>) -> ResolvedLibrary {
    ResolvedLibrary {
        enums,
        ..library(Vec::new())
    }
}

#[test]
fn lowers_serde_configs_into_ir() {
    let class = ResolvedClass {
        kind: ClassKindIr::Class,
        name: "User".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(0, 100),
        fields: vec![ResolvedField {
            name: "name".to_owned(),
            type_source: Some("String".to_owned()),
            parsed_type: None,
            has_default: false,
            span: span(20, 30),
            configs: vec![serde_config_with_default(
                "(rename: 'full_name', aliases: ['fullName'], using: const NameCodec(), defaultValue: 'guest')",
                AnnotationValueIr::String("guest".to_owned()),
            )],
        }],
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![serde_config(
            "(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)",
            1,
            10,
        )],
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
    assert_eq!(
        outcome.value.fields[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.default_value.as_ref()),
        Some(&AnnotationValueIr::String("guest".to_owned()))
    );
}

#[test]
fn lowers_enum_variant_serde_metadata() {
    let outcome = lower_library(&enum_library(vec![ResolvedEnum {
        name: "Status".to_owned(),
        span: span(0, 100),
        variants: vec![
            ResolvedEnumVariant {
                name: "pendingReview".to_owned(),
                span: span(20, 35),
                configs: vec![serde_config("(rename: 'pending')", 20, 35)],
            },
            ResolvedEnumVariant {
                name: "legacyFailed".to_owned(),
                span: span(40, 55),
                configs: vec![serde_config("(skip: true)", 40, 55)],
            },
        ],
        traits: Vec::new(),
        configs: Vec::new(),
    }]));

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let enum_ = &outcome.value.enums[0];
    assert_eq!(
        enum_.variants[0]
            .serde
            .as_ref()
            .and_then(|serde| serde.rename.as_deref()),
        Some("pending")
    );
    assert_eq!(
        enum_.variants[1].serde.as_ref().map(|serde| serde.skip),
        Some(true)
    );
}

#[test]
fn lowers_sealed_serde_variant_metadata_from_redirecting_factories() {
    let mut base = empty_class("PaymentStatus", ClassKindIr::SealedClass);
    base.configs = vec![serde_config(
        "(tag: 'type', renameAll: SerDeRename.snakeCase)",
        1,
        10,
    )];
    base.constructors = vec![
        factory_constructor("paymentPaid", "PaymentPaid", Vec::new()),
        factory_constructor(
            "failed",
            "PaymentFailed",
            vec![serde_config("(rename: 'failure')", 11, 20)],
        ),
    ];

    let mut paid = empty_class("PaymentPaid", ClassKindIr::Class);
    paid.superclass_name = Some("PaymentStatus".to_owned());
    let mut failed = empty_class("PaymentFailed", ClassKindIr::Class);
    failed.superclass_name = Some("PaymentStatus".to_owned());

    let outcome = lower_library(&library(vec![base, paid, failed]));

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let class = outcome
        .value
        .classes
        .iter()
        .find(|class| class.name == "PaymentStatus")
        .unwrap();
    let serde = class.serde.as_ref().unwrap();
    assert_eq!(serde.tag.as_deref(), Some("type"));
    assert_eq!(serde.variants.len(), 2);
    assert_eq!(serde.variants[0].constructor_name, "paymentPaid");
    assert_eq!(serde.variants[0].target_class_name, "PaymentPaid");
    assert_eq!(serde.variants[0].tag, "payment_paid");
    assert_eq!(serde.variants[1].constructor_name, "failed");
    assert_eq!(serde.variants[1].target_class_name, "PaymentFailed");
    assert_eq!(serde.variants[1].tag, "failure");
}

#[test]
fn lowers_sealed_serde_variant_params_without_source_target_class() {
    let mut base = empty_class("JsonPaymentEvent", ClassKindIr::SealedClass);
    base.configs = vec![serde_config(
        "(tag: 'type', renameAll: SerDeRename.snakeCase)",
        1,
        10,
    )];
    base.constructors = vec![factory_constructor_with_params(
        "success",
        "JsonPaymentSuccess",
        Vec::new(),
        vec![
            named_constructor_param("id", "String"),
            named_constructor_param("cents", "int"),
        ],
    )];

    let outcome = lower_library(&library(vec![base]));

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let class = outcome
        .value
        .classes
        .iter()
        .find(|class| class.name == "JsonPaymentEvent")
        .unwrap();
    let serde = class.serde.as_ref().unwrap();
    assert_eq!(serde.variants.len(), 1);
    assert_eq!(serde.variants[0].target_class_name, "JsonPaymentSuccess");
    assert_eq!(serde.variants[0].params.len(), 2);
    assert_eq!(serde.variants[0].params[0].name, "id");
    assert_eq!(serde.variants[0].params[0].ty.name(), Some("String"));
    assert_eq!(serde.variants[0].params[1].name, "cents");
    assert_eq!(serde.variants[0].params[1].ty.name(), Some("int"));
}

#[test]
fn lowers_sealed_serde_variant_metadata_without_explicit_representation() {
    let mut base = empty_class("PaymentStatus", ClassKindIr::SealedClass);
    base.configs = vec![serde_config("(renameAll: SerDeRename.snakeCase)", 1, 10)];
    base.constructors = vec![factory_constructor(
        "paymentPaid",
        "PaymentPaid",
        Vec::new(),
    )];

    let mut paid = empty_class("PaymentPaid", ClassKindIr::Class);
    paid.superclass_name = Some("PaymentStatus".to_owned());

    let outcome = lower_library(&library(vec![base, paid]));

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let class = outcome
        .value
        .classes
        .iter()
        .find(|class| class.name == "PaymentStatus")
        .unwrap();
    let serde = class.serde.as_ref().unwrap();
    assert_eq!(serde.tag.as_deref(), None);
    assert_eq!(serde.variants.len(), 1);
    assert_eq!(serde.variants[0].constructor_name, "paymentPaid");
    assert_eq!(serde.variants[0].target_class_name, "PaymentPaid");
    assert_eq!(serde.variants[0].tag, "payment_paid");
}

#[test]
fn non_sealed_serde_without_variant_options_does_not_require_factories() {
    let mut user = empty_class("User", ClassKindIr::Class);
    user.configs = vec![serde_config("(renameAll: SerDeRename.snakeCase)", 1, 10)];

    let outcome = lower_library(&library(vec![user]));

    assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
    let class = outcome
        .value
        .classes
        .iter()
        .find(|class| class.name == "User")
        .unwrap();
    assert!(class.serde.as_ref().unwrap().variants.is_empty());
}

#[test]
fn invalid_sealed_serde_class_options_produce_lowering_diagnostics() {
    let mut content_only = empty_class("ContentOnly", ClassKindIr::Class);
    content_only.configs = vec![serde_config("(content: 'data')", 1, 10)];
    let content_outcome = lower_class(&content_only);
    assert!(content_outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("SerDe(content: ...) requires SerDe(tag: ...)")
    }));

    let mut conflicting = empty_class("Conflicting", ClassKindIr::Class);
    conflicting.configs = vec![serde_config("(tag: 'type', untagged: true)", 11, 20)];
    let conflicting_outcome = lower_class(&conflicting);
    assert!(conflicting_outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("SerDe(tag: ...) cannot be used with SerDe(untagged: true)")
    }));

    let mut empty_sealed = empty_class("EmptyStatus", ClassKindIr::SealedClass);
    empty_sealed.configs = vec![serde_config("(tag: 'type')", 21, 30)];
    let empty_outcome = lower_library(&library(vec![empty_sealed]));
    assert!(empty_outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Sealed SerDe class EmptyStatus has no factory variants")
    }));

    let mut duplicate_tags = empty_class("DuplicateTags", ClassKindIr::SealedClass);
    duplicate_tags.configs = vec![serde_config("(tag: 'type')", 31, 40)];
    duplicate_tags.constructors = vec![
        factory_constructor("paid", "Paid", Vec::new()),
        factory_constructor("paid", "DuplicatePaid", Vec::new()),
    ];
    let duplicate_outcome = lower_library(&library(vec![duplicate_tags]));
    assert!(duplicate_outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Duplicate SerDe variant tag: paid")
    }));

    let mut invalid_target = empty_class("PaymentStatus", ClassKindIr::SealedClass);
    invalid_target.configs = vec![serde_config("(tag: 'type')", 41, 50)];
    invalid_target.constructors = vec![factory_constructor("paid", "PaymentPaid", Vec::new())];
    let mut paid = empty_class("PaymentPaid", ClassKindIr::Class);
    paid.superclass_name = Some("OtherStatus".to_owned());
    let invalid_target_outcome = lower_library(&library(vec![invalid_target, paid]));
    assert!(invalid_target_outcome.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("Variant target class PaymentPaid does not extend PaymentStatus")
    }));
}

#[test]
fn invalid_serde_options_produce_lowering_diagnostics() {
    let class = ResolvedClass {
        kind: ClassKindIr::Class,
        name: "User".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(0, 100),
        fields: vec![ResolvedField {
            name: "name".to_owned(),
            type_source: Some("String".to_owned()),
            parsed_type: None,
            has_default: false,
            span: span(20, 30),
            configs: vec![serde_config("(renameAll: SerDeRename.snakeCase)", 18, 30)],
        }],
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: vec![serde_config(
            "(aliases: ['legacy'], using: const NameCodec())",
            1,
            10,
        )],
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
        is_interface: false,
        superclass_name: None,
        span: span(0, 100),
        fields: vec![
            ResolvedField {
                name: "emptyCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                parsed_type: None,
                has_default: false,
                span: span(20, 30),
                configs: vec![serde_config("(using: )", 18, 30)],
            },
            ResolvedField {
                name: "stringCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                parsed_type: None,
                has_default: false,
                span: span(31, 40),
                configs: vec![serde_config("(using: 'codec')", 31, 40)],
            },
            ResolvedField {
                name: "nullCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                parsed_type: None,
                has_default: false,
                span: span(41, 50),
                configs: vec![serde_config("(using: null)", 41, 50)],
            },
            ResolvedField {
                name: "lambdaCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                parsed_type: None,
                has_default: false,
                span: span(51, 60),
                configs: vec![serde_config("(using: () => const DateTimeCodec())", 51, 60)],
            },
            ResolvedField {
                name: "typeCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                parsed_type: None,
                has_default: false,
                span: span(61, 70),
                configs: vec![serde_config("(using: DateTimeCodec)", 61, 70)],
            },
            ResolvedField {
                name: "validCodec".to_owned(),
                type_source: Some("DateTime".to_owned()),
                parsed_type: None,
                has_default: false,
                span: span(71, 80),
                configs: vec![serde_config("(using: const DateTimeCodec())", 71, 80)],
            },
        ],
        constructors: Vec::new(),
        methods: Vec::new(),
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
