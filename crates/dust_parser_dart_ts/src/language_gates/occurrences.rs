//! Locating where an unsupported language feature appears in a source file.

use super::*;

/// One detected language-versioned feature use.
#[derive(Debug, Clone, Copy)]
pub(super) struct FeatureOccurrence {
    /// Detected feature.
    pub(super) feature: DartLanguageFeature,
    /// Source range that best represents the feature.
    pub(super) range: TextRange,
}

/// Collects at most one occurrence for each feature Dust gates today.
pub(super) fn feature_occurrences(
    library: &ParsedDartFileSurface,
    source: &str,
) -> Vec<FeatureOccurrence> {
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
pub(super) fn push_source_occurrence(
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
pub(super) fn push_range_occurrence(
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
pub(super) fn find_record_surface(library: &ParsedDartFileSurface) -> Option<usize> {
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
pub(super) fn find_class_modifier(library: &ParsedDartFileSurface, source: &str) -> Option<usize> {
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
pub(super) fn find_null_aware_collection_element(source: &str) -> Option<usize> {
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
pub(super) fn find_dot_shorthand(source: &str) -> Option<usize> {
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
pub(super) fn find_private_named_parameter(library: &ParsedDartFileSurface) -> Option<TextRange> {
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
pub(super) fn find_primary_constructor(source: &str) -> Option<usize> {
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
pub(super) fn find_any(source: &str, needles: &[&str]) -> Option<usize> {
    needles
        .iter()
        .filter_map(|needle| source.find(needle))
        .min()
}
