use std::path::{Component, Path, PathBuf};

use dust_dart_emit::{
    DART_BOOL, DART_DATE_TIME, DART_DOUBLE, DART_DYNAMIC, DART_INT, DART_LIST, DART_NUM,
    DART_OBJECT, DART_OBJECT_NULLABLE, DART_STRING, DART_VOID, DYNAMIC_TYPES,
};
use dust_ir::DartFileIr;
use dust_plugin_api::WorkspaceAnalysisBuilder;

use super::{
    constants::{STATES_ANALYSIS_KEY, VIEW_MODELS_ANALYSIS_KEY},
    model::{StateFact, StateFieldFact, ViewModelFact},
};
/// Collects state and view model facts from canonical IR.
pub(crate) fn collect_state_workspace_analysis_ir(
    library: &DartFileIr,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    let declared_type_names = library
        .classes
        .iter()
        .map(|class| class.name.clone())
        .chain(library.enums.iter().map(|enum_| enum_.name.clone()))
        .collect::<Vec<_>>();
    for class in &library.classes {
        collect_state_fact_ir(class, &declared_type_names, analysis);
        if let Some(annotation) = super::parse::view_model_config(&class.configs)
            .and_then(super::parse::parse_view_model_config)
        {
            let fact = ViewModelFact {
                class_name: class.name.clone(),
                state_type: annotation.state_type,
                args_type: annotation.args_type,
                initial_source: annotation.initial_source,
                mode: annotation.mode,
                generated_base_class: format!("${}", class.name),
                import_uri: import_uri_ir(library),
            };
            if let Ok(value) = serde_json::to_string(&fact) {
                analysis.add_string_set_value(VIEW_MODELS_ANALYSIS_KEY, value);
            }
        }
    }
}

/// Records field metadata for a canonical IR class as a possible state class.
fn collect_state_fact_ir(
    class: &dust_ir::ClassIr,
    declared_type_names: &[String],
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    let fields = class
        .fields
        .iter()
        .map(|field| StateFieldFact {
            name: field.name.clone(),
            type_source: sanitize_type_source(
                &DYNAMIC_TYPES.render(&field.ty),
                declared_type_names,
            ),
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

/// Rewrites unavailable Dart type sources to nullable `Object` fallbacks.
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

/// Returns whether a type can be referenced from generated selector code.
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

/// Builds an import URI from canonical file metadata.
fn import_uri_ir(library: &DartFileIr) -> String {
    import_uri_from_paths(
        &library.package_name,
        Path::new(&library.package_root),
        Path::new(&library.source_path),
    )
}

/// Builds a package import URI or preserves the source path when it is outside `lib/`.
fn import_uri_from_paths(package_name: &str, package_root: &Path, source_path: &Path) -> String {
    if let Some(path) = source_path
        .strip_prefix(package_root)
        .ok()
        .and_then(|relative| relative.strip_prefix("lib").ok())
    {
        return format!("package:{package_name}/{}", normalize_path(path));
    }
    source_path.display().to_string()
}

/// Converts a filesystem path to a normalized forward-slash Dart import path.
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

/// Collapses `..` components so generated import paths stay stable.
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
