use dust_parser_dart::{ParsedAnnotation, ParsedClassSurface, ParsedDartFileSurface};
use dust_plugin_api::WorkspaceAnalysisBuilder;

pub(crate) const COPYABLE_TYPES_ANALYSIS_KEY: &str = "dust_plugin_derive.copyable_types";

pub(crate) fn collect_workspace_analysis(
    library: &ParsedDartFileSurface,
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

    annotation
        .positional_constructor_names()
        .iter()
        .any(|name| name == "CopyWith")
}
