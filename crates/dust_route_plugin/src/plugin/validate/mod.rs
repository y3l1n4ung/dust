use std::collections::HashSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{BuiltinType, ClassIr, ConstructorIr, ConstructorParamIr, LibraryIr, TypeIr};

use super::{
    model::RouteAnnotation,
    parse::{parse_route_config, route_config},
};

pub(crate) fn validate_library_routes(library: &LibraryIr) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut paths = HashSet::new();
    let mut names = HashSet::new();
    let local_classes = library
        .classes
        .iter()
        .map(|class| class.name.as_str())
        .collect::<HashSet<_>>();

    for class in &library.classes {
        let Some(config) = route_config(&class.configs) else {
            continue;
        };
        let Some(route) = parse_route_config(config) else {
            diagnostics.push(Diagnostic::error(format!(
                "`@Route` on `{}` requires a string path argument",
                class.name
            )));
            continue;
        };

        if !route.path.starts_with('/') {
            diagnostics.push(Diagnostic::error(format!(
                "route `{}` path `{}` must be absolute",
                class.name, route.path
            )));
        }
        if !paths.insert(route.path.clone()) {
            diagnostics.push(Diagnostic::error(format!(
                "duplicate route path `{}`",
                route.path
            )));
        }
        if let Some(name) = &route.name
            && !names.insert(name.clone())
        {
            diagnostics.push(Diagnostic::error(format!("duplicate route name `{name}`")));
        }

        validate_route_params(class, &route.path, &mut diagnostics);
        validate_visible_route_types(library, class, &route, &local_classes, &mut diagnostics);
    }

    diagnostics
}

fn validate_visible_route_types(
    library: &LibraryIr,
    class: &ClassIr,
    route: &RouteAnnotation,
    local_classes: &HashSet<&str>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(shell) = route.shell.as_deref()
        && !is_visible_type(library, local_classes, shell)
    {
        diagnostics.push(Diagnostic::error(format!(
            "route shell `{shell}` on `{}` must be declared in the same library or imported",
            class.name
        )));
    }
    for guard in &route.guards {
        if !is_visible_type(library, local_classes, guard) {
            diagnostics.push(Diagnostic::error(format!(
                "route guard `{guard}` on `{}` must be declared in the same library or imported",
                class.name
            )));
        }
    }
}

fn is_visible_type(library: &LibraryIr, local_classes: &HashSet<&str>, name: &str) -> bool {
    local_classes.contains(name) || !library.imports.is_empty()
}

fn validate_route_params(class: &ClassIr, path: &str, diagnostics: &mut Vec<Diagnostic>) {
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
        if !is_supported_url_primitive(&param.ty) {
            diagnostics.push(unsupported_param_diagnostic(&class.name, param));
        }
    }
}

fn route_constructor(class: &ClassIr) -> Option<&ConstructorIr> {
    class
        .constructors
        .iter()
        .find(|constructor| constructor.name.is_none() && !constructor.is_factory)
}

fn path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix(':'))
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

fn is_supported_url_primitive(ty: &TypeIr) -> bool {
    matches!(
        ty,
        TypeIr::Builtin {
            kind: BuiltinType::String | BuiltinType::Int | BuiltinType::Double | BuiltinType::Bool,
            ..
        }
    )
}

fn unsupported_param_diagnostic(class_name: &str, param: &ConstructorParamIr) -> Diagnostic {
    Diagnostic::error(format!(
        "route parameter `{}` on `{class_name}` must be a URL primitive (`String`, `int`, `double`, or `bool`)",
        param.name
    ))
}
