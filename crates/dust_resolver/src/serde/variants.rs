//! Resolving how one enum variant is tagged and what it carries.

use super::*;

/// Resolves one sealed variant wire tag.
pub(super) fn variant_serde_tag(
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
pub(super) fn variant_constructor_params(
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
