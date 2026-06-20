use std::path::{Component, Path, PathBuf};

use dust_dart_emit::{
    DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_DYNAMIC, DART_INT, DART_LIST, DART_NUM,
    DART_OBJECT, DART_OBJECT_NULLABLE, DART_STRING, DART_VOID,
};
use dust_parser_dart::{ParsedClassSurface, ParsedDartFileSurface};
use dust_plugin_api::{WorkspaceAnalysisBuilder, WorkspaceAnalysisContext};

use super::{
    constants::{STATES_ANALYSIS_KEY, VIEW_MODEL, VIEW_MODELS_ANALYSIS_KEY},
    model::{StateFact, StateFieldFact, ViewModelFact},
    parse::parse_view_model_surface,
};

pub(crate) fn collect_state_workspace_analysis(
    context: WorkspaceAnalysisContext<'_>,
    library: &ParsedDartFileSurface,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    let declared_type_names = declared_type_names(library);
    for class in &library.classes {
        collect_state_fact(class, &declared_type_names, analysis);
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

fn collect_state_fact(
    class: &ParsedClassSurface,
    declared_type_names: &[String],
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    let fields = class
        .fields
        .iter()
        .map(|field| StateFieldFact {
            name: field.name.clone(),
            type_source: field
                .type_source
                .as_deref()
                .map(|source| sanitize_type_source(source, declared_type_names))
                .unwrap_or_else(|| DART_DYNAMIC.to_owned()),
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
        .find(|annotation| annotation.is_named(VIEW_MODEL))
        .and_then(parse_view_model_surface)
}

fn declared_type_names(library: &ParsedDartFileSurface) -> Vec<String> {
    library
        .classes
        .iter()
        .map(|class| class.name.clone())
        .chain(library.enums.iter().map(|enum_| enum_.name.clone()))
        .collect()
}

fn sanitize_type_source(type_source: &str, declared_type_names: &[String]) -> String {
    let ty = type_source.trim();
    if let Some(inner) = ty
        .strip_prefix(&format!("{DART_LIST}<"))
        .and_then(|value| value.strip_suffix('>'))
    {
        return if is_visible_type(inner.trim(), declared_type_names) {
            ty.to_owned()
        } else {
            format!("{DART_LIST}<{DART_OBJECT_NULLABLE}>")
        };
    }
    if ty.contains('<') {
        return DART_OBJECT_NULLABLE.to_owned();
    }
    if is_visible_type(ty.trim_end_matches('?'), declared_type_names) {
        ty.to_owned()
    } else {
        DART_OBJECT_NULLABLE.to_owned()
    }
}

fn is_visible_type(type_name: &str, declared_type_names: &[String]) -> bool {
    matches!(
        type_name,
        DART_STRING
            | DART_INT
            | DART_DOUBLE
            | DART_NUM
            | DART_BOOL
            | DART_DATE_TIME
            | DART_OBJECT
            | DART_DYNAMIC
            | DART_VOID
    ) || declared_type_names.iter().any(|name| name == type_name)
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
