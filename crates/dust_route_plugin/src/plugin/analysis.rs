use std::path::{Component, Path, PathBuf};

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{ClassIr, DartFileIr, ParamKind, TypeIr};
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

/// Collects route, router, and guard facts from canonical IR.
pub(crate) fn collect_route_workspace_analysis_ir(
    library: &DartFileIr,
    analysis: &mut WorkspaceAnalysisBuilder,
) {
    for class in &library.classes {
        if let Some(route) =
            super::parse::route_config(&class.configs).map(super::parse::route_annotation)
        {
            let fact = RouteFact {
                class_name: class.name.clone(),
                path: route.path.clone(),
                name: route.name.clone(),
                annotation: route,
                import_uri: import_uri_ir(library),
                source_path: library.source_path.clone(),
                imports: library_imports_ir(library),
                params: route_params_ir(class),
            };
            if let Ok(value) = serde_json::to_string(&fact) {
                analysis.add_string_set_value(ROUTES_ANALYSIS_KEY, value);
            }
        }
        if let Some(router) = super::parse::router_config(&class.configs) {
            let fact = RouterFact {
                class_name: class.name.clone(),
                initial: router.initial.clone(),
                not_found: router.not_found.clone(),
                source_path: library.source_path.clone(),
            };
            if let Ok(value) = serde_json::to_string(&fact) {
                analysis.add_string_set_value(ROUTERS_ANALYSIS_KEY, value);
            }
        }
        let guard_fact = GuardFact {
            class_name: class.name.clone(),
            has_unnamed_constructor: class
                .constructors
                .iter()
                .any(|constructor| constructor.name.is_none() && !constructor.is_factory),
            import_uri: import_uri_ir(library),
            source_path: library.source_path.clone(),
            params: guard_params_ir(class),
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

/// Extracts route constructor parameters from a canonical IR class.
fn route_params_ir(class: &ClassIr) -> Vec<RouteParamFact> {
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
            type_source: type_source(&param.ty),
            is_named: matches!(param.kind, ParamKind::Named),
            has_default: param.has_default,
            default_value_source: param.default_value_source.clone(),
        })
        .collect()
}

/// Extracts guard constructor parameters from a canonical IR class.
fn guard_params_ir(class: &ClassIr) -> Vec<GuardParamFact> {
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
            type_source: type_source(&param.ty),
            is_named: matches!(param.kind, ParamKind::Named),
            has_default: param.has_default,
        })
        .collect()
}

/// Renders a normalized constructor type while preserving unresolved values as absent.
fn type_source(ty: &TypeIr) -> Option<String> {
    (!matches!(ty, TypeIr::Unknown)).then(|| DYNAMIC_TYPES.render(ty))
}

/// Builds a package import URI for the current source file.
fn import_uri(context: WorkspaceAnalysisContext<'_>) -> String {
    import_uri_from_paths(
        context.package_name,
        context.package_root,
        context.source_path,
    )
}

/// Builds a package import URI from canonical file metadata.
fn import_uri_ir(library: &DartFileIr) -> String {
    import_uri_from_paths(
        &library.package_name,
        Path::new(&library.package_root),
        Path::new(&library.source_path),
    )
}

/// Builds a package import URI or preserves the source path outside `lib/`.
fn import_uri_from_paths(package_name: &str, package_root: &Path, source_path: &Path) -> String {
    if let Some(path) = source_path
        .strip_prefix(package_root)
        .ok()
        .and_then(|relative| relative.strip_prefix("lib").ok())
    {
        return format!("package:{package_name}/{}", normalize_path(path));
    }

    if let Ok(path) = source_path.strip_prefix("lib") {
        return format!("package:{package_name}/{}", normalize_path(path));
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

/// Returns normalized user imports from canonical IR directives.
fn library_imports_ir(library: &DartFileIr) -> Vec<RouteImport> {
    let mut imports = library
        .import_directives
        .iter()
        .filter_map(|directive| {
            normalize_import_uri_from_paths(
                &library.package_name,
                Path::new(&library.package_root),
                Path::new(&library.source_path),
                &directive.uri,
            )
            .map(|uri| RouteImport {
                uri,
                prefix: directive.prefix.clone(),
                show: directive.show.clone(),
                hide: directive.hide.clone(),
                is_deferred: directive.is_deferred,
            })
        })
        .collect::<Vec<_>>();
    imports.sort();
    imports.dedup();
    imports
}

/// Normalizes an import URI from a route source file into generated-code form.
fn normalize_import_uri(context: WorkspaceAnalysisContext<'_>, uri: &str) -> Option<String> {
    normalize_import_uri_from_paths(
        context.package_name,
        context.package_root,
        context.source_path,
        uri,
    )
}

/// Normalizes an import URI using canonical file metadata.
fn normalize_import_uri_from_paths(
    package_name: &str,
    package_root: &Path,
    source_path: &Path,
    uri: &str,
) -> Option<String> {
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
    let parent = source_path.parent().unwrap_or_else(|| Path::new(""));
    let joined = parent.join(uri);
    let normalized = package_uri_from_source_path(package_name, package_root, &joined)?;
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
