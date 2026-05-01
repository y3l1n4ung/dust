use dust_parser_dart::{ParsedAnnotation, ParsedClassSurface, ParsedLibrarySurface};
use dust_plugin_api::WorkspaceAnalysisBuilder;

pub(crate) const COPYABLE_TYPES_ANALYSIS_KEY: &str = "dust_plugin_derive.copyable_types";

pub(crate) fn collect_workspace_analysis(
    library: &ParsedLibrarySurface,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    for class in &library.classes {
        if class_has_copy_with(class) {
            analysis.add_string_set_value(COPYABLE_TYPES_ANALYSIS_KEY, class.name.clone());
        }
    }
}

fn class_has_copy_with(class: &ParsedClassSurface) -> bool {
    class.annotations.iter().any(annotation_has_copy_with)
}

fn annotation_has_copy_with(annotation: &ParsedAnnotation) -> bool {
    if annotation.name == "CopyWith" {
        return true;
    }

    if annotation.name != "Derive" {
        return false;
    }

    derive_member_names(annotation.arguments_source.as_deref().unwrap_or(""))
        .into_iter()
        .any(|name| name == "CopyWith")
}

fn derive_member_names(arguments_source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut chars = arguments_source.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '_' || ch.is_ascii_alphabetic() {
            let mut ident = String::from(ch);
            while let Some(next) = chars.peek() {
                if *next == '_' || next.is_ascii_alphanumeric() {
                    ident.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }

            if chars.peek().copied() == Some('(') {
                names.push(ident);
            }
        }
    }

    names
}
