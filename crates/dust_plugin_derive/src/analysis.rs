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

    derive_arguments_has_copy_with(annotation.arguments_source.as_deref().unwrap_or(""))
}

fn derive_arguments_has_copy_with(arguments_source: &str) -> bool {
    let bytes = arguments_source.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if !is_ident_start(bytes[index]) {
            index += 1;
            continue;
        }

        let start = index;
        index += 1;
        while index < bytes.len() && is_ident_continue(bytes[index]) {
            index += 1;
        }

        if &arguments_source[start..index] == "CopyWith" && bytes.get(index) == Some(&b'(') {
            return true;
        }
    }

    false
}

fn is_ident_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_ident_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}
