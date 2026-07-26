use dust_dart_syntax::{DartLanguageFeature, DartLanguageVersion};
use dust_diagnostics::{Diagnostic, SourceLabel};
use dust_parser_dart::{ParameterKind, ParsedClassKind, ParsedDartFileSurface, ParsedTypeKind};
use dust_text::{SourceText, TextRange};

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

/// One detected language-versioned feature use.
#[derive(Debug, Clone, Copy)]
struct FeatureOccurrence {
    /// Detected feature.
    feature: DartLanguageFeature,
    /// Source range that best represents the feature.
    range: TextRange,
}

/// Collects at most one occurrence for each feature Dust gates today.
fn feature_occurrences(library: &ParsedDartFileSurface, source: &str) -> Vec<FeatureOccurrence> {
    let mut occurrences = Vec::new();

    push_source_occurrence(
        &mut occurrences,
        source,
        DartLanguageFeature::Records,
        find_record_surface(library).or_else(|| find_any(source, &["final (", "= ("])),
    );
    push_source_occurrence(
        &mut occurrences,
        source,
        DartLanguageFeature::Patterns,
        find_any(source, &["case ("]),
    );
    push_source_occurrence(
        &mut occurrences,
        source,
        DartLanguageFeature::ClassModifiers,
        find_class_modifier(library, source),
    );
    push_range_occurrence(
        &mut occurrences,
        DartLanguageFeature::ExtensionTypes,
        library
            .extension_types
            .first()
            .map(|extension_type| extension_type.span),
    );
    push_source_occurrence(
        &mut occurrences,
        source,
        DartLanguageFeature::NullAwareCollectionElements,
        find_null_aware_collection_element(source),
    );
    push_source_occurrence(
        &mut occurrences,
        source,
        DartLanguageFeature::DotShorthands,
        find_dot_shorthand(source),
    );
    push_range_occurrence(
        &mut occurrences,
        DartLanguageFeature::PrivateNamedParameters,
        find_private_named_parameter(library),
    );
    push_source_occurrence(
        &mut occurrences,
        source,
        DartLanguageFeature::PrimaryConstructors,
        find_primary_constructor(source),
    );

    occurrences
}

/// Records one source-indexed occurrence when the feature was found.
fn push_source_occurrence(
    occurrences: &mut Vec<FeatureOccurrence>,
    source: &str,
    feature: DartLanguageFeature,
    index: Option<usize>,
) {
    push_range_occurrence(
        occurrences,
        feature,
        index.map(|index| TextRange::at(index, 1usize)),
    );
    if let Some(last) = occurrences.last_mut() {
        if last.feature == feature && last.range.end().to_usize() > source.len() {
            last.range = TextRange::at(index.unwrap_or_default(), 1usize);
        }
    }
}

/// Records one range occurrence when the feature has not already been recorded.
fn push_range_occurrence(
    occurrences: &mut Vec<FeatureOccurrence>,
    feature: DartLanguageFeature,
    range: Option<TextRange>,
) {
    if occurrences
        .iter()
        .any(|occurrence| occurrence.feature == feature)
    {
        return;
    }
    if let Some(range) = range {
        occurrences.push(FeatureOccurrence { feature, range });
    }
}

/// Finds record usage through parsed type surfaces.
fn find_record_surface(library: &ParsedDartFileSurface) -> Option<usize> {
    let class_field = library
        .classes
        .iter()
        .flat_map(|class| class.fields.iter())
        .find(|field| {
            field
                .parsed_type
                .as_ref()
                .is_some_and(|ty| ty.kind == ParsedTypeKind::Record)
        });
    class_field.map(|field| field.span.start().to_usize())
}

/// Finds class modifier usage through parsed facts or source fallback.
fn find_class_modifier(library: &ParsedDartFileSurface, source: &str) -> Option<usize> {
    if let Some(class) = library.classes.iter().find(|class| {
        class.is_interface
            || matches!(
                class.kind,
                ParsedClassKind::SealedClass | ParsedClassKind::MixinClass
            )
    }) {
        return Some(class.span.start().to_usize());
    }
    find_any(
        source,
        &[
            "base class",
            "final class",
            "interface class",
            "abstract interface class",
            "abstract base class",
            "abstract final class",
            "mixin class",
            "sealed class",
        ],
    )
}

/// Finds null-aware collection element syntax.
fn find_null_aware_collection_element(source: &str) -> Option<usize> {
    if let Some(index) = source.find("...?") {
        return Some(index);
    }
    let mut offset = 0;
    for line in source.lines() {
        let leading = line.len() - line.trim_start().len();
        let trimmed = line.trim_start();
        if trimmed.starts_with('?')
            && trimmed
                .chars()
                .nth(1)
                .is_some_and(|ch| !ch.is_whitespace() && ch != '.')
        {
            return Some(offset + leading);
        }
        offset += line.len() + 1;
    }
    None
}

/// Finds dot shorthand syntax while avoiding ordinary member accesses.
fn find_dot_shorthand(source: &str) -> Option<usize> {
    for needle in ["= .", "[.", ", .", "(. ", "(.", ": .", "return ."] {
        if let Some(index) = source.find(needle) {
            let dot_index = index + needle.rfind('.').unwrap_or_default();
            if source[dot_index + 1..].starts_with('.') {
                continue;
            }
            return Some(dot_index);
        }
    }
    None
}

/// Finds a private named parameter in parsed constructor, method, or function params.
fn find_private_named_parameter(library: &ParsedDartFileSurface) -> Option<TextRange> {
    library
        .classes
        .iter()
        .flat_map(|class| class.constructors.iter())
        .flat_map(|constructor| constructor.params.iter())
        .find(|param| param.kind == ParameterKind::Named && param.name.starts_with('_'))
        .map(|param| param.span)
        .or_else(|| {
            library
                .classes
                .iter()
                .flat_map(|class| class.methods.iter())
                .flat_map(|method| method.params.iter())
                .find(|param| param.kind == ParameterKind::Named && param.name.starts_with('_'))
                .map(|param| param.span)
        })
        .or_else(|| {
            library
                .functions
                .iter()
                .flat_map(|function| function.params.iter())
                .find(|param| param.kind == ParameterKind::Named && param.name.starts_with('_'))
                .map(|param| param.span)
        })
}

/// Finds primary constructor syntax in source text.
fn find_primary_constructor(source: &str) -> Option<usize> {
    let mut rest = source;
    let mut base = 0;
    while let Some(local_index) = rest.find("class ") {
        let class_index = base + local_index;
        let after_class = &source[class_index + "class ".len()..];
        let name_len = after_class
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
            .map(char::len_utf8)
            .sum::<usize>();
        if name_len == 0 {
            base = class_index + "class ".len();
            rest = &source[base..];
            continue;
        }
        let after_name = after_class[name_len..].trim_start();
        if after_name.starts_with('(') {
            return Some(class_index);
        }
        base = class_index + "class ".len();
        rest = &source[base..];
    }
    None
}

/// Finds the first occurrence among a small set of source needles.
fn find_any(source: &str, needles: &[&str]) -> Option<usize> {
    needles
        .iter()
        .filter_map(|needle| source.find(needle))
        .min()
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
