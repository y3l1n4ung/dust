use std::collections::{HashMap, HashSet};

use dust_diagnostics::Diagnostic;
use dust_ir::{BuiltinType, ClassIr, ConstructorIr, ConstructorParamIr, DartFileIr, TypeIr};

use super::parse::{route_annotation, route_config};

/// Builds diagnostics for duplicate route annotations.
mod collisions;
/// Validates route shell, guard, and enum type visibility.
mod visibility;

use collisions::{
    RouteCollision, duplicate_route_name_diagnostic, duplicate_route_path_diagnostic,
};
use visibility::{is_visible_type, validate_visible_route_types};

/// Validates all `@AppRoute` pages in a lowered Dart library.
pub(crate) fn validate_library_routes(library: &DartFileIr) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut paths = HashMap::new();
    let mut names = HashMap::new();
    let local_classes = library
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect::<HashSet<_>>();

    for class in &library.classes {
        let has_route = class
            .configs
            .iter()
            .any(|config| config.symbol.0 == "dust_flutter::AppRoute");
        if !has_route {
            continue;
        }
        let Some(config) = route_config(&class.configs) else {
            diagnostics.push(Diagnostic::error(format!(
                "`@AppRoute` on `{}` requires a string path argument",
                class.name
            )));
            continue;
        };
        let route = route_annotation(config);

        if !route.path.starts_with('/') {
            diagnostics.push(Diagnostic::error(format!(
                "route `{}` path `{}` must be absolute",
                class.name, route.path
            )));
        }
        let route_key = RouteCollision {
            page_class: class.name.clone(),
            path: route.path.clone(),
            name: route.name.clone(),
        };
        if let Some(previous) = paths.insert(route.path.clone(), route_key.clone()) {
            diagnostics.push(duplicate_route_path_diagnostic(
                &route.path,
                &previous,
                &route_key,
            ));
        }
        if let Some(name) = &route.name
            && let Some(previous) = names.insert(name.clone(), route_key.clone())
        {
            diagnostics.push(duplicate_route_name_diagnostic(name, &previous, &route_key));
        }

        validate_route_params(
            library,
            class,
            &route.path,
            &local_classes,
            &mut diagnostics,
        );
        validate_visible_route_types(library, class, &route, &local_classes, &mut diagnostics);
    }

    diagnostics
}

/// Validates constructor parameters used by a route path and query string.
fn validate_route_params(
    library: &DartFileIr,
    class: &ClassIr,
    path: &str,
    local_classes: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(constructor) = route_constructor(class) else {
        diagnostics.push(Diagnostic::error(format!(
            "route page `{}` needs an unnamed generative constructor",
            class.name
        )));
        return;
    };

    let path_params = path_params(path);
    for path_param in &path_params {
        let Some(param) = constructor
            .params
            .iter()
            .find(|param| &param.name == path_param)
        else {
            diagnostics.push(Diagnostic::error(format!(
                "route path parameter `:{path_param}` on `{}` has no matching constructor parameter",
                class.name
            )));
            continue;
        };
        if param.ty.is_nullable() {
            diagnostics.push(Diagnostic::error(format!(
                "route path parameter `{}` on `{}` must be required and non-nullable",
                param.name, class.name
            )));
        }
        if param.has_default {
            diagnostics.push(Diagnostic::error(format!(
                "route path parameter `{}` on `{}` cannot use a constructor default",
                param.name, class.name
            )));
        }
        if !is_supported_url_primitive(&param.ty) {
            diagnostics.push(unsupported_param_diagnostic(&class.name, param));
        }
    }

    for param in &constructor.params {
        if param.name == "key" || path_params.contains(&param.name) {
            continue;
        }
        if param.has_default && param.default_value_source.is_none() {
            diagnostics.push(Diagnostic::error(format!(
                "route query parameter `{}` on `{}` has a constructor default that Dust could not preserve",
                param.name, class.name
            )));
        }
        if !is_supported_route_query_param(
            library,
            local_classes,
            &param.ty,
            class,
            param,
            diagnostics,
        ) {
            diagnostics.push(unsupported_param_diagnostic(&class.name, param));
        }
    }
}

/// Returns the unnamed generative constructor for a route page.
fn route_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
}

/// Extracts `:param` placeholders from a route path.
fn path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix(':'))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Returns true when a type can round-trip through route URLs.
fn is_supported_url_primitive(ty: &TypeIr) -> bool {
    matches!(
        ty,
        TypeIr::Builtin {
            kind: BuiltinType::String | BuiltinType::Int | BuiltinType::Double | BuiltinType::Bool,
            ..
        }
    )
}

/// Returns true when a constructor parameter can round-trip through a query string.
fn is_supported_route_query_param(
    library: &DartFileIr,
    local_classes: &HashSet<&str>,
    ty: &TypeIr,
    class: &ClassIr,
    param: &ConstructorParamIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> bool {
    if is_supported_url_primitive(ty)
        || ty.is_named("DateTime")
        || ty.is_named("Uri")
        || is_supported_repeated_query_param(ty)
    {
        return true;
    }
    if matches!(ty, TypeIr::Named { name, args, .. }
        if args.is_empty() && is_visible_enum_like_type(library, local_classes, name))
    {
        return true;
    }
    if matches!(ty, TypeIr::Named { name, .. } if name.as_ref() == "List") {
        diagnostics.push(Diagnostic::error(format!(
            "route repeated query parameter `{}` on `{}` must be `List<String>` or `List<int>`",
            param.name, class.name
        )));
        return true;
    }
    false
}

/// Returns true when a named query type can be referenced as an enum by generated code.
fn is_visible_enum_like_type(
    library: &DartFileIr,
    local_classes: &HashSet<&str>,
    name: &str,
) -> bool {
    library.enums.iter().any(|enum_| enum_.name == name)
        || is_visible_type(library, local_classes, name)
}

/// Returns true for the repeated query shapes supported by generated routing.
fn is_supported_repeated_query_param(ty: &TypeIr) -> bool {
    matches!(
        ty,
        TypeIr::Named { name, args, .. }
            if name.as_ref() == "List"
                && args.len() == 1
                && (args[0].is_builtin(BuiltinType::String)
                    || args[0].is_builtin(BuiltinType::Int))
    )
}

/// Builds a diagnostic for unsupported route constructor parameter types.
fn unsupported_param_diagnostic(class_name: &str, param: &ConstructorParamIr) -> Diagnostic {
    Diagnostic::error(format!(
        "route parameter `{}` on `{class_name}` must be a URL type (`String`, `int`, `double`, `bool`, `DateTime`, `Uri`, enum, `List<String>`, or `List<int>`)",
        param.name
    ))
}
