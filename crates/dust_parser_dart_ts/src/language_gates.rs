use dust_dart_syntax::{DartLanguageFeature, DartLanguageVersion};
use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_parser_dart::{ParameterKind, ParsedClassKind, ParsedDartFileSurface, ParsedTypeKind};
use dust_text::{SourceText, TextRange};

/// Locating where an unsupported language feature appears in a source file.
mod occurrences;
use self::occurrences::*;

/// Extracts diagnostics for Dart syntax that requires a newer language version.
pub(crate) fn extract_language_version_diagnostics(
    library: &ParsedDartFileSurface,
    source: &SourceText,
    version: DartLanguageVersion,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(extract_invalid_normal_parameter_modifier_diagnostics(
        library, source, version,
    ));
    for occurrence in feature_occurrences(library, source.as_str()) {
        if version.supports(occurrence.feature) {
            continue;
        }
        diagnostics.push(
            Diagnostic::error(format!(
                "{} require Dart {} or newer; current language version is {}",
                occurrence.feature.label(),
                occurrence.feature.required_version(),
                version
            ))
            .with_label(SourceLabel::new(
                source.file_id(),
                occurrence.range,
                format!("requires Dart {}", occurrence.feature.required_version()),
            )),
        );
    }
    diagnostics
}

/// Extracts diagnostics for `var`/`final` used as normal parameter modifiers.
fn extract_invalid_normal_parameter_modifier_diagnostics(
    library: &ParsedDartFileSurface,
    source: &SourceText,
    version: DartLanguageVersion,
) -> Vec<Diagnostic> {
    if version < DartLanguageVersion::DART_3_13 {
        return Vec::new();
    }

    normal_parameter_spans(library)
        .into_iter()
        .filter_map(|span| invalid_declaring_modifier(source, span))
        .map(|modifier| {
            Diagnostic::error(format!(
                "`{}` is only valid on primary-constructor declaring parameters in Dart {} or newer",
                modifier.keyword,
                DartLanguageVersion::DART_3_13
            ))
            .with_label(SourceLabel::new(
                source.file_id(),
                modifier.range,
                format!(
                    "remove `{}` from this normal parameter",
                    modifier.keyword
                ),
            ))
            .with_note(
                "Use `Type name` for normal function, method, and constructor parameters.",
            )
        })
        .collect()
}

/// One invalid normal parameter modifier occurrence.
struct InvalidParameterModifier {
    /// Modifier keyword.
    keyword: &'static str,
    /// Source range covering the keyword.
    range: TextRange,
}

/// Returns spans for normal function, method, and constructor parameters.
fn normal_parameter_spans(library: &ParsedDartFileSurface) -> Vec<TextRange> {
    let mut spans = Vec::new();
    for function in &library.functions {
        spans.extend(function.params.iter().map(|param| param.span));
    }
    for class in &library.classes {
        for constructor in &class.constructors {
            spans.extend(constructor.params.iter().map(|param| param.span));
        }
        for method in &class.methods {
            spans.extend(method.params.iter().map(|param| param.span));
        }
    }
    spans
}

/// Finds an invalid declaring modifier at the start of a normal parameter span.
fn invalid_declaring_modifier(
    source: &SourceText,
    span: TextRange,
) -> Option<InvalidParameterModifier> {
    let text = source.slice(span)?;
    let (prefix_offset, remaining) = skip_parameter_prefix(text);
    let leading_ws = remaining.len() - remaining.trim_start().len();
    let candidate = remaining.trim_start();
    let keyword_offset = prefix_offset + leading_ws;

    for keyword in ["final", "var"] {
        if let Some(after) = candidate.strip_prefix(keyword) {
            if after
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
            {
                continue;
            }
            return Some(InvalidParameterModifier {
                keyword,
                range: TextRange::at(span.start().to_usize() + keyword_offset, keyword.len()),
            });
        }
    }
    None
}

/// Skips annotations and legal leading modifiers before checking a parameter modifier.
fn skip_parameter_prefix(mut text: &str) -> (usize, &str) {
    let mut offset = 0;
    loop {
        let trimmed = text.trim_start();
        offset += text.len() - trimmed.len();
        text = trimmed;

        if let Some(after_annotation) = skip_leading_annotation(text) {
            offset += text.len() - after_annotation.len();
            text = after_annotation;
            continue;
        }
        if let Some((modifier_len, after_modifier)) =
            strip_leading_keyword(text, &["required", "covariant"])
        {
            offset += modifier_len;
            text = after_modifier;
            continue;
        }
        return (offset, text);
    }
}

/// Strips one leading keyword from a parameter prefix.
fn strip_leading_keyword<'a>(text: &'a str, keywords: &[&str]) -> Option<(usize, &'a str)> {
    for keyword in keywords {
        if let Some(after_keyword) = text.strip_prefix(keyword) {
            if after_keyword
                .chars()
                .next()
                .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
            {
                return Some((keyword.len(), after_keyword));
            }
        }
    }
    None
}

/// Skips one simple leading Dart metadata annotation.
fn skip_leading_annotation(text: &str) -> Option<&str> {
    let text = text.strip_prefix('@')?;
    let mut depth = 0usize;
    for (index, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ' ' | '\n' | '\r' | '\t' if depth == 0 => {
                return Some(&text[index..]);
            }
            _ => {}
        }
    }
    Some("")
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use dust_parser_dart::{ParseBackend, ParseOptions};
    use dust_text::{FileId, SourceText};

    use crate::TreeSitterDartBackend;

    #[test]
    fn reports_required_version_for_supported_future_feature() {
        let source = SourceText::new(
            FileId::new(1),
            Arc::<str>::from(
                "// @dart=3.0\n\
                 class PrivateNamedParameter {\n\
                   const PrivateNamedParameter({String? _traceId});\n\
                 }\n",
            ),
        );

        let result = TreeSitterDartBackend::new().parse_file(&source, ParseOptions::default());

        assert!(result.has_errors());
        assert!(result.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("private named parameters require Dart 3.12")
        }));
    }

    #[test]
    fn file_language_comment_overrides_parse_options_version() {
        let source = SourceText::new(
            FileId::new(2),
            Arc::<str>::from(
                "// @dart=3.12\n\
                 class PrivateNamedParameter {\n\
                   const PrivateNamedParameter({String? _traceId});\n\
                 }\n",
            ),
        );
        let options = ParseOptions {
            language_version: dust_dart_syntax::DartLanguageVersion::DART_3_0,
            ..ParseOptions::default()
        };

        let result = TreeSitterDartBackend::new().parse_file(&source, options);

        assert!(
            result.diagnostics.is_empty(),
            "diagnostics: {:?}",
            result.diagnostics
        );
    }
}
