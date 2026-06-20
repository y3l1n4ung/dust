use dust_diagnostics::Diagnostic;
use dust_ir::{ConfigApplicationIr, MethodParamIr, SpanIr};

use crate::plugin::util::{config_name, label};

/// Parses a required string argument from a parameter annotation.
pub(crate) fn parse_required_string_argument(
    param: &MethodParamIr,
    annotation_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    parse_optional_string_argument(param, annotation_name, diagnostics).or_else(|| {
        diagnostics.push(
            Diagnostic::error(format!(
                "`@{annotation_name}` on parameter `{}` requires a string argument",
                param.name
            ))
            .with_label(label(
                param.span,
                "add a quoted key argument to this HTTP parameter annotation",
            )),
        );
        None
    })
}

/// Parses an optional string argument from a parameter annotation.
pub(crate) fn parse_optional_string_argument(
    param: &MethodParamIr,
    annotation_name: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let config = param
        .configs
        .iter()
        .find(|config| config_name(&config.symbol.0) == annotation_name)?;
    parse_config_string_argument(config, diagnostics, annotation_name)
}

/// Parses a positional string argument from an annotation config.
pub(crate) fn parse_config_string_argument(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
    label_name: &str,
) -> Option<String> {
    let _ = config.positional_argument_source(0)?;
    match config.positional_string(0) {
        Some(value) => Some(value),
        None => {
            diagnostics.push(
                Diagnostic::error(format!("`{label_name}` expects a quoted string argument"))
                    .with_label(label(config.span, "use a quoted string literal here")),
            );
            None
        }
    }
}

/// Parses a positional string-to-string map argument from an annotation config.
pub(crate) fn parse_config_map_argument(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
    label_name: &str,
) -> Vec<(String, String)> {
    let Some(source) = config.positional_argument_source(0).map(str::trim) else {
        return Vec::new();
    };
    if source.is_empty() {
        return Vec::new();
    }
    match config.positional_string_map(0) {
        Some(values) => values,
        None => {
            diagnostics.push(invalid_string_map(label_name, config.span));
            Vec::new()
        }
    }
}

/// Builds a diagnostic for an invalid string-to-string map annotation argument.
pub(crate) fn invalid_string_map(label_name: &str, span: SpanIr) -> Diagnostic {
    Diagnostic::error(format!(
        "`{label_name}` expects a string-to-string map literal"
    ))
    .with_label(label(span, "use a `{ 'key': 'value' }` literal here"))
}
