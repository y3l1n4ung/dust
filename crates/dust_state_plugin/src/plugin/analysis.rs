use std::path::{Component, Path, PathBuf};

use dust_parser_dart::{ParsedClassSurface, ParsedLibrarySurface};
use dust_plugin_api::{WorkspaceAnalysisBuilder, WorkspaceAnalysisContext};

use super::{
    constants::{STATES_ANALYSIS_KEY, VIEW_MODEL, VIEW_MODELS_ANALYSIS_KEY},
    model::{StateFact, StateFieldFact, ViewModelFact},
    parse::parse_view_model_surface,
};

pub(crate) fn collect_state_workspace_analysis(
    context: WorkspaceAnalysisContext<'_>,
    library: &ParsedLibrarySurface,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    for class in &library.classes {
        collect_state_fact(class, analysis);
        if let Some(annotation) = view_model_annotation(class) {
            let fact = ViewModelFact {
                class_name: class.name.clone(),
                state_type: annotation.state_type,
                args_type: annotation.args_type,
                initial_source: annotation.initial_source,
                generated_base_class: format!("${}", class.name),
                import_uri: import_uri(context),
            };
            if let Ok(value) = serde_json::to_string(&fact) {
                analysis.add_string_set_value(VIEW_MODELS_ANALYSIS_KEY, value);
            }
        }
    }
}

fn collect_state_fact(class: &ParsedClassSurface, analysis: &mut WorkspaceAnalysisBuilder) {
    let fields = class
        .fields
        .iter()
        .map(|field| StateFieldFact {
            name: field.name.clone(),
            type_source: field
                .type_source
                .clone()
                .unwrap_or_else(|| "dynamic".to_owned()),
        })
        .collect::<Vec<_>>();
    let fact = StateFact {
        class_name: class.name.clone(),
        fields,
    };
    if let Ok(value) = serde_json::to_string(&fact) {
        analysis.add_string_set_value(STATES_ANALYSIS_KEY, value);
    }
}

fn view_model_annotation(class: &ParsedClassSurface) -> Option<super::model::ViewModelAnnotation> {
    class
        .annotations
        .iter()
        .find(|annotation| annotation.name == VIEW_MODEL)
        .and_then(parse_view_model_surface)
}

fn import_uri(context: WorkspaceAnalysisContext<'_>) -> String {
    let source_path = context.source_path;
    let package_root = context.package_root;
    if let Some(path) = source_path
        .strip_prefix(package_root)
        .ok()
        .and_then(|relative| relative.strip_prefix("lib").ok())
    {
        return format!("package:{}/{}", context.package_name, normalize_path(path));
    }
    source_path.display().to_string()
}

fn normalize_path(path: &Path) -> String {
    normalize_components(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_components(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => components.push(value.to_owned()),
            Component::ParentDir => {
                components.pop();
            }
            _ => {}
        }
    }
    components.into_iter().collect()
}
