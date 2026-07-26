use std::collections::{HashMap, HashSet};

use dust_diagnostics::Diagnostic;
use dust_ir::{
    AnnotationValueIr, ClassKindIr, ConfigApplicationIr, ConstructorParamIr, ParamKind,
    SerdeClassConfigIr, SerdeEnumVariantConfigIr, SerdeFieldConfigIr, SerdeRenameRuleIr,
    SerdeVariantConfigIr, SpanIr, TypeIr, apply_serde_rename_rule,
};
use dust_parser_dart::ParameterKind;

use crate::{ResolvedClass, ResolvedConstructor, lower_type_ir};

/// Normalizes class-level SerDe configuration after symbol resolution.
pub(crate) fn normalize_class_serde(
    class_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeClassConfigIr> {
    let mut serde = SerdeClassConfigIr::default();
    let mut saw_serde = false;

    for config in configs
        .iter()
        .filter(|config| config.symbol.0 == "dust_dart::SerDe")
    {
        saw_serde = true;
        if !config.positional_args.is_empty() {
            diagnostics.push(Diagnostic::error(
                "SerDe config arguments must use parenthesized named arguments",
            ));
        }

        for key in config.named_args.keys().map(String::as_str) {
            match key {
                "rename" => match config.named_string(key) {
                    Some(value) => serde.rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "renameAll" => match config.named_member(key).as_deref().and_then(rename_rule) {
                    Some(value) => serde.rename_all = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses an unknown `SerDe(renameAll: ...)` rule"
                    ))),
                },
                "tag" => match config.named_string(key) {
                    Some(value) => serde.tag = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(tag: ...)` value"
                    ))),
                },
                "content" => match config.named_string(key) {
                    Some(value) => serde.content = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(content: ...)` value"
                    ))),
                },
                "untagged" => match config.named_bool(key) {
                    Some(value) => serde.untagged = value,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-boolean `SerDe(untagged: ...)` value"
                    ))),
                },
                "disallowUnrecognizedKeys" => match config.named_bool(key) {
                    Some(value) => serde.disallow_unrecognized_keys = value,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-boolean `SerDe(disallowUnrecognizedKeys: ...)` value"
                    ))),
                },
                "aliases" | "defaultValue" | "skip" | "skipSerializing"
                | "skipDeserializing" | "using" => diagnostics.push(Diagnostic::error(format!(
                    "class `{class_name}` does not support `SerDe({key}: ...)`"
                ))),
                unknown => diagnostics.push(Diagnostic::warning(format!(
                    "class `{class_name}` uses unknown `SerDe` option `{unknown}`"
                ))),
            }
        }
    }

    if serde.content.is_some() && serde.tag.is_none() {
        diagnostics.push(Diagnostic::error(
            "SerDe(content: ...) requires SerDe(tag: ...)",
        ));
    }
    if serde.tag.is_some() && serde.untagged {
        diagnostics.push(Diagnostic::error(
            "SerDe(tag: ...) cannot be used with SerDe(untagged: true)",
        ));
    }

    saw_serde.then_some(serde)
}

/// Parses the short member of a SerDe rename rule.
fn rename_rule(source: &str) -> Option<SerdeRenameRuleIr> {
    match source.trim().rsplit('.').next()? {
        "lowerCase" => Some(SerdeRenameRuleIr::LowerCase),
        "upperCase" => Some(SerdeRenameRuleIr::UpperCase),
        "pascalCase" => Some(SerdeRenameRuleIr::PascalCase),
        "camelCase" => Some(SerdeRenameRuleIr::CamelCase),
        "snakeCase" => Some(SerdeRenameRuleIr::SnakeCase),
        "screamingSnakeCase" => Some(SerdeRenameRuleIr::ScreamingSnakeCase),
        "kebabCase" => Some(SerdeRenameRuleIr::KebabCase),
        "screamingKebabCase" => Some(SerdeRenameRuleIr::ScreamingKebabCase),
        _ => None,
    }
}

/// Normalizes enum-variant SerDe configuration after symbol resolution.
pub(crate) fn normalize_enum_variant_serde(
    variant_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeEnumVariantConfigIr> {
    let mut serde = SerdeEnumVariantConfigIr::default();
    let mut saw_serde = false;
    for config in configs
        .iter()
        .filter(|config| config.symbol.0 == "dust_dart::SerDe")
    {
        saw_serde = true;
        for key in config.named_args.keys().map(String::as_str) {
            match key {
                "rename" => match config.named_string(key) {
                    Some(value) => serde.rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "enum variant `{variant_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "skip" => match config.named_bool(key) {
                    Some(value) => serde.skip = value,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "enum variant `{variant_name}` uses a non-boolean `SerDe(skip: ...)` value"
                    ))),
                },
                unknown => diagnostics.push(Diagnostic::error(format!(
                    "enum variant `{variant_name}` does not support `SerDe({unknown}: ...)`"
                ))),
            }
        }
    }
    saw_serde.then_some(serde)
}

/// Normalizes field-level SerDe configuration after symbol resolution.
pub(crate) fn normalize_field_serde(
    field_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeFieldConfigIr> {
    let mut serde = SerdeFieldConfigIr::default();
    let mut saw_serde = false;
    for config in configs
        .iter()
        .filter(|config| config.symbol.0 == "dust_dart::SerDe")
    {
        saw_serde = true;
        for key in config.named_args.keys().map(String::as_str) {
            let source = config.named_argument_source(key).unwrap_or_default();
            match key {
                "rename" => match config.named_string(key) {
                    Some(value) => serde.rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-string `SerDe(rename: ...)` value"))),
                },
                "aliases" => match config.named_string_list(key) {
                    Some(value) => serde.aliases = value,
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-string-list `SerDe(aliases: ...)` value"))),
                },
                "using" => match config.named_argument_value(key) {
                    Some(AnnotationValueIr::Constructor { .. }) => serde.codec_source = Some(source.trim().to_owned()),
                    Some(AnnotationValueIr::Member(name)) if name.prefix.is_some() || name.source.contains('.') || name.short.chars().next().is_some_and(|first| first.is_ascii_lowercase()) => serde.codec_source = Some(source.trim().to_owned()),
                    Some(AnnotationValueIr::Member(_)) => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses suspicious `SerDe(using: ...)` type reference `{}`", source.trim())).with_note(codec_guidance())),
                    _ => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses invalid `SerDe(using: ...)` value `{}`", source.trim())).with_note(codec_guidance())),
                },
                "defaultValue" => {
                    serde.default_value_source = Some(source.trim().to_owned());
                    serde.default_value = config.named_argument_value(key).cloned();
                }
                "skip" => match config.named_bool(key) {
                    Some(true) => { serde.skip_serializing = true; serde.skip_deserializing = true; }
                    Some(false) => {}
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-boolean `SerDe(skip: ...)` value"))),
                },
                "skipSerializing" => match config.named_bool(key) {
                    Some(value) => serde.skip_serializing = value,
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-boolean `SerDe(skipSerializing: ...)` value"))),
                },
                "skipDeserializing" => match config.named_bool(key) {
                    Some(value) => serde.skip_deserializing = value,
                    None => diagnostics.push(Diagnostic::error(format!("field `{field_name}` uses a non-boolean `SerDe(skipDeserializing: ...)` value"))),
                },
                unsupported @ ("renameAll" | "tag" | "content" | "untagged" | "disallowUnrecognizedKeys") => diagnostics.push(Diagnostic::error(format!("field `{field_name}` does not support `SerDe({unsupported}: ...)`"))),
                unknown => diagnostics.push(Diagnostic::warning(format!("field `{field_name}` uses unknown `SerDe` option `{unknown}`"))),
            }
        }
    }
    saw_serde.then_some(serde)
}

/// Returns guidance for passing a codec instance instead of a type or literal.
fn codec_guidance() -> &'static str {
    "Use a codec object such as `const UnixEpochDateTimeCodec()` or `unixEpochDateTimeCodec`."
}

/// Completes sealed class SerDe variant metadata from redirecting factory constructors.
pub(crate) fn normalize_sealed_serde_variants(
    classes: &mut [ResolvedClass],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let index_by_name = classes
        .iter()
        .enumerate()
        .map(|(index, class)| (class.name.clone(), index))
        .collect::<HashMap<_, _>>();
    for index in 0..classes.len() {
        let has_serde = classes[index].serde.is_some();
        let uses_sealed_serde = classes[index]
            .serde
            .as_ref()
            .is_some_and(|serde| serde.uses_sealed_representation());
        if !has_serde {
            continue;
        }

        let base_name = classes[index].name.clone();
        if classes[index].kind != ClassKindIr::SealedClass {
            if !uses_sealed_serde {
                continue;
            }
            diagnostics.push(Diagnostic::error(format!(
                "SerDe class `{base_name}` uses sealed variant options but is not sealed"
            )));
            continue;
        }

        let rename_all = classes[index]
            .serde
            .as_ref()
            .and_then(|serde| serde.rename_all);
        let constructors = &classes[index].constructors;
        let mut variants = Vec::new();
        let mut seen_tags = HashSet::new();

        for constructor in constructors
            .iter()
            .filter(|constructor| constructor.surface.is_factory)
        {
            let Some(constructor_name) = constructor.surface.name.as_deref() else {
                continue;
            };
            let Some(target_class_name) = constructor.surface.redirected_target_name.as_deref()
            else {
                continue;
            };

            let target_superclass = index_by_name
                .get(target_class_name)
                .and_then(|target_index| classes.get(*target_index))
                .and_then(|target| target.superclass_name.as_deref());
            if target_superclass.is_some() && target_superclass != Some(base_name.as_str()) {
                diagnostics.push(Diagnostic::error(format!(
                    "Variant target class {target_class_name} does not extend {base_name}"
                )));
            }

            let tag = variant_serde_tag(constructor_name, constructor, rename_all, diagnostics);
            if !seen_tags.insert(tag.clone()) {
                diagnostics.push(Diagnostic::error(format!(
                    "Duplicate SerDe variant tag: {tag}"
                )));
            }

            variants.push(SerdeVariantConfigIr {
                constructor_name: constructor_name.to_owned(),
                target_class_name: target_class_name.to_owned(),
                tag,
                params: variant_constructor_params(
                    classes[index].span.file_id,
                    constructor,
                    diagnostics,
                ),
            });
        }

        if variants.is_empty() {
            diagnostics.push(Diagnostic::error(format!(
                "Sealed SerDe class {base_name} has no factory variants"
            )));
        }

        if let Some(serde) = &mut classes[index].serde {
            serde.variants = variants;
        }
    }
}

/// Resolves one sealed variant wire tag.
fn variant_serde_tag(
    variant_name: &str,
    constructor: &ResolvedConstructor,
    rename_all: Option<SerdeRenameRuleIr>,
    diagnostics: &mut Vec<Diagnostic>,
) -> String {
    let mut rename = None;

    for config in constructor
        .configs
        .iter()
        .filter(|config| config.symbol.0 == "dust_dart::SerDe")
    {
        for key in config.named_args.keys().map(String::as_str) {
            match key {
                "rename" => match config.named_string(key) {
                    Some(value) => rename = Some(value),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "variant `{variant_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "renameAll"
                | "tag"
                | "content"
                | "untagged"
                | "disallowUnrecognizedKeys"
                | "aliases"
                | "defaultValue"
                | "skip"
                | "skipSerializing"
                | "skipDeserializing"
                | "using" => diagnostics.push(Diagnostic::error(format!(
                    "variant `{variant_name}` does not support `SerDe({key}: ...)`"
                ))),
                unknown => diagnostics.push(Diagnostic::warning(format!(
                    "variant `{variant_name}` uses unknown `SerDe` option `{unknown}`"
                ))),
            }
        }
    }

    rename.unwrap_or_else(|| match rename_all {
        Some(rule) => apply_serde_rename_rule(variant_name, rule),
        None => variant_name.to_owned(),
    })
}

/// Converts variant factory parameters into normalized IR parameters.
fn variant_constructor_params(
    file_id: dust_text::FileId,
    constructor: &ResolvedConstructor,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<ConstructorParamIr> {
    constructor
        .surface
        .params
        .iter()
        .map(|param| {
            let outcome = param
                .type_source
                .as_deref()
                .map(|source| lower_type_ir(param.parsed_type.as_ref(), Some(source)))
                .unwrap_or_else(|| dust_ir::LoweringOutcome::new(TypeIr::unknown()));
            diagnostics.extend(outcome.diagnostics);
            ConstructorParamIr {
                name: param.name.clone(),
                ty: outcome.value,
                span: SpanIr::new(file_id, param.span),
                kind: match param.kind {
                    ParameterKind::Positional => ParamKind::Positional,
                    ParameterKind::Named => ParamKind::Named,
                },
                has_default: param.has_default,
                default_value_source: param.default_value_source.clone(),
            }
        })
        .collect()
}
