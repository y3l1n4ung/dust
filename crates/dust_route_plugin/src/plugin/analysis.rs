use std::path::{Component, Path, PathBuf};

use dust_parser_dart::{ParameterKind, ParsedClassSurface, ParsedDartFileSurface, ParsedDirective};
use dust_plugin_api::{WorkspaceAnalysisBuilder, WorkspaceAnalysisContext};

use super::{
    constants::{GUARDS_ANALYSIS_KEY, ROUTE, ROUTER, ROUTERS_ANALYSIS_KEY, ROUTES_ANALYSIS_KEY},
    model::{GuardFact, GuardParamFact, RouteFact, RouteImport, RouteParamFact, RouterFact},
    parse::{parse_route_surface, parse_router_surface},
};

/// Collects route, router, and guard facts from parsed Dart surface syntax.
pub(crate) fn collect_route_workspace_analysis(
    context: WorkspaceAnalysisContext<'_>,
    library: &ParsedDartFileSurface,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    for class in &library.classes {
        for annotation in &class.annotations {
            if annotation.is_named(ROUTE) {
                if let Some(route) = parse_route_surface(annotation) {
                    let fact = RouteFact {
                        class_name: class.name.clone(),
                        path: route.path.clone(),
                        name: route.name.clone(),
                        annotation: route,
                        import_uri: import_uri(context),
                        source_path: context.source_path.display().to_string(),
                        imports: library_imports(context, library),
                        params: route_params(class),
                    };
                    if let Ok(value) = serde_json::to_string(&fact) {
                        analysis.add_string_set_value(ROUTES_ANALYSIS_KEY, value);
                    }
                }
            } else if annotation.is_named(ROUTER) {
                let router = parse_router_surface(annotation);
                let fact = RouterFact {
                    class_name: class.name.clone(),
                    initial: router.initial,
                    not_found: router.not_found,
                    source_path: context.source_path.display().to_string(),
                };
                if let Ok(value) = serde_json::to_string(&fact) {
                    analysis.add_string_set_value(ROUTERS_ANALYSIS_KEY, value);
                }
            }
        }
        let guard_fact = GuardFact {
            class_name: class.name.clone(),
            has_unnamed_constructor: class
                .constructors
                .iter()
                .any(|constructor| constructor.name.is_none() && !constructor.is_factory),
            import_uri: import_uri(context),
            source_path: context.source_path.display().to_string(),
            params: guard_params(class),
        };
        if let Ok(value) = serde_json::to_string(&guard_fact) {
            analysis.add_string_set_value(GUARDS_ANALYSIS_KEY, value);
        }
    }
}

/// Extracts route constructor parameters from a parsed page class.
fn route_params(class: &ParsedClassSurface) -> Vec<RouteParamFact> {
    let Some(constructor) = class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
    else {
        return Vec::new();
    };

    constructor
        .params
        .iter()
        .filter(|param| param.name != "key")
        .map(|param| RouteParamFact {
            name: param.name.clone(),
            type_source: param
                .type_source
                .clone()
                .or_else(|| field_type_source(class, &param.name)),
            is_named: matches!(param.kind, ParameterKind::Named),
            has_default: param.has_default,
            default_value_source: param.default_value_source.clone(),
        })
        .collect()
}

/// Finds a field type source that backs a field-formal constructor parameter.
fn field_type_source(class: &ParsedClassSurface, name: &str) -> Option<String> {
    class
        .fields
        .iter()
        .find(|field| field.name == name)
        .and_then(|field| field.type_source.clone())
}

/// Extracts guard constructor parameters from a parsed guard class.
fn guard_params(class: &ParsedClassSurface) -> Vec<GuardParamFact> {
    let Some(constructor) = class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
    else {
        return Vec::new();
    };

    constructor
        .params
        .iter()
        .map(|param| GuardParamFact {
            name: param.name.clone(),
            type_source: param
                .type_source
                .clone()
                .or_else(|| field_type_source(class, &param.name)),
            is_named: matches!(param.kind, ParameterKind::Named),
            has_default: param.has_default,
        })
        .collect()
}

/// Builds a package import URI for the current source file.
fn import_uri(context: WorkspaceAnalysisContext<'_>) -> String {
    let source_path = context.source_path;
    let package_root = context.package_root;
    if let Some(path) = source_path
        .strip_prefix(package_root)
        .ok()
        .and_then(|relative| relative.strip_prefix("lib").ok())
    {
        let normalized = normalize_path(path);
        return format!("package:{}/{}", context.package_name, normalized);
    }

    if let Ok(path) = source_path.strip_prefix("lib") {
        let normalized = normalize_path(path);
        return format!("package:{}/{}", context.package_name, normalized);
    }

    source_path.display().to_string()
}

/// Returns normalized user imports needed by generated route code.
fn library_imports(
    context: WorkspaceAnalysisContext<'_>,
    library: &ParsedDartFileSurface,
) -> Vec<RouteImport> {
    let mut imports = library
        .directives
        .iter()
        .filter_map(|directive| match directive {
            ParsedDirective::Import {
                uri,
                prefix,
                show,
                hide,
                is_deferred,
                ..
            } => normalize_import_uri(context, uri).map(|uri| RouteImport {
                uri,
                prefix: prefix.clone(),
                show: show.clone(),
                hide: hide.clone(),
                is_deferred: *is_deferred,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

/// Normalizes an import URI from a route source file into generated-code form.
fn normalize_import_uri(context: WorkspaceAnalysisContext<'_>, uri: &str) -> Option<String> {
    if matches!(
        uri,
        "package:flutter/material.dart"
            | "package:flutter/cupertino.dart"
            | "package:dust_flutter/route.dart"
            | "package:dust_flutter/dust_flutter.dart"
    ) {
        return None;
    }
    if uri.starts_with("dart:") || uri.starts_with("package:") {
        return Some(uri.to_owned());
    }
    if uri == "route.g.dart" || uri == "routing_core.dart" || uri == "route_annotations.dart" {
        return None;
    }
    let parent = context
        .source_path
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let joined = parent.join(uri);
    let normalized =
        package_uri_from_source_path(context.package_name, context.package_root, &joined)?;
    if normalized.ends_with("/route_annotations.dart") {
        None
    } else {
        Some(normalized)
    }
}

/// Converts a source path under `lib/` into a package import URI.
fn package_uri_from_source_path(
    package_name: &str,
    package_root: &Path,
    source_path: &Path,
) -> Option<String> {
    let relative = source_path
        .strip_prefix(package_root)
        .ok()
        .and_then(|path| path.strip_prefix("lib").ok())
        .or_else(|| source_path.strip_prefix("lib").ok())?;
    let normalized = normalize_path(&normalize_components(relative));
    Some(format!("package:{package_name}/{normalized}"))
}

/// Resolves `.` and `..` path components without touching the filesystem.
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

/// Renders a path with forward slashes for Dart package imports.
fn normalize_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
